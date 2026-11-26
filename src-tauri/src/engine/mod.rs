//! Software effect engine: renders effects on the host and streams per-key frames.
//!
//! The firmware switches to effect 21 (host mode) on the first per-key frame and freezes on the
//! last one; stopping the engine therefore re-applies a firmware effect (done by the caller).

pub mod audio;
pub mod effects;
pub mod input;
pub mod layers;
pub mod screen;

use crate::layout::Layout;
use effects::{AudioLevels, Inputs, ScreenGrid};
use layers::{Scene, SceneState};
use parking_lot::Mutex;
use serde::Serialize;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    Off,
    Preview,
    Live,
}

pub struct Shared {
    pub scene: Option<Scene>,
    pub mode: Mode,
    pub layout: Arc<Layout>,
    pub lock: bool,
    pub visible: bool,
    /// keys pressed since the last frame
    pub presses: Vec<usize>,
}

/// What the patched firmware writes to the Fn LED while Fn lock (profile 1) is on.
const FN_LOCK_WHITE: [u8; 3] = [255, 255, 255];

pub type SendFrame = Box<dyn Fn(&[u8; 382]) -> anyhow::Result<()> + Send>;
pub type EmitFrame = Box<dyn Fn(Vec<[u8; 3]>) + Send>;
pub type EmitError = Box<dyn Fn(String) + Send>;

pub struct Engine {
    pub shared: Arc<Mutex<Shared>>,
    audio_on: Arc<AtomicBool>,
    screen_on: Arc<AtomicBool>,
    hook_started: AtomicBool,
    audio: Arc<Mutex<AudioLevels>>,
    screen: Arc<Mutex<ScreenGrid>>,
}

impl Engine {
    pub fn new(layout: Layout) -> Arc<Self> {
        let e = Arc::new(Self {
            shared: Arc::new(Mutex::new(Shared {
                scene: None,
                mode: Mode::Off,
                layout: Arc::new(layout),
                lock: false,
                visible: true,
                presses: Vec::new(),
            })),
            audio_on: Arc::new(AtomicBool::new(false)),
            screen_on: Arc::new(AtomicBool::new(false)),
            hook_started: AtomicBool::new(false),
            audio: Arc::new(Mutex::new(AudioLevels::default())),
            screen: Arc::new(Mutex::new(ScreenGrid::default())),
        });
        audio::spawn(e.audio.clone(), e.audio_on.clone());
        screen::spawn(e.screen.clone(), e.screen_on.clone());
        e
    }

    pub fn press(&self, key_index: usize) {
        let mut s = self.shared.lock();
        if s.mode != Mode::Off && s.presses.len() < 64 {
            s.presses.push(key_index);
        }
    }

    pub fn press_scancode(&self, sc: u16) {
        let idx = {
            let s = self.shared.lock();
            s.layout.keys.iter().position(|k| crate::layout::scancodes(&k.ansi).contains(&sc))
        };
        if let Some(i) = idx {
            self.press(i);
        }
    }

    pub fn press_fn(&self) {
        let idx = self.shared.lock().layout.keys.iter().position(|k| k.ansi == "Fn");
        if let Some(i) = idx {
            self.press(i);
        }
    }

    /// Run only the inputs the visible layers need (audio capture and screen capture cost CPU).
    fn ensure_inputs(self: &Arc<Self>, scene: &Scene) {
        let any = |list: &[&str]| scene.kinds().any(|k| list.contains(&k));
        self.audio_on.store(any(effects::AUDIO), Ordering::Relaxed);
        self.screen_on.store(any(effects::SCREEN), Ordering::Relaxed);
        if any(effects::REACTIVE) && !self.hook_started.swap(true, Ordering::Relaxed) {
            let me = self.clone();
            input::start(move |sc| me.press_scancode(sc));
        }
    }

    pub fn spawn(self: &Arc<Self>, send: SendFrame, emit: EmitFrame, on_error: EmitError) {
        let me = self.clone();
        thread::spawn(move || {
            let t0 = Instant::now();
            let mut st = SceneState::default();
            let mut out: Vec<effects::Rgb> = Vec::new();
            let mut frame = [0u8; 382];
            frame[..4].copy_from_slice(&[0x08, 0x0A, 0x7A, 0x01]);
            let mut n: u64 = 0;
            let mut timer_period = false;
            loop {
                let (scene, mode, layout, lock, visible, presses) = {
                    let mut s = me.shared.lock();
                    (s.scene.clone(), s.mode, s.layout.clone(), s.lock, s.visible, std::mem::take(&mut s.presses))
                };
                let live = mode == Mode::Live;
                if live != timer_period {
                    timer::set(live);
                    timer_period = live;
                }
                let idle = mode == Mode::Off || (mode == Mode::Preview && !visible);
                let Some(scene) = scene.filter(|_| !idle) else {
                    // nothing to draw: stop capturing sound and screen too
                    me.audio_on.store(false, Ordering::Relaxed);
                    me.screen_on.store(false, Ordering::Relaxed);
                    thread::sleep(Duration::from_millis(120));
                    continue;
                };
                me.ensure_inputs(&scene);
                let start = Instant::now();
                let t = t0.elapsed().as_secs_f32();
                let pressed: Vec<(usize, f32)> = presses.into_iter().map(|i| (i, t)).collect();
                let audio = me.audio.lock().clone();
                let screen = me.screen.lock().clone();
                layers::render(&scene, &layout, t, &Inputs { presses: &pressed, audio: &audio, screen: &screen }, &mut st, &mut out);
                let mut keys: Vec<[u8; 3]> = out.iter().map(|c| [c[0].clamp(0.0, 255.0) as u8, c[1].clamp(0.0, 255.0) as u8, c[2].clamp(0.0, 255.0) as u8]).collect();
                let fn_key = layout.keys.iter().position(|k| k.ansi == "Fn");
                if lock {
                    if let Some(i) = fn_key {
                        keys[i] = [255, 255, 255];
                    }
                }
                if live {
                    frame[4..].fill(0);
                    for (i, (k, rgb)) in layout.keys.iter().zip(&keys).enumerate() {
                        // the stage shows sRGB; the LEDs need it corrected. Exception: the Fn-lock
                        // white. The patched firmware paints the Fn LED FF FF FF itself every ~30 ms
                        // while profile 1 is active, into the same buffer these frames fill; sending
                        // a different (corrected) white made the key flicker between the two.
                        let led_rgb = if lock && Some(i) == fn_key { FN_LOCK_WHITE } else { crate::colour::to_led_u8(*rgb) };
                        for &led in &k.leds {
                            let o = 4 + led * 3;
                            if o + 3 <= frame.len() {
                                frame[o..o + 3].copy_from_slice(&led_rgb);
                            }
                        }
                    }
                    if let Err(e) = send(&frame) {
                        me.shared.lock().mode = Mode::Preview;
                        on_error(format!("could not send a frame to the keyboard: {e}"));
                    }
                }
                if visible && (!live || n % 2 == 0) {
                    emit(keys);
                }
                n += 1;
                let budget = if live { Duration::from_micros(16_667) } else { Duration::from_micros(33_333) };
                if let Some(rest) = budget.checked_sub(start.elapsed()) {
                    thread::sleep(rest);
                }
            }
        });
    }
}

/// 1 ms timer resolution while streaming so 60 fps pacing is smooth.
mod timer {
    pub fn set(on: bool) {
        unsafe {
            if on {
                windows::Win32::Media::timeBeginPeriod(1);
            } else {
                windows::Win32::Media::timeEndPeriod(1);
            }
        }
    }
}
