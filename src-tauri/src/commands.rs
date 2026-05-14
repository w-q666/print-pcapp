use std::fs;
use std::path::Path;
use tauri::AppHandle;
use tauri::Manager;
use crate::db::AppState;
use crate::entities::{CreatePrintJobRequest, PrintJob, SystemLog};
use crate::logger;
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
    let result = PrintJobRepo::create(state.db(), &req);
    match &result {
        Ok(job) => logger::log_info(
            &state, "print", "rust:commands::print_jobs_create",
            &format!("创建打印任务: {} (ID: {}, 打印机: {})", job.name, job.id, job.printer),
        ),
        Err(e) => logger::log_error(
            &state, "print", "rust:commands::print_jobs_create",
            &format!("创建打印任务失败: {} - {}", req.name, e),
        ),
    }
    result
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
    let result = PrintJobRepo::delete(state.db(), id);
    if result.is_ok() {
        logger::log_info(&state, "print", "rust:commands::print_jobs_delete",
            &format!("删除打印任务 ID: {}", id));
    }
    result
}

#[tauri::command]
pub fn print_jobs_update_status(
    state: tauri::State<'_, AppState>,
    id: i64,
    status: String,
    error_msg: Option<String>,
) -> Result<(), String> {
    let result = PrintJobRepo::update_status(state.db(), id, &status, error_msg.as_deref());
    if result.is_ok() {
        let msg = match &error_msg {
            Some(err) => format!("打印任务状态更新: ID {} → {} ({})", id, status, err),
            None => format!("打印任务状态更新: ID {} → {}", id, status),
        };
        let level = if status == "failed" || status == "cancelled" { "WARN" } else { "INFO" };
        match level {
            "WARN" => logger::log_warn(&state, "print", "rust:commands::print_jobs_update_status", &msg),
            _ => logger::log_info(&state, "print", "rust:commands::print_jobs_update_status", &msg),
        }
    }
    result
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
    logger: String,
) -> Result<(), String> {
    SystemLogRepo::insert(state.db(), &level, &category, &message, &logger)
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

#[tauri::command]
pub fn log_export_csv(
    state: tauri::State<'_, AppState>,
    path: String,
    level: Option<String>,
    category: Option<String>,
    keyword: Option<String>,
) -> Result<u64, String> {
    use crate::entities::LogQuery;
    use std::io::Write;

    let logs = SystemLogRepo::query_all(
        state.db(),
        &LogQuery { level, category, keyword, limit: None },
    )?;

    let mut file = std::fs::File::create(&path)
        .map_err(|e| format!("创建文件失败: {}", e))?;

    // UTF-8 BOM for Excel compatibility
    file.write_all(b"\xEF\xBB\xBF")
        .map_err(|e| format!("写入 BOM 失败: {}", e))?;

    file.write_all(b"\xE6\x97\xB6\xE9\x97\xB4,\xE7\xBA\xA7\xE5\x88\xAB,\xE5\x88\x86\xE7\xB1\xBB,\xE6\x9D\xA5\xE6\xBA\x90,\xE5\x86\x85\xE5\xAE\xB9\r\n")
        .map_err(|e| format!("写入标题行失败: {}", e))?;

    let count = logs.len() as u64;
    for log in &logs {
        let line = format!(
            "\"{}\",\"{}\",\"{}\",\"{}\",\"{}\"\r\n",
            csv_escape(&log.timestamp),
            csv_escape(&log.level),
            csv_escape(&log.category),
            csv_escape(&log.logger),
            csv_escape(&log.message),
        );
        file.write_all(line.as_bytes())
            .map_err(|e| format!("写入数据行失败: {}", e))?;
    }

    Ok(count)
}

fn csv_escape(s: &str) -> String {
    s.replace('"', "\"\"")
}

// ── LAN server commands ──

#[tauri::command]
pub fn lan_server_url(
    lan_token: tauri::State<'_, crate::LanServerToken>,
    port: Option<u16>,
) -> Result<String, String> {
    let ip = crate::network::get_local_ip()?;
    let p = port.unwrap_or(5000);
    Ok(format!("http://{}:{}?token={}", ip, p, lan_token.0))
}

#[tauri::command]
pub fn lan_server_qrcode(
    lan_token: tauri::State<'_, crate::LanServerToken>,
    port: Option<u16>,
) -> Result<String, String> {
    let url = lan_server_url(lan_token, port)?;
    crate::qr::generate_qr_base64(&url)
}

// ── File commands ──

#[tauri::command]
pub fn file_save(
    app_handle: AppHandle,
    state: tauri::State<'_, AppState>,
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
    let size = bytes.len();
    let path = files_dir.join(&safe_name);
    match fs::write(&path, &bytes) {
        Ok(_) => {
            logger::log_info(&state, "file", "rust:commands::file_save",
                &format!("文件保存成功: {} ({:.2} KB)", safe_name, size as f64 / 1024.0));
            Ok(safe_name)
        }
        Err(e) => {
            logger::log_error(&state, "file", "rust:commands::file_save",
                &format!("文件保存失败: {} - {}", safe_name, e));
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub fn file_read(
    app_handle: AppHandle,
    state: tauri::State<'_, AppState>,
    name: String,
) -> Result<String, String> {
    let data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;
    let safe_name = safe_filename(&name)?;
    let path = data_dir.join("files").join(&safe_name);
    match fs::read(&path) {
        Ok(bytes) => {
            logger::log_debug(&state, "file", "rust:commands::file_read",
                &format!("文件读取成功: {} ({:.2} KB)", safe_name, bytes.len() as f64 / 1024.0));
            use base64::Engine;
            Ok(base64::engine::general_purpose::STANDARD.encode(&bytes))
        }
        Err(e) => {
            logger::log_error(&state, "file", "rust:commands::file_read",
                &format!("文件读取失败: {} - {}", safe_name, e));
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub fn file_delete(
    app_handle: AppHandle,
    state: tauri::State<'_, AppState>,
    name: String,
) -> Result<(), String> {
    let data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;
    let safe_name = safe_filename(&name)?;
    let path = data_dir.join("files").join(&safe_name);
    match fs::remove_file(&path) {
        Ok(_) => {
            logger::log_info(&state, "file", "rust:commands::file_delete",
                &format!("文件删除成功: {}", safe_name));
            Ok(())
        }
        Err(e) => {
            logger::log_error(&state, "file", "rust:commands::file_delete",
                &format!("文件删除失败: {} - {}", safe_name, e));
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub fn file_list(
    app_handle: AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<Vec<crate::entities::FileInfo>, String> {
    let data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;
    let files_dir = data_dir.join("files");
    if !files_dir.exists() {
        return Ok(Vec::new());
    }
    let mut files = Vec::new();
    let entries = fs::read_dir(&files_dir).map_err(|e| e.to_string())?;
    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        if entry.file_type().map_err(|e| e.to_string())?.is_file() {
            let name = entry.file_name().to_string_lossy().to_string();
            let meta = entry.metadata().map_err(|e| e.to_string())?;
            let size = meta.len();
            let modified_at = meta
                .modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs())
                .unwrap_or(0);
            files.push(crate::entities::FileInfo {
                name,
                size,
                modified_at,
            });
        }
    }
    logger::log_debug(&state, "file", "rust:commands::file_list",
        &format!("文件列表查询完成，共 {} 个文件", files.len()));
    Ok(files)
}
