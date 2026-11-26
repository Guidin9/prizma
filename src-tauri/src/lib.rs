mod colour;
mod engine;
mod k617;
mod layout;
mod scenes;

use engine::layers::Scene;
use scenes::SavedScene;
use engine::{Engine, Mode};
use hidapi::HidApi;
use k617::effects::{self, EffectInfo, HwEffect};
use k617::hid::{self, K617};
use parking_lot::Mutex;
use serde::Serialize;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager, RunEvent, State, WindowEvent};

struct Device {
    api: Option<HidApi>,
    dev: Option<K617>,
    /// Fn lock = firmware keymap profile 1
    lock: Option<bool>,
    /// firmware effect to restore when software effects stop
    last_hw: Option<HwEffect>,
}

type Shared = Arc<Mutex<Device>>;

const DEFAULT_HW: HwEffect = HwEffect { id: 0x03, colour: None, speed: 3, brightness: 4, reverse: false };

#[derive(Serialize, Clone)]
struct Status {
    connected: bool,
    lock: Option<bool>,
    effect: Option<HwEffect>,
    /// settings the keyboard keeps for every effect (catalogue order)
    stored: Vec<HwEffect>,
    engine: Mode,
    error: Option<String>,
}

#[derive(Serialize, Clone)]
struct KbdEvent {
    kind: &'static str,
    raw: Vec<u8>,
}

fn ensure_open(d: &mut Device) -> Result<(), String> {
    if d.dev.is_some() {
        return Ok(());
    }
    if d.api.is_none() {
        d.api = Some(HidApi::new().map_err(|e| e.to_string())?);
    }
    let api = d.api.as_mut().unwrap();
    api.refresh_devices().map_err(|e| e.to_string())?;
    let dev = K617::open(api).map_err(|e| e.to_string())?;
    d.lock = k617::saved_profile(&dev).ok().map(|p| p == 1);
    d.dev = Some(dev);
    Ok(())
}

fn with_dev<T>(state: &Shared, f: impl FnOnce(&K617) -> anyhow::Result<T>) -> Result<T, String> {
    let mut d = state.lock();
    ensure_open(&mut d)?;
    let res = f(d.dev.as_ref().unwrap());
    if res.is_err() {
        d.dev = None; // force a reopen next time (unplugged keyboard etc.)
    }
    res.map_err(|e| e.to_string())
}

#[tauri::command]
fn status(state: State<'_, Shared>, eng: State<'_, Arc<Engine>>) -> Status {
    let engine = eng.shared.lock().mode;
    match with_dev(&state, |dev| k617::read_settings(dev)) {
        Ok(k617::Settings { current, stored }) => {
            let mut d = state.lock();
            if current.id != effects::SELF_DEFINE {
                d.last_hw = Some(current);
            }
            Status { connected: true, lock: d.lock, effect: Some(current), stored, engine, error: None }
        }
        Err(e) => Status { connected: false, lock: None, effect: None, stored: Vec::new(), engine, error: Some(e) },
    }
}

#[tauri::command]
fn effect_catalog() -> Vec<EffectInfo> {
    effects::EFFECTS.to_vec()
}

#[tauri::command]
fn layouts() -> &'static str {
    k617::LAYOUTS_JSON
}

fn apply_hw(state: &Shared, eng: &Engine, effect: HwEffect) -> Result<HwEffect, String> {
    if eng.shared.lock().mode == Mode::Live {
        eng.shared.lock().mode = Mode::Off; // a firmware effect takes the keyboard back
    }
    let res = with_dev(state, |dev| {
        k617::apply_effect(dev, &effect)?;
        k617::current_effect(dev)
    })?;
    state.lock().last_hw = Some(res);
    Ok(res)
}

