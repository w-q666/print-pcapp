use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};
use futures_util::StreamExt;

use crate::db::AppState;
use crate::repos::PrintJobRepo;

const MAX_ATTEMPTS: i32 = 3;
const POLL_INTERVAL_MS: u64 = 1000;
const RETRY_BACKOFF_MS: u64 = 3000;
const WS_TIMEOUT_SECS: u64 = 60;

// ── Data structures ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrintTask {
    pub job_id: i64,
    pub file_name: String,
    pub print_type: String,
    pub printer: String,
    pub copies: i32,
    pub color: bool,
    pub paper_size: String,
    pub direction: String,
    pub service_url: String,
    /// 提交任务时的隐藏列表快照，用于 `__auto__` 在可见打印机中随机
    #[serde(default)]
    pub printer_blacklist: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrintJobEvent {
    pub job_id: i64,
    pub file_name: String,
    pub status: String,
    pub error_msg: Option<String>,
}

pub struct QueueState {
    pub db: Arc<Mutex<Connection>>,
}

// ── Queue table migration ──

pub fn init_queue_table(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS print_queue (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            job_id INTEGER NOT NULL,
            payload TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'pending',
            attempts INTEGER NOT NULL DEFAULT 0,
            max_attempts INTEGER NOT NULL DEFAULT 3,
            error_msg TEXT DEFAULT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        );
        CREATE INDEX IF NOT EXISTS idx_queue_status ON print_queue(status);",
    )
    .map_err(|e| format!("创建队列表失败: {}", e))
}

/// 启动时将上次崩溃遗留的 `running` 任务重置为 `pending`，使其可以被重新消费
pub fn recover_stale_tasks(conn: &Connection) -> Result<u64, String> {
    let count = conn
        .execute(
            "UPDATE print_queue SET status = 'pending', updated_at = datetime('now')
             WHERE status = 'running'",
            [],
        )
        .map_err(|e| format!("恢复孤儿任务失败: {}", e))?;
    Ok(count as u64)
}

// ── Queue operations ──

pub fn push_task(db: &Mutex<Connection>, task: &PrintTask) -> Result<i64, String> {
    let payload = serde_json::to_string(task).map_err(|e| format!("序列化失败: {}", e))?;
    let conn = db.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO print_queue (job_id, payload, max_attempts) VALUES (?1, ?2, ?3)",
        rusqlite::params![task.job_id, payload, MAX_ATTEMPTS],
    )
    .map_err(|e| format!("入队失败: {}", e))?;
    Ok(conn.last_insert_rowid())
}

fn fetch_next(db: &Mutex<Connection>) -> Result<Option<(i64, PrintTask)>, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT id, payload FROM print_queue
             WHERE status = 'pending' AND attempts < max_attempts
             ORDER BY id ASC LIMIT 1",
        )
        .map_err(|e| e.to_string())?;

    let row = stmt
        .query_row([], |row| {
            let id: i64 = row.get(0)?;
            let payload: String = row.get(1)?;
            Ok((id, payload))
        })
        .ok();

    match row {
        Some((id, payload)) => {
            let task: PrintTask =
                serde_json::from_str(&payload).map_err(|e| format!("反序列化失败: {}", e))?;
            conn.execute(
                "UPDATE print_queue SET status = 'running', attempts = attempts + 1,
                 updated_at = datetime('now') WHERE id = ?1",
                rusqlite::params![id],
            )
            .map_err(|e| e.to_string())?;
            Ok(Some((id, task)))
        }
        None => Ok(None),
    }
}

