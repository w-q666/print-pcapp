use std::fs;
use std::path::Path;
use tauri::AppHandle;
use tauri::Manager;
use crate::db::AppState;
use crate::entities::{CreatePrintJobRequest, PrintJob, SystemLog};
use crate::repos::{PrintJobRepo, SystemLogRepo};

fn safe_filename(name: &str) -> Result<String, String> {
    Path::new(name)
        .file_name()
        .and_then(|f| f.to_str())
        .map(|f| f.to_string())
        .ok_or_else(|| format!("Invalid file name: {}", name))
}

// ── PrintJob commands ──

#[tauri::command]
pub fn print_jobs_list(
    state: tauri::State<'_, AppState>,
    limit: Option<i64>,
) -> Result<Vec<PrintJob>, String> {
    PrintJobRepo::list(state.db(), limit)
}

#[tauri::command]
pub fn print_jobs_create(
    state: tauri::State<'_, AppState>,
    req: CreatePrintJobRequest,
) -> Result<PrintJob, String> {
    PrintJobRepo::create(state.db(), &req)
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

#[tauri::command]
pub fn print_jobs_update_status(
    state: tauri::State<'_, AppState>,
    id: i64,
    status: String,
    error_msg: Option<String>,
) -> Result<(), String> {
    PrintJobRepo::update_status(state.db(), id, &status, error_msg.as_deref())
}

#[tauri::command]
pub fn print_jobs_count_queue(state: tauri::State<'_, AppState>) -> Result<i64, String> {
    PrintJobRepo::count_by_status(state.db(), "queued")
}

#[tauri::command]
pub fn print_jobs_count_today(state: tauri::State<'_, AppState>) -> Result<i64, String> {
    PrintJobRepo::count_today_completed(state.db())
}

// ── SystemLog commands ──

#[tauri::command]
pub fn log_insert(
    state: tauri::State<'_, AppState>,
    level: String,
    category: String,
    message: String,
) -> Result<(), String> {
    SystemLogRepo::insert(state.db(), &level, &category, &message, "frontend")
}

#[tauri::command]
pub fn log_query(
    state: tauri::State<'_, AppState>,
    level: Option<String>,
    category: Option<String>,
    keyword: Option<String>,
    limit: Option<i64>,
) -> Result<Vec<SystemLog>, String> {
    use crate::entities::LogQuery;
    SystemLogRepo::query(
        state.db(),
        &LogQuery { level, category, keyword, limit },
    )
}

#[tauri::command]
pub fn log_clear(state: tauri::State<'_, AppState>) -> Result<(), String> {
    SystemLogRepo::clear(state.db())
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
    let safe_name = safe_filename(&name)?;
    let path = files_dir.join(&safe_name);
    fs::write(&path, &bytes).map_err(|e| e.to_string())?;
    Ok(safe_name)
}

#[tauri::command]
pub fn file_read(app_handle: AppHandle, name: String) -> Result<String, String> {
    let data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;
    let safe_name = safe_filename(&name)?;
    let path = data_dir.join("files").join(&safe_name);
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
    let safe_name = safe_filename(&name)?;
    let path = data_dir.join("files").join(&safe_name);
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
