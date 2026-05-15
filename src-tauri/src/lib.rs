mod commands;
mod db;
mod discovery;
mod entities;
mod http_server;
pub mod logger;
mod network;
mod print_queue;
mod qr;
mod repos;

use db::{init_db, AppState};
use tauri::Manager;
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::menu::{Menu, MenuItem};
use tauri::image::Image;
use tauri::WindowEvent;

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

            let app_state = app.state::<AppState>();
            logger::log_info(&app_state, "system", "rust:lib::setup", "SQLite 数据库初始化完成");

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
            logger::log_info(&app_state, "system", "rust:lib::setup", "LAN 认证 token 已生成");

            let allowed_extensions = vec![
                "pdf",
                "jpg", "jpeg", "png", "gif", "bmp", "tiff", "webp",
                "txt", "htm", "html",
            ].into_iter().map(String::from).collect();

            let db_for_http = app_state.db.lock().map_err(|e| e.to_string()).ok().and_then(|_| {
                // Clone the db path for http_server to open its own connection
                Some(db_path.clone())
            });

            let state = http_server::HttpState::new(
                token,
                allowed_extensions,
                50 * 1024 * 1024, // 50MB
                files_dir,
                db_for_http.unwrap_or(db_path.clone()),
            );

            let port: u16 = 5000;
            logger::log_info(
                &app_state,
                "system",
                "rust:lib::setup",
                &format!("正在启动 LAN HTTP 服务 (端口 {})", port),
            );

            tauri::async_runtime::spawn(async move {
                if let Err(e) = http_server::start_server(state, port).await {
                    eprintln!("[HTTP] LAN 服务启动失败: {}", e);
                }
            });

            // 初始化打印队列
            {
                let queue_conn = {
                    let conn = app_state.db.lock().map_err(|e| e.to_string())?;
                    print_queue::init_queue_table(&conn)?;

                    let recovered = print_queue::recover_stale_tasks(&conn)?;
                    if recovered > 0 {
                        logger::log_warn(
                            &app_state, "system", "rust:lib::setup",
                            &format!("恢复了 {} 个崩溃遗留的打印任务", recovered),
                        );
                    }

                    drop(conn);

                    let db_path = data_dir.join("app.db");
                    let c = rusqlite::Connection::open(&db_path)
                        .map_err(|e| format!("打开队列数据库连接失败: {}", e))?;
                    c
                };

                let queue_state = print_queue::QueueState {
                    db: std::sync::Arc::new(std::sync::Mutex::new(queue_conn)),
                };
                app.manage(queue_state);
                print_queue::spawn_worker(app.handle());
                logger::log_info(&app_state, "system", "rust:lib::setup", "打印队列初始化完成");
            }

            // 创建系统托盘
            let show_item = MenuItem::with_id(app, "show", "显示主窗口", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
            let tray_menu = Menu::with_items(app, &[&show_item, &quit_item])?;

            let tray_icon = Image::from_bytes(include_bytes!("../icons/tray-icon.png"))
                .expect("无法加载托盘图标");

            TrayIconBuilder::new()
                .icon(tray_icon)
                .tooltip("网络打印服务")
                .menu(&tray_menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.unminimize();
                            let _ = window.set_focus();
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.unminimize();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            logger::log_info(&app_state, "system", "rust:lib::setup", "系统托盘已创建");
            logger::log_info(&app_state, "system", "rust:lib::setup", "应用启动完成");

            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
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
            commands::log_export_csv,
            commands::file_save,
            commands::file_read,
            commands::file_delete,
            commands::file_list,
            commands::lan_server_url,
            commands::lan_server_qrcode,
            commands::discover_service,
            commands::get_network_local_ip,
            commands::print_queue_submit,
            commands::print_queue_pending_count,
            commands::print_queue_list,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
