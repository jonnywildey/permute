use std::{sync::Arc, thread};

use tauri::{AppHandle, Emitter, State};

use crate::state::{AppState, PermuteStateDto};

// ─── State query ─────────────────────────────────────────────────────────────

#[tauri::command]
pub fn get_state(state: State<'_, AppState>) -> PermuteStateDto {
    state.shared.lock().unwrap().to_state_dto()
}

// ─── Processing commands ─────────────────────────────────────────────────────

#[tauri::command]
pub fn run_processor(state: State<'_, AppState>) -> Result<(), String> {
    let mut s = state.shared.lock().unwrap();
    if !s.processing {
        s.run_process();
    }
    Ok(())
}

#[tauri::command]
pub fn reverse_file(
    state: State<'_, AppState>,
    app_handle: AppHandle,
    file: String,
) -> Result<(), String> {
    let shared = Arc::clone(&state.shared);
    let app = app_handle.clone();
    thread::spawn(move || {
        let result = {
            let mut s = shared.lock().unwrap();
            s.reverse_file(file)
        };
        if let Err(e) = result {
            let mut s = shared.lock().unwrap();
            s.set_error(e.to_string());
        }
        let dto = shared.lock().unwrap().to_state_dto();
        let _ = app.emit("permute-ended", dto);
    });
    Ok(())
}

#[tauri::command]
pub fn trim_file(
    state: State<'_, AppState>,
    app_handle: AppHandle,
    file: String,
) -> Result<(), String> {
    let shared = Arc::clone(&state.shared);
    let app = app_handle.clone();
    thread::spawn(move || {
        let result = {
            let mut s = shared.lock().unwrap();
            s.trim_file(file)
        };
        if let Err(e) = result {
            let mut s = shared.lock().unwrap();
            s.set_error(e.to_string());
        }
        let dto = shared.lock().unwrap().to_state_dto();
        let _ = app.emit("permute-ended", dto);
    });
    Ok(())
}

#[tauri::command]
pub fn cancel(state: State<'_, AppState>) {
    state.shared.lock().unwrap().cancel();
}

// ─── File management ─────────────────────────────────────────────────────────

#[tauri::command]
pub fn add_file(state: State<'_, AppState>, file: String) -> Result<(), String> {
    state
        .shared
        .lock()
        .unwrap()
        .add_file(file)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn remove_file(state: State<'_, AppState>, file: String) {
    state.shared.lock().unwrap().remove_file(file);
}

#[tauri::command]
pub fn clear_all_files(state: State<'_, AppState>) {
    state.shared.lock().unwrap().clear_all_files();
}

#[tauri::command]
pub fn delete_output_file(state: State<'_, AppState>, file: String) -> Result<(), String> {
    state
        .shared
        .lock()
        .unwrap()
        .delete_output_file(file)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_all_output_files(state: State<'_, AppState>) -> Result<(), String> {
    state
        .shared
        .lock()
        .unwrap()
        .delete_all_output_files()
        .map_err(|e| e.to_string())
}

// ─── Processor pool ───────────────────────────────────────────────────────────

#[tauri::command]
pub fn add_processor(state: State<'_, AppState>, name: String) {
    state.shared.lock().unwrap().add_processor(name);
}

#[tauri::command]
pub fn remove_processor(state: State<'_, AppState>, name: String) {
    state.shared.lock().unwrap().remove_processor(name);
}

#[tauri::command]
pub fn select_all_processors(state: State<'_, AppState>) {
    state.shared.lock().unwrap().select_all_processors();
}

#[tauri::command]
pub fn deselect_all_processors(state: State<'_, AppState>) {
    state.shared.lock().unwrap().deselect_all_processors();
}

// ─── Configuration setters ────────────────────────────────────────────────────

#[tauri::command]
pub fn set_output(state: State<'_, AppState>, output: String) {
    state.shared.lock().unwrap().set_output(output);
}

#[tauri::command]
pub fn set_depth(state: State<'_, AppState>, depth: usize) {
    state.shared.lock().unwrap().set_depth(depth);
}

#[tauri::command]
pub fn set_permutations(state: State<'_, AppState>, permutations: usize) {
    state.shared.lock().unwrap().set_permutations(permutations);
}

#[tauri::command]
pub fn set_normalised(state: State<'_, AppState>, normalised: bool) {
    state.shared.lock().unwrap().set_normalised(normalised);
}

#[tauri::command]
pub fn set_trim_all(state: State<'_, AppState>, trim_all: bool) {
    state.shared.lock().unwrap().set_trim_all(trim_all);
}

#[tauri::command]
pub fn set_input_trail(state: State<'_, AppState>, trail: f64) {
    state.shared.lock().unwrap().set_input_trail(trail);
}

#[tauri::command]
pub fn set_output_trail(state: State<'_, AppState>, trail: f64) {
    state.shared.lock().unwrap().set_output_trail(trail);
}

#[tauri::command]
pub fn set_max_stretch(state: State<'_, AppState>, max_stretch: f64) {
    state.shared.lock().unwrap().set_max_stretch(max_stretch);
}

#[tauri::command]
pub fn set_create_subdirectories(state: State<'_, AppState>, create: bool) {
    state
        .shared
        .lock()
        .unwrap()
        .set_create_subdirectories(create);
}

#[tauri::command]
pub fn set_viewed_welcome(state: State<'_, AppState>, viewed: bool) {
    state.shared.lock().unwrap().set_viewed_welcome(viewed);
}

// ─── Dialogs ─────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn open_output_dialog(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;
    let path = app.dialog().file().blocking_pick_folder();
    if let Some(p) = path {
        let path_str = p.to_string();
        state.shared.lock().unwrap().set_output(path_str.clone());
        Ok(Some(path_str))
    } else {
        Ok(None)
    }
}

#[tauri::command]
pub async fn save_scene(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;
    let path = app
        .dialog()
        .file()
        .add_filter("Scene Files", &["json"])
        .set_file_name("scene.json")
        .blocking_save_file();
    if let Some(p) = path {
        let path_str = p.to_string();
        state
            .shared
            .lock()
            .unwrap()
            .write_to_json(path_str.clone())
            .map_err(|e| e.to_string())?;
        Ok(Some(path_str))
    } else {
        Ok(None)
    }
}

#[tauri::command]
pub async fn load_scene(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    use tauri_plugin_dialog::DialogExt;
    let path = app
        .dialog()
        .file()
        .add_filter("Scene Files", &["json"])
        .blocking_pick_file();
    if let Some(p) = path {
        let path_str = p.to_string();
        match state.shared.lock().unwrap().read_from_json(path_str) {
            Ok(()) => Ok(serde_json::json!({ "success": true })),
            Err(e) => Ok(serde_json::json!({ "success": false, "error": e.to_string() })),
        }
    } else {
        Ok(serde_json::json!({ "success": false, "error": "cancelled" }))
    }
}

// ─── Shell ───────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn show_in_folder(_app: AppHandle, file: String) -> Result<(), String> {
    // `open -R` reveals the file in Finder (macOS only)
    std::process::Command::new("open")
        .args(["-R", &file])
        .spawn()
        .map(|_| ())
        .map_err(|e| e.to_string())
}