#[tauri::command]
async fn apply_effect(state: State<'_, Shared>, eng: State<'_, Arc<Engine>>, effect: HwEffect) -> Result<HwEffect, String> {
    let (state, eng) = (state.inner().clone(), eng.inner().clone());
    tauri::async_runtime::spawn_blocking(move || apply_hw(&state, &eng, effect)).await.map_err(|e| e.to_string())?
}

#[tauri::command]
fn set_layout(eng: State<'_, Arc<Engine>>, id: String) -> Result<(), String> {
    let l = layout::load(k617::LAYOUTS_JSON, &id).ok_or("unknown layout")?;
    eng.shared.lock().layout = Arc::new(l);
    Ok(())
}

#[tauri::command]
fn set_visible(eng: State<'_, Arc<Engine>>, visible: bool) {
    eng.shared.lock().visible = visible;
}

/// Show `scene` on the stage only (nothing is sent to the keyboard). While live, the new scene
/// goes straight to the keyboard.
#[tauri::command]
fn engine_preview(eng: State<'_, Arc<Engine>>, scene: Option<Scene>) {
    let mut s = eng.shared.lock();
    let some = scene.is_some();
    if let Some(sc) = scene {
        s.scene = Some(sc);
    }
    if s.mode != Mode::Live {
        s.mode = if some { Mode::Preview } else { Mode::Off };
    }
}