fn mark_done(db: &Mutex<Connection>, queue_id: i64) -> Result<(), String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE print_queue SET status = 'done', updated_at = datetime('now') WHERE id = ?1",
        rusqlite::params![queue_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

/// Returns `true` if permanently failed (no retries left), `false` if will be retried.
fn mark_failed(db: &Mutex<Connection>, queue_id: i64, msg: &str) -> Result<bool, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;

    let (attempts, max): (i32, i32) = conn
        .query_row(
            "SELECT attempts, max_attempts FROM print_queue WHERE id = ?1",
            rusqlite::params![queue_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|e| e.to_string())?;

    let permanent = attempts >= max;
    let new_status = if permanent { "failed" } else { "pending" };
    conn.execute(
        "UPDATE print_queue SET status = ?1, error_msg = ?2, updated_at = datetime('now')
         WHERE id = ?3",
        rusqlite::params![new_status, msg, queue_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(permanent)
}

#[derive(Debug, Clone, Serialize)]
pub struct QueueItem {
    pub id: i64,
    pub job_id: i64,
    pub status: String,
    pub attempts: i32,
    pub max_attempts: i32,
    pub error_msg: Option<String>,
    pub created_at: String,
    pub file_name: String,
    pub printer: String,
}

pub fn list_queue(db: &Mutex<Connection>) -> Result<Vec<QueueItem>, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT id, job_id, payload, status, attempts, max_attempts, error_msg, created_at
             FROM print_queue
             WHERE status IN ('pending', 'running')
             ORDER BY id ASC",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            let payload_str: String = row.get(2)?;
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, i64>(1)?,
                payload_str,
                row.get::<_, String>(3)?,
                row.get::<_, i32>(4)?,
                row.get::<_, i32>(5)?,
                row.get::<_, Option<String>>(6)?,
                row.get::<_, String>(7)?,
            ))
        })
        .map_err(|e| e.to_string())?;

    let mut items = Vec::new();
    for row in rows {
        let (id, job_id, payload_str, status, attempts, max_attempts, error_msg, created_at) =
            row.map_err(|e| e.to_string())?;

        let (file_name, printer) = serde_json::from_str::<PrintTask>(&payload_str)
            .map(|t| (t.file_name, t.printer))
            .unwrap_or_else(|_| ("unknown".to_string(), String::new()));

        items.push(QueueItem {
            id,
            job_id,
            status,
            attempts,
            max_attempts,
            error_msg,
            created_at,
            file_name,
            printer,
        });
    }
    Ok(items)
}

pub fn pending_count(db: &Mutex<Connection>) -> Result<i64, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    conn.query_row(
        "SELECT COUNT(*) FROM print_queue WHERE status IN ('pending', 'running')",
        [],
        |row| row.get(0),
    )
    .map_err(|e| e.to_string())
}

// ── Event helpers ──

fn emit_status(app: &AppHandle, job_id: i64, file_name: &str, status: &str, error_msg: Option<&str>) {
    let state = app.state::<AppState>();
    let _ = PrintJobRepo::update_status(state.db(), job_id, status, error_msg);

    let _ = app.emit(
        "print-job-update",
        PrintJobEvent {
            job_id,
            file_name: file_name.to_string(),
            status: status.to_string(),
            error_msg: error_msg.map(String::from),
        },
    );
}

// ── Worker ──

const AUTO_ASSIGN_PRINTER: &str = "__auto__";

async fn resolve_printer(
    printer: &str,
    service_url: &str,
    http: &reqwest::Client,
    printer_blacklist: &[String],
) -> Result<String, String> {
    if !printer.is_empty() && printer != AUTO_ASSIGN_PRINTER {
        return Ok(printer.to_string());
    }

    let resp = http
        .get(format!("{}/print/getPrintServers", service_url))
        .send()
        .await
        .map_err(|e| format!("获取打印机列表失败: {}", e))?;

    let result: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("解析打印机列表失败: {}", e))?;

    let mut printers: Vec<String> = result
        .get("data")
        .and_then(|d| d.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default();

    let exclude: std::collections::HashSet<&str> =
        printer_blacklist.iter().map(|s| s.as_str()).collect();
    printers.retain(|p| !exclude.contains(p.as_str()));

    if printers.is_empty() {
        return Err("没有可用的打印机（可能已全部被隐藏）".to_string());
    }

    use rand::Rng;
    let idx = rand::thread_rng().gen_range(0..printers.len());
    Ok(printers[idx].clone())
}

/// Build the WebSocket URL from the HTTP service URL (http://host:port → ws://host:port/print)
fn ws_url_from_service(service_url: &str) -> String {
    let base = service_url
        .replace("https://", "wss://")
        .replace("http://", "ws://");
    format!("{}/print", base.trim_end_matches('/'))
}

