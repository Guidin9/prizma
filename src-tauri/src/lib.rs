mod k617;

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
use tauri::{AppHandle, Emitter, Manager, State, WindowEvent};

struct Device {
    api: Option<HidApi>,
    dev: Option<K617>,
    /// Fn lock = firmware keymap profile 1
    lock: Option<bool>,
}

type Shared = Arc<Mutex<Device>>;

#[derive(Serialize, Clone)]
struct Status {
    connected: bool,
    lock: Option<bool>,
    effect: Option<HwEffect>,
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
fn status(state: State<'_, Shared>) -> Status {
    match with_dev(&state, |dev| k617::current_effect(dev)) {
        Ok(effect) => Status { connected: true, lock: state.lock().lock, effect: Some(effect), error: None },
        Err(e) => Status { connected: false, lock: None, effect: None, error: Some(e) },
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

#[tauri::command]
async fn apply_effect(state: State<'_, Shared>, effect: HwEffect) -> Result<HwEffect, String> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        with_dev(&state, |dev| {
            k617::apply_effect(dev, &effect)?;
            k617::current_effect(dev)
        })
    })
    .await
    .map_err(|e| e.to_string())?
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
fn spawn_event_thread(app: AppHandle, state: Shared) {
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
                    if kind == "lock_toggle" {
                        let mut d = state.lock();
                        d.lock = d.lock.map(|v| !v).or(Some(true));
                        let _ = app.emit("kbd-lock", d.lock);
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

fn show_main(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let shared: Shared = Arc::new(Mutex::new(Device { api: None, dev: None, lock: None }));
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| show_main(app)))
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--hidden"]),
        ))
        .plugin(tauri_plugin_opener::init())
        .manage(shared.clone())
        .invoke_handler(tauri::generate_handler![status, effect_catalog, layouts, apply_effect])
        .setup(move |app| {
            let show = MenuItem::with_id(app, "show", "Prizma'yı aç", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Çıkış", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show, &quit])?;
            TrayIconBuilder::with_id("tray")
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("Prizma")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, ev| match ev.id.as_ref() {
                    "show" => show_main(app),
                    "quit" => app.exit(0),
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
            spawn_event_thread(app.handle().clone(), shared.clone());
            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close(); // keep running in the tray
                let _ = window.hide();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running Prizma");
}