/// Start streaming `scene` to the keyboard.
#[tauri::command]
async fn engine_start(state: State<'_, Shared>, eng: State<'_, Arc<Engine>>, scene: Scene) -> Result<(), String> {
    let (state, eng) = (state.inner().clone(), eng.inner().clone());
    tauri::async_runtime::spawn_blocking(move || {
        if eng.shared.lock().mode != Mode::Live {
            // remember the firmware effect so stopping can hand the keyboard back to it
            if let Ok(e) = with_dev(&state, |dev| k617::current_effect(dev)) {
                if e.id != effects::SELF_DEFINE {
                    state.lock().last_hw = Some(e);
                }
            }
        }
        let mut s = eng.shared.lock();
        s.scene = Some(scene);
        s.mode = Mode::Live;
        Ok(())
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Stop streaming and give the keyboard back its own effect.
#[tauri::command]
async fn engine_stop(state: State<'_, Shared>, eng: State<'_, Arc<Engine>>) -> Result<HwEffect, String> {
    let (state, eng) = (state.inner().clone(), eng.inner().clone());
    tauri::async_runtime::spawn_blocking(move || stop_and_restore(&state, &eng)).await.map_err(|e| e.to_string())?
}

fn stop_and_restore(state: &Shared, eng: &Engine) -> Result<HwEffect, String> {
    {
        let mut s = eng.shared.lock();
        s.mode = if s.scene.is_some() { Mode::Preview } else { Mode::Off };
    }
    thread::sleep(Duration::from_millis(80)); // let the engine finish its last frame
    let hw = state.lock().last_hw.unwrap_or(DEFAULT_HW);
    apply_hw(state, eng, hw)
}

/// Tray menu entries, renamed when the interface language changes.
struct Tray {
    show: MenuItem<tauri::Wry>,
    quit: MenuItem<tauri::Wry>,
}

fn tray_texts(lang: &str) -> (&'static str, &'static str) {
    if lang == "tr" {
        ("Prizma'yı aç", "Çıkış")
    } else {
        ("Open Prizma", "Quit")
    }
}

fn settings_path(app: &AppHandle) -> Option<std::path::PathBuf> {
    app.path().app_config_dir().ok().map(|d| d.join("settings.json"))
}

/// Language for the tray before the window has ever opened: the one the user chose, else
/// Windows' display language.
fn startup_lang(app: &AppHandle) -> String {
    let chosen = settings_path(app)
        .and_then(|p| std::fs::read_to_string(p).ok())
        .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
        .and_then(|v| v.get("lang").and_then(|l| l.as_str()).map(str::to_string));
    chosen.unwrap_or_else(|| {
        // primary language id 0x1F = Turkish
        let id = unsafe { windows::Win32::Globalization::GetUserDefaultUILanguage() };
        if id & 0x3FF == 0x1F { "tr" } else { "en" }.to_string()
    })
}

#[tauri::command]
fn set_language(app: AppHandle, tray: State<'_, Tray>, lang: String) {
    let lang = if lang == "tr" { "tr" } else { "en" };
    let (show, quit) = tray_texts(lang);
    let _ = tray.show.set_text(show);
    let _ = tray.quit.set_text(quit);
    if let Some(p) = settings_path(&app) {
        if let Some(dir) = p.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let _ = std::fs::write(p, serde_json::json!({ "lang": lang }).to_string());
    }
}

fn scenes_path(app: &AppHandle) -> Result<std::path::PathBuf, String> {
    app.path().app_config_dir().map(|d| d.join("scenes.json")).map_err(|e| e.to_string())
}

#[tauri::command]
fn scenes_list(app: AppHandle) -> Result<Vec<SavedScene>, String> {
    Ok(scenes::load(&scenes_path(&app)?))
}

#[tauri::command]
fn scene_save(app: AppHandle, name: String, scene: Scene) -> Result<Vec<SavedScene>, String> {
    let path = scenes_path(&app)?;
    let mut list = scenes::load(&path);
    scenes::upsert(&mut list, &name, scene).map_err(|e| e.to_string())?;
    scenes::store(&path, &list).map_err(|e| e.to_string())?;
    Ok(list)
}

#[tauri::command]
fn scene_delete(app: AppHandle, name: String) -> Result<Vec<SavedScene>, String> {
    let path = scenes_path(&app)?;
    let mut list = scenes::load(&path);
    scenes::remove(&mut list, &name);
    scenes::store(&path, &list).map_err(|e| e.to_string())?;
    Ok(list)
}

fn event_kind(raw: &[u8]) -> &'static str {
    match raw {
        [0x03, 0x20, _, 0x01, ..] => "fn_down",
        [0x03, 0x20, _, 0x00, ..] => "fn_up",
        [0x03, 0x01, 0x03, _, ..] => "lock_toggle",
        [0x03, 0x0B, ..] => "effect_key",
        [0x03, 0x0C, ..] => "brightness_key",
        [0x03, 0x0D, ..] => "speed_key",
        _ => "other",
    }
}

/// Listens on Col03 for keyboard events; reconnects when the keyboard is unplugged.
fn spawn_event_thread(app: AppHandle, state: Shared, eng: Arc<Engine>) {
    thread::spawn(move || loop {
        let dev = {
            let mut d = state.lock();
            if d.api.is_none() {
                d.api = HidApi::new().ok();
            }
            d.api.as_mut().and_then(|api| {
                let _ = api.refresh_devices();
                hid::open_events(api).ok()
            })
        };
        let Some(dev) = dev else {
            let _ = app.emit("kbd-connection", false);
            thread::sleep(Duration::from_secs(2));
            continue;
        };
        let _ = app.emit("kbd-connection", true);
        let mut buf = [0u8; 16];
        loop {
            match dev.read_timeout(&mut buf, 250) {
                Ok(0) => continue,
                Ok(n) => {
                    let raw = buf[..n].to_vec();
                    let kind = event_kind(&raw);
                    match kind {
                        "lock_toggle" => {
                            let mut d = state.lock();
                            d.lock = d.lock.map(|v| !v).or(Some(true));
                            eng.shared.lock().lock = d.lock.unwrap_or(false);
                            let _ = app.emit("kbd-lock", d.lock);
                        }
                        "fn_down" => eng.press_fn(),
                        _ => {}
                    }
                    let _ = app.emit("kbd-event", KbdEvent { kind, raw });
                }
                Err(_) => {
                    state.lock().dev = None;
                    break;
                }
            }
        }
    });
}

/// Show the window, creating it first: it is destroyed while Prizma sits in the tray, which frees
/// the web view (about 500 MB and a steady few % CPU when kept alive but hidden).
fn show_main(app: &AppHandle) {
    let w = match app.get_webview_window("main") {
        Some(w) => w,
        None => {
            let Some(cfg) = app.config().app.windows.iter().find(|w| w.label == "main").cloned() else { return };
            match tauri::WebviewWindowBuilder::from_config(app, &cfg).and_then(|b| b.build()) {
                Ok(w) => w,
                Err(_) => return,
            }
        }
    };
    let _ = w.show();
    let _ = w.unminimize();
    let _ = w.set_focus();
    app.state::<Arc<Engine>>().shared.lock().visible = true;
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let shared: Shared = Arc::new(Mutex::new(Device { api: None, dev: None, lock: None, last_hw: None }));
    let eng = Engine::new(layout::load(k617::LAYOUTS_JSON, "tr_iso").expect("layout"));
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| show_main(app)))
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--hidden"]),
        ))
        .plugin(tauri_plugin_opener::init())
        .manage(shared.clone())
        .manage(eng.clone())
        .invoke_handler(tauri::generate_handler![
            status,
            effect_catalog,
            layouts,
            apply_effect,
            set_layout,
            set_visible,
            engine_preview,
            engine_start,
            engine_stop,
            scenes_list,
            scene_save,
            scene_delete,
            set_language
        ])
        .setup(move |app| {
            let (show_text, quit_text) = tray_texts(&startup_lang(app.handle()));
            let show = MenuItem::with_id(app, "show", show_text, true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", quit_text, true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show, &quit])?;
            app.manage(Tray { show: show.clone(), quit: quit.clone() });
            let (q_state, q_eng) = (shared.clone(), eng.clone());
            TrayIconBuilder::with_id("tray")
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("Prizma")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(move |app, ev| match ev.id.as_ref() {
                    "show" => show_main(app),
                    "quit" => {
                        if q_eng.shared.lock().mode == Mode::Live {
                            let _ = stop_and_restore(&q_state, &q_eng); // never leave the keyboard frozen
                        }
                        app.exit(0)
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, ev| {
                    if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = ev {
                        show_main(tray.app_handle());
                    }
                })
                .build(app)?;
            if !std::env::args().any(|a| a == "--hidden") {
                show_main(app.handle());
            }
            let (h1, h2, h3) = (app.handle().clone(), app.handle().clone(), app.handle().clone());
            let send_state = shared.clone();
            eng.spawn(
                Box::new(move |frame| with_dev(&send_state, |dev| dev.send_perkey(frame)).map_err(anyhow::Error::msg)),
                Box::new(move |keys| {
                    let _ = h1.emit("engine-frame", keys);
                }),
                Box::new(move |msg| {
                    let _ = h2.emit("engine-error", msg);
                }),
            );
            spawn_event_thread(h3, shared.clone(), eng.clone());
            Ok(())
        })
        .on_window_event(|window, event| match event {
            // closing the window only sends Prizma to the tray: the window (and its web view) is
            // destroyed, the engine keeps running; `show_main` builds a new window
            WindowEvent::CloseRequested { .. } | WindowEvent::Destroyed => {
                let eng = window.state::<Arc<Engine>>();
                let mut s = eng.shared.lock();
                s.visible = false;
                if s.mode == Mode::Preview {
                    s.mode = Mode::Off;
                }
            }
            WindowEvent::Focused(_) => {
                let visible = window.is_visible().unwrap_or(true);
                window.state::<Arc<Engine>>().shared.lock().visible = visible;
            }
            _ => {}
        })
        .build(tauri::generate_context!())
        .expect("error while building Prizma")
        .run(|_app, event| {
            // no window is not a reason to quit; only the tray's Quit (an explicit exit code) is
            if let RunEvent::ExitRequested { api, code, .. } = event {
                if code.is_none() {
                    api.prevent_exit();
                }
            }
        });
}