/// Connect to Java WebSocket and retrieve sessionId from the first message.
async fn ws_get_session_id(
    ws_url: &str,
) -> Result<(
    futures_util::stream::SplitStream<tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>>,
    String,
), String> {
    let (ws_stream, _) = tokio_tungstenite::connect_async(ws_url)
        .await
        .map_err(|e| format!("WebSocket 连接失败: {}", e))?;

    let (_write, mut read) = futures_util::StreamExt::split(ws_stream);

    let first_msg = tokio::time::timeout(
        std::time::Duration::from_secs(10),
        read.next(),
    )
    .await
    .map_err(|_| "WebSocket 获取 sessionId 超时".to_string())?
    .ok_or_else(|| "WebSocket 连接已关闭".to_string())?
    .map_err(|e| format!("WebSocket 读取失败: {}", e))?;

    let text = match first_msg {
        tokio_tungstenite::tungstenite::Message::Text(t) => t.to_string(),
        other => return Err(format!("WebSocket 首条消息格式异常: {:?}", other)),
    };

    let json: serde_json::Value =
        serde_json::from_str(&text).map_err(|e| format!("解析 sessionId 失败: {}", e))?;

    let session_id = json
        .get("data")
        .and_then(|v| v.as_str())
        .ok_or_else(|| format!("sessionId 不存在于响应中: {}", text))?
        .to_string();

    Ok((read, session_id))
}

/// Determine what WebSocket status code means "success" for each print type.
fn terminal_status_for_type(print_type: &str) -> (Vec<i64>, Vec<i64>) {
    match print_type {
        "PDF" => (
            vec![200002],                   // success: 打印结束
            vec![200003, 200005, 200006, 200007, 200008, 200009], // failure
        ),
        "IMG" => (
            vec![200001],                   // success: 开始打印 (IMG has no completion event)
            vec![200003, 200008, 200009],
        ),
        _ => (vec![], vec![]),              // TEXT/HTML: no WS status, use HTTP only
    }
}

/// Wait on the WebSocket for a terminal print status.
/// Returns Ok(status_code) for success, Err(msg) for failure/timeout.
async fn wait_for_ws_result(
    read: &mut futures_util::stream::SplitStream<tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>>,
    success_codes: &[i64],
    failure_codes: &[i64],
) -> Result<i64, String> {
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(WS_TIMEOUT_SECS);

    loop {
        let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
        if remaining.is_zero() {
            return Err("等待打印结果超时".to_string());
        }

        let msg = tokio::time::timeout(remaining, read.next())
            .await
            .map_err(|_| "等待打印结果超时".to_string())?
            .ok_or_else(|| "WebSocket 连接意外关闭".to_string())?
            .map_err(|e| format!("WebSocket 读取错误: {}", e))?;

        let text = match msg {
            tokio_tungstenite::tungstenite::Message::Text(t) => t.to_string(),
            tokio_tungstenite::tungstenite::Message::Close(_) => {
                return Err("WebSocket 被服务端关闭".to_string());
            }
            _ => continue,
        };

        let json: serde_json::Value = match serde_json::from_str(&text) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let code = json.get("code").and_then(|v| v.as_i64()).unwrap_or(0);
        let ws_msg = json.get("msg").and_then(|v| v.as_str()).unwrap_or("");

        eprintln!("[Queue] WS 状态: code={}, msg={}", code, ws_msg);

        if success_codes.contains(&code) {
            return Ok(code);
        }
        if failure_codes.contains(&code) {
            return Err(format!("{} (code: {})", ws_msg, code));
        }
    }
}

