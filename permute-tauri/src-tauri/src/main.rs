// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod state;

use commands::*;
use permute::permute_files::PermuteUpdate;
use state::{AppState, SharedState, PermuteProgressEvent, build_progress_event, update_state_from_message};
use std::{sync::Arc, thread};
use tauri::{Emitter, Manager, WindowEvent};

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let app_handle = app.handle().clone();

            // Config path in platform userData directory
            let config_path = app
                .path()
                .app_data_dir()
                .expect("failed to resolve app data dir")
                .join("config.json");

            // Create the update channel
            let (permute_tx, permute_rx) =
                crossbeam_channel::bounded::<PermuteUpdate>(100);

            let mut initial_state = SharedState::init(permute_tx);

            // Load saved settings (ignore errors on first launch)
            let _ = initial_state.read_from_json(config_path.to_string_lossy().into_owned());

            let shared = Arc::new(std::sync::Mutex::new(initial_state));
            let shared_for_update = Arc::clone(&shared);

            // Per-run Tauri channel: set by run_processor, cleared on completion.
            let active_channel: Arc<std::sync::Mutex<Option<tauri::ipc::Channel<PermuteProgressEvent>>>> =
                Arc::new(std::sync::Mutex::new(None));
            let active_channel_for_thread = Arc::clone(&active_channel);

            // Spawn the update thread.
            // When a Tauri Channel is active (run_processor), send lightweight delta
            // events instead of serialising the full state on every tick.
            // When no channel is active (reverse/trim), fall back to global events.
            thread::Builder::new()
                .name("PermuteUpdateThread".into())
                .spawn(move || {
                    while let Ok(message) = permute_rx.recv() {
                        let is_complete = matches!(
                            message,
                            PermuteUpdate::ProcessComplete(_) | PermuteUpdate::Error(_)
                        );
                        let has_channel = active_channel_for_thread.lock().unwrap().is_some();

                        if has_channel {
                            // Update state, then build a lightweight delta event.
                            let progress_event = {
                                let mut s = shared_for_update.lock().unwrap();
                                update_state_from_message(&mut s, &message);
                                build_progress_event(&message)
                            };
                            if let Some(event) = progress_event {
                                let guard = active_channel_for_thread.lock().unwrap();
                                if let Some(channel) = guard.as_ref() {
                                    channel.send(event).ok();
                                }
                            }
                            if is_complete {
                                *active_channel_for_thread.lock().unwrap() = None;
                            }
                        } else {
                            // No active channel: update state and emit full DTO as
                            // global events (used by reverse_file / trim_file).
                            let dto = {
                                let mut s = shared_for_update.lock().unwrap();
                                update_state_from_message(&mut s, &message);
                                s.to_state_dto()
                            };
                            if is_complete {
                                let _ = app_handle.emit("permute-ended", dto);
                            } else {
                                let _ = app_handle.emit("permute-update", dto);
                            }
                        }
                    }
                })
                .expect("failed to spawn update thread");

            app.manage(AppState { shared, active_channel });

            // Store config path for the on-close handler
            app.manage(ConfigPath(config_path.to_string_lossy().into_owned()));

            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::Destroyed = event {
                let app = window.app_handle();
                if let (Some(state), Some(cfg)) = (
                    app.try_state::<AppState>().map(|s| s.inner().shared.lock().unwrap().clone()),
                    app.try_state::<ConfigPath>().map(|c| c.0.clone()),
                ) {
                    let mut s = state;
                    let _ = s.write_to_json(cfg);
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            get_state,
            run_processor,
            reverse_file,
            trim_file,
            cancel,
            add_file,
            remove_file,
            clear_all_files,
            delete_output_file,
            delete_all_output_files,
            add_processor,
            remove_processor,
            select_all_processors,
            deselect_all_processors,
            set_output,
            set_depth,
            set_permutations,
            set_normalised,
            set_trim_all,
            set_input_trail,
            set_output_trail,
            set_max_stretch,
            set_create_subdirectories,
            set_viewed_welcome,
            open_output_dialog,
            save_scene,
            load_scene,
            show_in_folder,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// Managed type to pass the config path to the window-close handler
struct ConfigPath(String);
