#![allow(dead_code)]

use axum::{
    extract::{DefaultBodyLimit, Multipart, State},
    response::Html,
    routing::{get, post},
    Json, Router,
};
use std::path::PathBuf;
use std::sync::Arc;
use tower_http::cors::CorsLayer;

/// HTTP 服务共享状态
#[derive(Clone)]
pub struct HttpState {
    inner: Arc<HttpStateInner>,
}

struct HttpStateInner {
    pub token: String,
    pub allowed_extensions: Vec<String>,
    /// 单文件最大字节数
    pub max_file_size: u64,
    /// 上传文件存储目录
    pub files_dir: PathBuf,
    /// 数据库路径，用于写入日志
    pub db_path: PathBuf,
}

fn http_log(db_path: &PathBuf, level: &str, source: &str, message: &str) {
    if let Ok(conn) = rusqlite::Connection::open(db_path) {
        let _ = conn.execute(
            "INSERT INTO system_logs (level, category, message, logger) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![level, "upload", message, source],
        );
    }
}

impl HttpState {
    pub fn new(
        token: String,
        allowed_extensions: Vec<String>,
        max_file_size: u64,
        files_dir: PathBuf,
        db_path: PathBuf,
    ) -> Self {
        Self {
            inner: Arc::new(HttpStateInner {
                token,
                allowed_extensions,
                max_file_size,
                files_dir,
                db_path,
            }),
        }
    }
}

/// 构建 axum 路由
pub fn create_router(state: HttpState) -> Router {
    Router::new()
        .route("/", get(index_handler))
        .route("/upload", post(upload_handler))
        .route("/api/health", get(health_handler))
        .layer(CorsLayer::permissive())
        .layer(DefaultBodyLimit::max(100 * 1024 * 1024)) // 100MB body limit
        .with_state(state)
}

/// GET / — 返回移动端上传页面
async fn index_handler() -> Html<&'static str> {
    Html(include_str!("../static/mobile-upload.html"))
}

/// GET /api/health — 健康检查
async fn health_handler() -> Json<serde_json::Value> {
    Json(serde_json::json!({"code": 0, "msg": "ok"}))
}

/// POST /upload — 接收 multipart 文件上传
async fn upload_handler(
    State(state): State<HttpState>,
    mut multipart: Multipart,
) -> Json<serde_json::Value> {
    let log = |level: &str, msg: &str| {
        http_log(&state.inner.db_path, level, "rust:http_server::handle_upload", msg);
    };

    let mut token_value: Option<String> = None;
    let mut saved_file: Option<String> = None;
    let mut error: Option<String> = None;

    while let Ok(Some(field)) = multipart.next_field().await {
        let field_name = field.name().unwrap_or_default().to_string();

        if field_name == "token" {
            if let Ok(val) = field.text().await {
                token_value = Some(val);
            }
            continue;
        }

        if field_name == "file" {
            if let Some(ref t) = token_value {
                if t != &state.inner.token {
                    log("WARN", "token 校验失败：无效的访问令牌");
                    error = Some("无效的访问令牌".to_string());
                    break;
                }
            }

            let file_name = match field.file_name() {
                Some(name) => name.to_string(),
                None => {
                    log("WARN", "上传请求未提供文件名");
                    error = Some("未提供文件名".to_string());
                    break;
                }
            };

            let ext = file_name
                .rsplit('.')
                .next()
                .unwrap_or("")
                .to_lowercase();

            if !state.inner.allowed_extensions.is_empty()
                && !state.inner.allowed_extensions.contains(&ext)
            {
                log("WARN", &format!("扩展名被拒绝: .{} (文件: {})", ext, file_name));
                error = Some(format!("不支持的文件格式: .{}", ext));
                break;
            }

            let data = match field.bytes().await {
                Ok(bytes) => bytes,
                Err(e) => {
                    log("ERROR", &format!("读取文件数据失败: {} (文件: {})", e, file_name));
                    error = Some(format!("读取文件数据失败: {}", e));
                    break;
                }
            };

            let file_size = data.len() as u64;
            log("INFO", &format!(
                "收到上传请求: {} ({:.2} KB)",
                file_name, file_size as f64 / 1024.0
            ));

            if file_size > state.inner.max_file_size {
                let max_mb = state.inner.max_file_size / (1024 * 1024);
                log("WARN", &format!(
                    "文件大小超限: {:.2} MB > {} MB (文件: {})",
                    file_size as f64 / (1024.0 * 1024.0), max_mb, file_name
                ));
                error = Some(format!("文件过大，最大允许 {}MB", max_mb));
                break;
            }

            let unique_name = format!(
                "{}_{}",
                chrono_free_timestamp(),
                sanitize_filename(&file_name)
            );
            let dest = state.inner.files_dir.join(&unique_name);

            if let Err(e) = std::fs::create_dir_all(&state.inner.files_dir) {
                log("ERROR", &format!("创建存储目录失败: {}", e));
                error = Some(format!("创建存储目录失败: {}", e));
                break;
            }

            if let Err(e) = std::fs::write(&dest, &data) {
                log("ERROR", &format!("保存文件失败: {} (文件: {})", e, file_name));
                error = Some(format!("保存文件失败: {}", e));
                break;
            }

            log("INFO", &format!(
                "文件上传成功: {} → {} ({:.2} KB)",
                file_name, unique_name, file_size as f64 / 1024.0
            ));

            saved_file = Some(unique_name);
        }
    }

    if error.is_none() {
        match &token_value {
            None => {
                log("WARN", "上传请求缺少访问令牌");
                return Json(serde_json::json!({
                    "code": 401,
                    "msg": "缺少访问令牌"
                }));
            }
            Some(t) if t != &state.inner.token => {
                log("WARN", "token 校验失败：无效的访问令牌");
                return Json(serde_json::json!({
                    "code": 403,
                    "msg": "无效的访问令牌"
                }));
            }
            _ => {}
        }
    }

    if let Some(err_msg) = error {
        return Json(serde_json::json!({
            "code": 400,
            "msg": err_msg
        }));
    }

    match saved_file {
        Some(name) => Json(serde_json::json!({
            "code": 0,
            "msg": "上传成功",
            "data": { "filename": name }
        })),
        None => {
            log("WARN", "上传请求中未收到文件");
            Json(serde_json::json!({
                "code": 400,
                "msg": "未收到文件"
            }))
        }
    }
}

/// 启动 HTTP 服务，绑定到 0.0.0.0:{port}
pub async fn start_server(state: HttpState, port: u16) -> Result<(), String> {
    let addr = format!("0.0.0.0:{}", port);
    let db_path = state.inner.db_path.clone();
    let router = create_router(state);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .map_err(|e| {
            let msg = format!("绑定端口 {} 失败: {}", port, e);
            http_log(&db_path, "ERROR", "rust:http_server::start", &msg);
            msg
        })?;
    http_log(&db_path, "INFO", "rust:http_server::start",
        &format!("LAN 上传服务已启动: http://{}", addr));
    println!("[HTTP] LAN 上传服务已启动: http://{}", addr);
    axum::serve(listener, router)
        .await
        .map_err(|e| format!("HTTP 服务运行错误: {}", e))?;
    Ok(())
}

/// 生成不依赖 chrono 的时间戳（毫秒级），用于文件去重
fn chrono_free_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

/// 清理文件名中的不安全字符
fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            _ => c,
        })
        .collect()
}