async fn execute_task(
    task: &PrintTask,
    files_dir: &PathBuf,
    http: &reqwest::Client,
) -> Result<String, String> {
    let actual_printer =
        resolve_printer(&task.printer, &task.service_url, http, &task.printer_blacklist).await?;

    let (success_codes, failure_codes) = terminal_status_for_type(&task.print_type);
    let use_ws = !success_codes.is_empty();

    // 1. Optionally connect WebSocket to get sessionId
    let ws_url = ws_url_from_service(&task.service_url);
    let (mut ws_read, session_id) = if use_ws {
        let (read, sid) = ws_get_session_id(&ws_url).await?;
        eprintln!("[Queue] WebSocket 已连接, sessionId: {}", sid);
        (Some(read), Some(sid))
    } else {
        (None, None)
    };

    // 2. Read file
    let file_path = files_dir.join(&task.file_name);
    let file_bytes = tokio::fs::read(&file_path)
        .await
        .map_err(|e| format!("读取文件失败: {}", e))?;

    // 3. Build multipart form (include sessionId if available)
    let part = reqwest::multipart::Part::bytes(file_bytes).file_name(task.file_name.clone());

    let mut form = reqwest::multipart::Form::new()
        .text("type", task.print_type.clone())
        .text("source", "blob")
        .text("copies", task.copies.to_string())
        .text("color", task.color.to_string())
        .text("paperSize", task.paper_size.clone())
        .text("direction", task.direction.clone())
        .text("printServer", actual_printer.clone())
        .part("file", part);

    if let Some(ref sid) = session_id {
        form = form.text("sessionId", sid.clone());
    }

    // 4. POST to Java
    let resp = http
        .post(format!("{}/print/single", task.service_url))
        .multipart(form)
        .send()
        .await
        .map_err(|e| format!("请求打印服务失败: {}", e))?;

    let result: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("解析响应失败: {}", e))?;

    let http_code = result.get("code").and_then(|v| v.as_i64()).unwrap_or(1);
    if http_code != 0 {
        let msg = result
            .get("msg")
            .and_then(|v| v.as_str())
            .unwrap_or("打印失败");
        return Err(msg.to_string());
    }

    // 5. Wait for WebSocket confirmation (or return immediately for TEXT/HTML)
    if let Some(ref mut read) = ws_read {
        wait_for_ws_result(read, &success_codes, &failure_codes).await?;
    }

    Ok(actual_printer)
}

pub fn spawn_worker(app: &AppHandle) {
    let app_handle = app.clone();
    let data_dir = app
        .path()
        .app_data_dir()
        .expect("无法获取 app_data_dir");
    let files_dir = data_dir.join("files");
    let http = reqwest::Client::new();

    tauri::async_runtime::spawn(async move {
        loop {
            let queue_db = {
                let qs = app_handle.try_state::<QueueState>();
                match qs {
                    Some(s) => Arc::clone(&s.db),
                    None => {
                        tokio::time::sleep(std::time::Duration::from_millis(POLL_INTERVAL_MS)).await;
                        continue;
                    }
                }
            };

            let next = fetch_next(&queue_db);
            match next {
                Ok(Some((queue_id, task))) => {
                    emit_status(&app_handle, task.job_id, &task.file_name, "printing", None);

                    match execute_task(&task, &files_dir, &http).await {
                        Ok(actual_printer) => {
                            let _ = mark_done(&queue_db, queue_id);
                            if task.printer != actual_printer {
                                let state = app_handle.state::<AppState>();
                                let _ = PrintJobRepo::update_printer(
                                    state.db(), task.job_id, &actual_printer,
                                );
                            }
                            emit_status(&app_handle, task.job_id, &task.file_name, "done", None);

                            let file_to_delete = files_dir.join(&task.file_name);
                            if let Err(e) = tokio::fs::remove_file(&file_to_delete).await {
                                eprintln!("[Queue] 自动删除文件失败 {}: {}", task.file_name, e);
                            } else {
                                let _ = app_handle.emit("file-changed", ());
                            }
                        }
                        Err(msg) => {
                            let permanent = mark_failed(&queue_db, queue_id, &msg)
                                .unwrap_or(true);
                            let status = if permanent { "failed" } else { "queued" };
                            emit_status(
                                &app_handle,
                                task.job_id,
                                &task.file_name,
                                status,
                                Some(&msg),
                            );
                            tokio::time::sleep(std::time::Duration::from_millis(RETRY_BACKOFF_MS))
                                .await;
                        }
                    }
                }
                Ok(None) => {
                    tokio::time::sleep(std::time::Duration::from_millis(POLL_INTERVAL_MS)).await;
                }
                Err(e) => {
                    eprintln!("[Queue] 获取任务失败: {}", e);
                    tokio::time::sleep(std::time::Duration::from_millis(POLL_INTERVAL_MS)).await;
                }
            }
        }
    });
}
