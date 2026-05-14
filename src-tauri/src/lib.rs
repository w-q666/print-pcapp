mod commands;
mod db;
mod entities;
mod repos;

use db::{init_db, AppState};
use tauri::Manager;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let conn = init_db(app.handle())?;
            app.manage(AppState {
                db: std::sync::Mutex::new(conn),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            commands::print_jobs_list,
            commands::print_jobs_create,
            commands::print_jobs_get_by_id,
            commands::print_jobs_delete,
            commands::file_save,
            commands::file_read,
            commands::file_delete,
            commands::file_list,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
