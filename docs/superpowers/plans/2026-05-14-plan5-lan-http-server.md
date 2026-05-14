# Plan 5: LAN HTTP 服务与手机扫码

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 在 Tauri 进程内嵌入 HTTP 服务（axum），支持手机通过局域网扫码上传文件到打印队列。

**Architecture:** axum HTTP 服务在 Tauri `setup` 阶段用 `tokio::spawn` 启动，绑定 `0.0.0.0:{configurable_port}`。共享 `AppState`（SQLite）实现文件上传 → 写入 files 目录 → 创建 print_job 记录的完整链路。

**Tech Stack:** Rust, axum 0.8+, tokio, qrcode crate, serde, Tauri 2

**Dependencies:** 无（第一波，可与 Plan 1、Plan 2 并行）。但上传后写入 print_jobs 表需要 Plan 2 的扩展 schema（如果 Plan 2 未完成，可先用基础 schema）。

**Files to create:**
- `src-tauri/src/http_server.rs` — axum HTTP 服务
- `src-tauri/src/network.rs` — 网络接口枚举（获取 LAN IP）
- `src-tauri/src/qr.rs` — QR 码生成
- `src-tauri/static/mobile-upload.html` — 移动端上传页面（静态 HTML）

**Files to modify:**
- `src-tauri/Cargo.toml` — 添加 axum, tokio, qrcode, image 依赖
- `src-tauri/src/lib.rs` — 启动 HTTP 服务，注册新 commands
- `src-tauri/src/commands.rs` — 新增 LAN 服务相关 commands
- `src-tauri/tauri.conf.json` — 如需放开 CSP 或添加资源目录

---

### Task 1: 添加 Rust 依赖

**Files:** Modify `src-tauri/Cargo.toml`

- [ ] **Step 1: 添加依赖**

```toml
axum = "0.8"
tokio = { version = "1", features = ["full"] }
qrcode = "0.14"
image = { version = "0.25", default-features = false, features = ["png"] }
tower-http = { version = "0.6", features = ["cors"] }
rand = "0.8"
```

注意：检查 Tauri 2 是否已内含 tokio runtime。如果是，则不需要单独添加 tokio，但需要确认 features 足够。Tauri 2 默认使用 tokio，但可能需要 `rt-multi-thread`。

- [ ] **Step 2: cargo check**

```bash
cd src-tauri && cargo check
```

- [ ] **Step 3: Commit**

```bash
git add src-tauri/Cargo.toml
git commit -m "chore(deps): add axum, tokio, qrcode, image for LAN HTTP server"
```

---

### Task 2: 网络接口枚举

**Files:** Create `src-tauri/src/network.rs`

- [ ] **Step 1: 实现获取本机 LAN IP**

```rust
use std::net::UdpSocket;

/// 获取本机局域网 IP 地址
/// 通过创建 UDP socket 连接外部地址来获取本地绑定的 IP
pub fn get_local_ip() -> Result<String, String> {
    let socket = UdpSocket::bind("0.0.0.0:0").map_err(|e| e.to_string())?;
    socket.connect("8.8.8.8:80").map_err(|e| e.to_string())?;
    let addr = socket.local_addr().map_err(|e| e.to_string())?;
    Ok(addr.ip().to_string())
}

/// 获取所有本地 IPv4 地址（排除 loopback）
/// 在多网卡环境下返回所有候选地址
pub fn get_all_local_ips() -> Vec<String> {
    // 使用 if_addrs 或手动枚举
    // 简化实现：先返回主 IP
    match get_local_ip() {
        Ok(ip) => vec![ip],
        Err(_) => vec!["127.0.0.1".to_string()],
    }
}
```

注意：如果需要更可靠的多网卡枚举，可以添加 `if-addrs` crate。MVP 阶段用 UDP trick 即可。

- [ ] **Step 2: Commit**

```bash
git add src-tauri/src/network.rs
git commit -m "feat(network): add LAN IP detection via UDP socket"
```

---

### Task 3: QR 码生成

**Files:** Create `src-tauri/src/qr.rs`

- [ ] **Step 1: 实现 QR 码生成**

```rust
use qrcode::QrCode;
use image::Luma;
use base64::Engine;

/// 生成 QR 码 PNG 图片的 base64 字符串
pub fn generate_qr_base64(content: &str) -> Result<String, String> {
    let code = QrCode::new(content.as_bytes()).map_err(|e| e.to_string())?;
    let image = code.render::<Luma<u8>>()
        .quiet_zone(true)
        .min_dimensions(256, 256)
        .build();

    let mut png_bytes = Vec::new();
    let encoder = image::codecs::png::PngEncoder::new(&mut png_bytes);
    image::ImageEncoder::write_image(
        &encoder,
        image.as_raw(),
        image.width(),
        image.height(),
        image::ExtendedColorType::L8,
    ).map_err(|e| e.to_string())?;

    Ok(base64::engine::general_purpose::STANDARD.encode(&png_bytes))
}
```

