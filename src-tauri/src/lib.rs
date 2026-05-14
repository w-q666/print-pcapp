mod commands;
mod db;
mod entities;
mod http_server;
pub mod logger;
mod network;
mod qr;
mod repos;

use db::{init_db, AppState};
use tauri::Manager;

/// 存储 LAN 服务器的 token，供 commands 读取
pub struct LanServerToken(pub String);

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    std::panic::set_hook(Box::new(|info| {
        eprintln!("[PANIC] {}", info);
    }));

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let conn = init_db(app.handle())?;
            app.manage(AppState {
                db: std::sync::Mutex::new(conn),
            });

            // 启动 LAN HTTP 上传服务器
            let data_dir = app.path().app_data_dir().expect("无法获取 app_data_dir");
            let files_dir = data_dir.join("files");
            let db_path = data_dir.join("print.db");
            std::fs::create_dir_all(&files_dir).ok();

            let token: String = {
                use rand::Rng;
                let mut rng = rand::thread_rng();
                (0..16).map(|_| format!("{:02x}", rng.gen::<u8>())).collect()
            };
            app.manage(LanServerToken(token.clone()));

            let allowed_extensions = vec![
                "pdf",
                "jpg", "jpeg", "png", "gif", "bmp", "tiff", "webp",
                "txt", "htm", "html",
            ].into_iter().map(String::from).collect();

            let state = http_server::HttpState::new(
                token,
                allowed_extensions,
                50 * 1024 * 1024, // 50MB
                files_dir,
                db_path,
            );

            let port: u16 = 5000;
            tauri::async_runtime::spawn(async move {
                if let Err(e) = http_server::start_server(state, port).await {
                    eprintln!("[HTTP] LAN 服务启动失败: {}", e);
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            commands::print_jobs_list,
            commands::print_jobs_create,
            commands::print_jobs_get_by_id,
            commands::print_jobs_delete,
            commands::print_jobs_update_status,
            commands::print_jobs_count_queue,
            commands::print_jobs_count_today,
            commands::log_insert,
            commands::log_query,
            commands::log_clear,
            commands::file_save,
            commands::file_read,
            commands::file_delete,
            commands::file_list,
            commands::lan_server_url,
            commands::lan_server_qrcode,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
