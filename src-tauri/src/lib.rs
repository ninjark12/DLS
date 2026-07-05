mod config;
mod firmware;
mod hid;
mod switcher;

use std::sync::{Arc, Mutex};
use std::sync::mpsc;

use tauri::Emitter;

use config::Config;
use switcher::SwitcherEvent;

struct AppState {
    config: Arc<Mutex<Config>>,
    switcher_thread_id: Mutex<Option<u32>>,
    switcher_handle: Mutex<Option<std::thread::JoinHandle<()>>>,
}

#[tauri::command]
fn get_config(state: tauri::State<AppState>) -> Config {
    state.config.lock().unwrap().clone()
}

#[tauri::command]
fn save_config(state: tauri::State<AppState>, new_config: Config) -> Result<(), String> {
    config::write_config(&new_config)?;
    *state.config.lock().unwrap() = new_config;
    Ok(())
}

#[tauri::command]
fn list_devices() -> Vec<hid::KeyboardInfo> {
    hid::list_keyboards()
}

#[tauri::command]
fn start_switcher(
    state: tauri::State<AppState>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    let mut handle_guard = state.switcher_handle.lock().unwrap();
    if handle_guard.is_some() {
        return Err("Switcher already running".to_string());
    }

    let config = Arc::clone(&state.config);
    let (event_tx, event_rx) = mpsc::channel::<SwitcherEvent>();
    let (tid_tx, tid_rx) = mpsc::sync_channel::<u32>(1);

    let handle = std::thread::spawn(move || {
        switcher::run(config, event_tx, tid_tx);
    });

    let thread_id = tid_rx.recv().map_err(|e| e.to_string())?;
    *state.switcher_thread_id.lock().unwrap() = Some(thread_id);
    *handle_guard = Some(handle);

    std::thread::spawn(move || {
        for event in event_rx {
            app_handle.emit("switcher-status", &event).ok();
        }
    });

    Ok(())
}

#[tauri::command]
fn stop_switcher(state: tauri::State<AppState>) -> Result<(), String> {
    let thread_id = state.switcher_thread_id.lock().unwrap().take();
    if let Some(tid) = thread_id {
        switcher::stop(tid);
        if let Some(handle) = state.switcher_handle.lock().unwrap().take() {
            handle.join().ok();
        }
        Ok(())
    } else {
        Err("Switcher not running".to_string())
    }
}

#[tauri::command]
fn get_status(state: tauri::State<AppState>) -> bool {
    state.switcher_thread_id.lock().unwrap().is_some()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let config = Arc::new(Mutex::new(config::get_config()));

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState {
            config,
            switcher_thread_id: Mutex::new(None),
            switcher_handle: Mutex::new(None),
        })
        .invoke_handler(tauri::generate_handler![
            get_config,
            save_config,
            list_devices,
            start_switcher,
            stop_switcher,
            get_status,
            firmware::flash_half,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