- [ ] **Step 2: Commit**

```bash
git add src-tauri/src/qr.rs
git commit -m "feat(qr): add QR code PNG generation with base64 output"
```

---

### Task 4: 移动端上传页面

**Files:** Create `src-tauri/static/mobile-upload.html`

- [ ] **Step 1: 创建静态 HTML 上传页面**

纯 HTML + 内联 CSS + vanilla JS，无外部依赖。功能：
- 响应式布局，适配手机屏幕
- 文件选择按钮（支持多文件）
- 拖拽上传区
- 上传进度显示
- 上传结果反馈（成功/失败）
- Token 从 URL query 参数中读取

页面设计参考截图主界面的上传区风格，但简化为移动端单页。

```html
<!DOCTYPE html>
<html lang="zh-CN">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>网络打印服务 - 手机上传</title>
  <style>
    /* 响应式样式，蓝色主题，参考截图 */
  </style>
</head>
<body>
  <header>网络打印服务</header>
  <main>
    <div class="upload-zone" id="uploadZone">
      <p>点击或拖拽文件到此处</p>
      <input type="file" id="fileInput" multiple>
      <button onclick="document.getElementById('fileInput').click()">选择文件</button>
    </div>
    <div id="fileList"></div>
  </main>
  <script>
    const token = new URLSearchParams(location.search).get('token') || '';
    const uploadZone = document.getElementById('uploadZone');
    const fileInput = document.getElementById('fileInput');

    fileInput.addEventListener('change', handleFiles);
    uploadZone.addEventListener('dragover', e => { e.preventDefault(); uploadZone.classList.add('drag-over'); });
    uploadZone.addEventListener('dragleave', () => uploadZone.classList.remove('drag-over'));
    uploadZone.addEventListener('drop', e => { e.preventDefault(); uploadZone.classList.remove('drag-over'); handleFiles({ target: { files: e.dataTransfer.files } }); });

    async function handleFiles(e) {
      for (const file of e.target.files) {
        await uploadFile(file);
      }
    }

    async function uploadFile(file) {
      const formData = new FormData();
      formData.append('file', file);
      formData.append('token', token);
      try {
        const resp = await fetch('/upload', { method: 'POST', body: formData });
        const result = await resp.json();
        showResult(file.name, result.code === 0);
      } catch (e) {
        showResult(file.name, false);
      }
    }

    function showResult(name, success) {
      const list = document.getElementById('fileList');
      const div = document.createElement('div');
      div.className = 'file-item ' + (success ? 'success' : 'error');
      div.textContent = (success ? '✓ ' : '✗ ') + name;
      list.prepend(div);
    }
  </script>
</body>
</html>
```

- [ ] **Step 2: Commit**

```bash
git add src-tauri/static/mobile-upload.html
git commit -m "feat(static): add mobile upload HTML page for LAN access"
```

---

### Task 5: axum HTTP 服务

**Files:** Create `src-tauri/src/http_server.rs`

- [ ] **Step 1: 实现 HTTP 服务**

核心路由：

| 方法 | 路径 | 功能 |
|------|------|------|
| GET | `/` | 返回移动端上传页面 HTML |
| POST | `/upload` | 接收文件上传 |
| GET | `/api/health` | 健康检查 |

关键实现：

```rust
use axum::{
    Router, Json,
    extract::{Multipart, State, Query},
    routing::{get, post},
    response::Html,
};
use std::sync::Arc;
use crate::db::AppState;

#[derive(Clone)]
pub struct HttpState {
    pub app_state: Arc<AppState>,
    pub token: String,
    pub allowed_extensions: Vec<String>,
    pub max_file_size: u64, // bytes
    pub files_dir: std::path::PathBuf,
}

pub fn create_router(state: HttpState) -> Router {
    Router::new()
        .route("/", get(index_handler))
        .route("/upload", post(upload_handler))
        .route("/api/health", get(health_handler))
        .layer(tower_http::cors::CorsLayer::permissive())
        .with_state(state)
}

async fn index_handler() -> Html<&'static str> {
    Html(include_str!("../static/mobile-upload.html"))
}

async fn upload_handler(
    State(state): State<HttpState>,
    Query(params): Query<HashMap<String, String>>,
    mut multipart: Multipart,
) -> Json<serde_json::Value> {
    // 1. 验证 token
    // 2. 读取 multipart field
    // 3. 校验扩展名
    // 4. 校验文件大小
    // 5. 写入 files_dir
    // 6. 创建 print_job 记录（source="mobile"）
    // 7. 返回结果
}

async fn health_handler() -> Json<serde_json::Value> {
    Json(serde_json::json!({"code": 0, "msg": "ok"}))
}
```

