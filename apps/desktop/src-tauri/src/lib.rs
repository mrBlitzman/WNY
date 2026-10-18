mod macos;

use parking_lot::Mutex;
use std::sync::Arc;
use std::time::Duration;
use tauri::Manager;
use wny_core::mode::{Language, Mode, Slot, SlotChoice};
use wny_core::orchestrator::{Orchestrator, UiSnapshot};
use wny_egress::{EgressGuard, PlatformGuard};

struct AppState {
    orch: Mutex<Orchestrator>,
    guard: Mutex<PlatformGuard>,
}

fn with_orch<F, T>(state: &AppState, f: F) -> Result<T, String>
where
    F: FnOnce(&mut Orchestrator) -> wny_core::Result<T>,
{
    let mut g = state.orch.lock();
    f(&mut g).map_err(|e| e.to_string())
}

#[tauri::command]
fn snapshot(state: tauri::State<Arc<AppState>>) -> Result<UiSnapshot, String> {
    Ok(state.orch.lock().snapshot())
}

#[tauri::command]
fn set_on(state: tauri::State<Arc<AppState>>, on: bool) -> Result<UiSnapshot, String> {
    let snap = with_orch(&state, |o| o.set_on(on))?;
    let mut guard = state.guard.lock();
    if on {
        let mode = Mode::parse(&snap.mode).unwrap_or(Mode::Protection);
        let _ = guard.engage(mode);
    } else {
        let _ = guard.disengage();
    }
    Ok(snap)
}

#[tauri::command]
fn set_mode(state: tauri::State<Arc<AppState>>, mode: String) -> Result<UiSnapshot, String> {
    let m = Mode::parse(&mode).ok_or_else(|| "bad mode".to_string())?;
    with_orch(&state, |o| o.set_mode(m))
}

#[tauri::command]
fn set_language(state: tauri::State<Arc<AppState>>, lang: String) -> Result<UiSnapshot, String> {
    with_orch(&state, |o| o.set_language(Language::parse(&lang)))
}

#[tauri::command]
fn set_slot(
    state: tauri::State<Arc<AppState>>,
    slot: String,
    choice: String,
) -> Result<UiSnapshot, String> {
    let s = Slot::parse(&slot).ok_or_else(|| "bad slot".to_string())?;
    with_orch(&state, |o| o.set_slot(s, SlotChoice::parse(&choice)))
}

#[tauri::command]
fn finish_wizard(
    state: tauri::State<Arc<AppState>>,
    start_at_login: bool,
) -> Result<UiSnapshot, String> {
    with_orch(&state, |o| o.finish_wizard(start_at_login))
}

#[tauri::command]
fn new_identity(state: tauri::State<Arc<AppState>>) -> Result<UiSnapshot, String> {
    with_orch(&state, |o| o.new_identity())
}

#[tauri::command]
fn captive(state: tauri::State<Arc<AppState>>) -> Result<UiSnapshot, String> {
    let snap = with_orch(&state, |o| o.open_captive_portal())?;
    let mut guard = state.guard.lock();
    let _ = guard.allow_captive_portal(Duration::from_secs(300));
    Ok(snap)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    std::env::set_var("WNY_DEV", std::env::var("WNY_DEV").unwrap_or_else(|_| "1".into()));
    let orch = Orchestrator::open_default().expect("orchestrator");
    let mode = orch.snapshot().mode;
    let m = Mode::parse(&mode).unwrap_or(Mode::Protection);
    let guard = PlatformGuard::new(m).expect("egress");
    let state = Arc::new(AppState {
        orch: Mutex::new(orch),
        guard: Mutex::new(guard),
    });

    tauri::Builder::default()
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            snapshot,
            set_on,
            set_mode,
            set_language,
            set_slot,
            finish_wizard,
            new_identity,
            captive
        ])
        .setup(|app| {
            #[cfg(desktop)]
            {
                let _ = app;
                // Tray is the daily switch. No other vendor UI is launched.
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("WNY desktop");
}
