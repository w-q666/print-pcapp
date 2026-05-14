use std::fs;
use tauri::AppHandle;
use crate::db::AppState;
use crate::entities::PrintJob;
use crate::repos::PrintJobRepo;

// ── PrintJob commands ──

#[tauri::command]
pub fn print_jobs_list(state: tauri::State<'_, AppState>) -> Result<Vec<PrintJob>, String> {
    PrintJobRepo::list(state.db())
}

#[tauri::command]
pub fn print_jobs_create(
    state: tauri::State<'_, AppState>,
    name: String,
) -> Result<PrintJob, String> {
    PrintJobRepo::create(state.db(), &name)
}

#[tauri::command]
pub fn print_jobs_get_by_id(
    state: tauri::State<'_, AppState>,
    id: i64,
) -> Result<PrintJob, String> {
    PrintJobRepo::get_by_id(state.db(), id)
}

#[tauri::command]
pub fn print_jobs_delete(
    state: tauri::State<'_, AppState>,
    id: i64,
) -> Result<(), String> {
    PrintJobRepo::delete(state.db(), id)
}

// ── File commands ──

#[tauri::command]
pub fn file_save(
    app_handle: AppHandle,
    name: String,
    bytes: Vec<u8>,
) -> Result<String, String> {
    let data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;
    let files_dir = data_dir.join("files");
    fs::create_dir_all(&files_dir).map_err(|e| e.to_string())?;
    let path = files_dir.join(&name);
    fs::write(&path, &bytes).map_err(|e| e.to_string())?;
    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
pub fn file_read(app_handle: AppHandle, name: String) -> Result<String, String> {
    let data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;
    let path = data_dir.join("files").join(&name);
    let bytes = fs::read(&path).map_err(|e| e.to_string())?;
    use base64::Engine;
    Ok(base64::engine::general_purpose::STANDARD.encode(&bytes))
}

#[tauri::command]
pub fn file_delete(app_handle: AppHandle, name: String) -> Result<(), String> {
    let data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;
    let path = data_dir.join("files").join(&name);
    fs::remove_file(&path).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn file_list(app_handle: AppHandle) -> Result<Vec<String>, String> {
    let data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;
    let files_dir = data_dir.join("files");
    if !files_dir.exists() {
        return Ok(Vec::new());
    }
    let mut names = Vec::new();
    let entries = fs::read_dir(&files_dir).map_err(|e| e.to_string())?;
    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        if entry.file_type().map_err(|e| e.to_string())?.is_file() {
            names.push(entry.file_name().to_string_lossy().to_string());
        }
    }
    Ok(names)
}
