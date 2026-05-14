# Tauri 2.0 持久化存储设计

**日期:** 2026-05-14
**状态:** 已确认

## 概述

为 Tauri 2.0 桌面应用添加三层持久化存储：键值配置、SQLite 业务数据、文件管理。原则：Tauri 已有能力的直接复用，不重复封装。

## 架构

```
┌─────────────────────────────────────────────────────┐
│  Vue 3 前端                                         │
│  键值配置：直接调 tauri-plugin-store                │
│  业务数据：通过 Tauri commands 间接访问              │
│  文件操作：通过 Tauri commands + plugin-dialog      │
├─────────────────────────────────────────────────────┤
│  Tauri Command 层                                   │
│  薄转发，参数序列化 → 调用 DB/FS → 返回             │
├──────────────┬────────────────┬─────────────────────┤
│  tauri-plugin│  rusqlite      │  std::fs            │
│  -store      │  (SQLite)      │  + app_data_dir()   │
│  (键值配置)   │  (业务数据)    │  + plugin-dialog    │
│  已内置       │  需自建        │  (文件管理)         │
└──────────────┴────────────────┴─────────────────────┘
    所有数据存在 App Data 目录下
    Windows: %APPDATA%/com.print.app/
```

## 模块详设

### 1. 键值配置 — tauri-plugin-store

**用途：** 应用设置、用户偏好、窗口位置尺寸等。

**实现：** 直接使用 Tauri 官方插件，不做任何二次封装。

**依赖：**
- Cargo: `tauri-plugin-store = "2"`
- npm: `@tauri-apps/plugin-store`

**用法示例：**
```js
import { Store } from '@tauri-apps/plugin-store';
const store = await Store.load('settings.json');
await store.set('theme', 'dark');
await store.set('windowWidth', 1280);
const name = await store.get('userName');
await store.save(); // 手动刷盘，也可 await store.set() 自动保存
```

```rust
// Rust 端注册插件
fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::default().build())
        .run(tauri::generate_context!())
        .expect("...");
}
```

### 2. 业务数据 — rusqlite

**用途：** 打印任务、客户信息、模板记录等结构化业务数据。

**依赖：**
- Cargo: `rusqlite = { version = "0.31", features = ["bundled"] }`

**核心设计：**

启动时打开 SQLite 连接并通过 Tauri State 注入：

```rust
use std::sync::Mutex;
use rusqlite::Connection;

struct AppState {
    db: Mutex<Connection>,
}

fn main() {
    let db_path = /* app_data_dir / "app.db" */;
    let db = Connection::open(&db_path).expect("failed to open db");
    // 执行迁移
    db.execute_batch("CREATE TABLE IF NOT EXISTS print_jobs (...);").unwrap();

    tauri::Builder::default()
        .manage(AppState { db: Mutex::new(db) })
        .run(tauri::generate_context!())
        .expect("...");
}
```

**表定义规范：** 每张业务表对应一个 Rust struct + repository 模块。

```rust
// src/entities/print_job.rs
#[derive(Debug, Serialize, Deserialize)]
struct PrintJob {
    id: i64,
    name: String,
    created_at: String,
}

// src/repos/print_job_repo.rs
struct PrintJobRepo;

impl PrintJobRepo {
    fn list(db: &Mutex<Connection>) -> Result<Vec<PrintJob>, String> { /* ... */ }
    fn create(db: &Mutex<Connection>, name: &str) -> Result<PrintJob, String> { /* ... */ }
    fn delete(db: &Mutex<Connection>, id: i64) -> Result<(), String> { /* ... */ }
}
```

**Commands 命名规范：** `<表名>_<操作>`

| Command | 说明 |
|---------|------|
| `print_jobs_list` | 查询所有打印任务 |
| `print_jobs_create` | 创建新的打印任务 |
| `print_jobs_delete` | 按 ID 删除打印任务 |
| `print_jobs_get_by_id` | 按 ID 获取单个任务 |

**迁移策略：** 启动时执行内联 SQL 字符串创建表（`CREATE TABLE IF NOT EXISTS`），不做外部迁移文件。数据量初期小，够用。

### 3. 文件管理 — Tauri 内置 + std::fs

**用途：** 打印产生的 PDF、图片、模板文件等。

**核心设计：** 利用 Tauri 内置能力 + Rust 标准库，不引入额外依赖。

| 能力 | 使用 |
|------|------|
| 获取 App 数据目录 | `app_handle.path().app_data_dir()` — Tauri 内置 |
| 文件读/写/删 | `std::fs` — Rust 标准库 |
| 用户选择文件 | `tauri-plugin-dialog` 的 `file.open()` / `file.save()` |

**依赖：**
- Cargo: `tauri-plugin-dialog = "2"`
- npm: `@tauri-apps/plugin-dialog`

**暴露 Commands：**

| Command | 说明 |
|---------|------|
| `file_save(name, bytes)` | 保存文件到 `{app_data}/files/` 目录 |
| `file_read(name)` | 读取文件，返回 base64 |
| `file_delete(name)` | 删除文件 |
| `file_list` | 列出所有文件 |

## 依赖汇总

| 用途 | Cargo | npm |
|------|-------|-----|
| 键值配置 | `tauri-plugin-store = "2"` | `@tauri-apps/plugin-store` |
| 文件对话框 | `tauri-plugin-dialog = "2"` | `@tauri-apps/plugin-dialog` |
| SQLite | `rusqlite = { version = "0.31", features = ["bundled"] }` | — |

## 错误处理

- Command 层统一返回 `Result<T, String>`，错误信息从 Rust 端透传到前端
- rusqlite 错误用 `.map_err(|e| e.to_string())` 转换
- 文件操作 I/O 错误同理
- 前端用 try-catch 处理 `invoke()` 调用，展示用户友好的错误提示

## 测试策略

- Rust 端：repository 层做单元测试，使用 `rusqlite` 的内存数据库 `Connection::open_in_memory()`
- 前端：Vitest（未来引入）测试 command 调用逻辑
- 集成测试：手动验证端到端数据流

## 扩展路径

- SQLite → PostgreSQL：替换 `rusqlite` 为 `sqlx`；Commands 签名不变；前端无感
- 单用户 → 多用户：引入用户认证层；数据库按 `user_id` 分区
