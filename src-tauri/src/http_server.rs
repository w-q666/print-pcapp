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
    /// 数据库路径（预留，当前未使用）
    pub db_path: PathBuf,
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
            // 验证 token（可能在 file field 之前或之后收到 token field）
            if let Some(ref t) = token_value {
                if t != &state.inner.token {
                    error = Some("无效的访问令牌".to_string());
                    break;
                }
            }

            let file_name = match field.file_name() {
                Some(name) => name.to_string(),
                None => {
                    error = Some("未提供文件名".to_string());
                    break;
                }
            };

            // 校验文件扩展名
            let ext = file_name
                .rsplit('.')
                .next()
                .unwrap_or("")
                .to_lowercase();

            if !state.inner.allowed_extensions.is_empty()
                && !state.inner.allowed_extensions.contains(&ext)
            {
                error = Some(format!("不支持的文件格式: .{}", ext));
                break;
            }

            // 读取文件数据
            let data = match field.bytes().await {
                Ok(bytes) => bytes,
                Err(e) => {
                    error = Some(format!("读取文件数据失败: {}", e));
                    break;
                }
            };

            // 校验文件大小
            if data.len() as u64 > state.inner.max_file_size {
                let max_mb = state.inner.max_file_size / (1024 * 1024);
                error = Some(format!("文件过大，最大允许 {}MB", max_mb));
                break;
            }

            // 生成唯一文件名避免冲突
            let unique_name = format!(
                "{}_{}",
                chrono_free_timestamp(),
                sanitize_filename(&file_name)
            );
            let dest = state.inner.files_dir.join(&unique_name);

            // 确保目录存在
            if let Err(e) = std::fs::create_dir_all(&state.inner.files_dir) {
                error = Some(format!("创建存储目录失败: {}", e));
                break;
            }

            // 写入文件
            if let Err(e) = std::fs::write(&dest, &data) {
                error = Some(format!("保存文件失败: {}", e));
                break;
            }

            // TODO: 写入 print_jobs 表（source="mobile"）
            // 当 DB 模块稳定后，在此处调用类似：
            //   db::insert_print_job(&state.inner.db_path, &unique_name, "mobile")?;

            saved_file = Some(unique_name);
        }
    }

    // 延迟 token 验证：如果 token field 在 file field 之后出现
    if error.is_none() {
        match &token_value {
            None => {
                return Json(serde_json::json!({
                    "code": 401,
                    "msg": "缺少访问令牌"
                }));
            }
            Some(t) if t != &state.inner.token => {
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
        None => Json(serde_json::json!({
            "code": 400,
            "msg": "未收到文件"
        })),
    }
}

/// 启动 HTTP 服务，绑定到 0.0.0.0:{port}
pub async fn start_server(state: HttpState, port: u16) -> Result<(), String> {
    let addr = format!("0.0.0.0:{}", port);
    let router = create_router(state);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .map_err(|e| format!("绑定端口 {} 失败: {}", port, e))?;
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