- [ ] **Step 2: 启动函数**

```rust
pub async fn start_server(state: HttpState, port: u16) -> Result<(), String> {
    let addr = format!("0.0.0.0:{}", port);
    let router = create_router(state);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .map_err(|e| e.to_string())?;
    axum::serve(listener, router)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}
```

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/http_server.rs
git commit -m "feat(http): add axum HTTP server with upload endpoint and token auth"
```

---

### Task 6: 集成到 Tauri + 新增 Commands

**Files:** Modify `src-tauri/src/lib.rs`, `src-tauri/src/commands.rs`

- [ ] **Step 1: 在 Tauri setup 中启动 HTTP 服务**

```rust
.setup(|app| {
    // ... 现有 DB 初始化 ...

    // 启动 LAN HTTP 服务
    let app_state = app.state::<AppState>();
    let data_dir = app.path().app_data_dir().unwrap();
    let files_dir = data_dir.join("files");
    std::fs::create_dir_all(&files_dir).unwrap();

    let http_state = http_server::HttpState {
        app_state: Arc::new(/* ... */),
        token: generate_token(),
        allowed_extensions: default_extensions(),
        max_file_size: 50 * 1024 * 1024, // 50MB
        files_dir,
    };

    let port = 5000u16; // 从配置读取
    tokio::spawn(async move {
        if let Err(e) = http_server::start_server(http_state, port).await {
            eprintln!("HTTP server error: {}", e);
        }
    });

    Ok(())
})
```

注意：需要处理 AppState 的共享问题。Tauri 的 `State` 是 `Arc` 包装的，需要在 tokio::spawn 中安全共享。可能需要将 AppState 包装为 `Arc<AppState>` 而非直接 `Mutex<Connection>`。这是一个需要仔细处理的点。

- [ ] **Step 2: 新增 Tauri commands**

```rust
#[tauri::command]
pub fn lan_server_url() -> Result<String, String> {
    let ip = crate::network::get_local_ip()?;
    Ok(format!("http://{}:{}", ip, 5000)) // port 从配置读取
}

#[tauri::command]
pub fn lan_server_qrcode() -> Result<String, String> {
    let url = lan_server_url()?;
    // 添加 token query 参数
    let url_with_token = format!("{}?token={}", url, "TODO_TOKEN");
    crate::qr::generate_qr_base64(&url_with_token)
}

#[tauri::command]
pub fn lan_server_token() -> Result<String, String> {
    // 返回当前 token
    Ok("TODO".to_string())
}
```

实际实现中 token 和 port 需要从共享状态中读取。

- [ ] **Step 3: 注册新 mod 和 commands**

在 lib.rs 中：
```rust
mod http_server;
mod network;
mod qr;
```

在 `generate_handler![]` 中注册新 commands。

- [ ] **Step 4: cargo check**

```bash
cd src-tauri && cargo check
```

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/
git commit -m "feat(lan): integrate HTTP server into Tauri setup, add URL/QR commands"
```

---

### Task 7: 端到端验证

- [ ] **Step 1: cargo build 验证**

```bash
cd src-tauri && cargo build
```

- [ ] **Step 2: pnpm tauri dev 启动**

启动应用后，在浏览器中访问 `http://localhost:5000`，应显示移动端上传页面。

- [ ] **Step 3: 手机测试**

用手机浏览器访问 `http://{电脑IP}:5000`，上传文件，验证：
1. 文件保存到 `app_data_dir/files/`
2. print_jobs 表新增记录（source="mobile"）

- [ ] **Step 4: 验证 QR 码 command**

在前端 console 中调用 `invoke('lan_server_qrcode')`，获得 base64 PNG 字符串。

---

### 注意事项

1. **防火墙**: Windows 首次监听 0.0.0.0:5000 可能弹出防火墙确认窗口。
2. **AppState 共享**: axum 和 Tauri 共享同一个 SQLite 连接需要仔细处理 Arc/Mutex。考虑使用 `Arc<Mutex<Connection>>` 或者为 HTTP 服务单独开一个连接。
3. **Token 安全**: Token 应为随机生成的字符串（每次启动重新生成），长度至少 16 字符。
4. **文件大小限制**: axum 的 multipart 有默认大小限制，需要配置 `DefaultBodyLimit`。

### 验收标准

1. `cargo build` 编译通过
2. 应用启动后 HTTP 服务在配置端口监听
3. `GET /` 返回移动端上传 HTML 页面
4. `POST /upload` 成功接收文件并存储
5. 无效 token 的上传请求被拒绝（返回错误码）
6. 不支持的文件扩展名被拒绝
7. `lan_server_url` command 返回正确的 LAN URL
8. `lan_server_qrcode` command 返回可扫码的 QR 码
9. 手机扫码后能打开上传页面并成功上传
