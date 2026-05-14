# Plan 2: Rust 后端 Schema 扩展与日志系统

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 升级 SQLite schema（print_jobs 扩展字段 + system_logs 表 + app_config 表），实现 DB 迁移机制，扩展 Tauri commands。

**Architecture:** 在 `db.rs` 中增加版本号迁移逻辑，用 `PRAGMA user_version` 跟踪 schema 版本。每次启动检查版本并执行增量 ALTER TABLE / CREATE TABLE。

**Tech Stack:** Rust, rusqlite 0.31, serde, Tauri 2

**Dependencies:** 无（第一波，可与 Plan 1、Plan 5 并行）

**Files to modify:**
- `src-tauri/src/db.rs` — 迁移逻辑
- `src-tauri/src/entities.rs` — 扩展 PrintJob、新增 SystemLog、AppSetting
- `src-tauri/src/repos.rs` — 扩展 PrintJobRepo、新增 SystemLogRepo
- `src-tauri/src/commands.rs` — 新增日志与配置 commands
- `src-tauri/src/lib.rs` — 注册新 commands

---

### Task 1: DB 迁移机制

**Files:** Modify `src-tauri/src/db.rs`

- [ ] **Step 1: 实现版本迁移框架**

在 `init_db` 中：
1. 查询 `PRAGMA user_version` 获取当前版本
2. 按版本号依次执行迁移
3. 更新 `PRAGMA user_version`

迁移版本定义：
- **V0 → V1**：现有 `print_jobs` 表（已存在，兼容处理）
- **V1 → V2**：ALTER TABLE `print_jobs` 添加扩展字段
- **V2 → V3**：CREATE TABLE `system_logs`
- **V3 → V4**：CREATE TABLE `app_config`

```rust
fn get_db_version(conn: &Connection) -> Result<i32, String> {
    conn.query_row("PRAGMA user_version", [], |row| row.get(0))
        .map_err(|e| e.to_string())
}

fn set_db_version(conn: &Connection, version: i32) -> Result<(), String> {
    conn.execute_batch(&format!("PRAGMA user_version = {}", version))
        .map_err(|e| e.to_string())
}

fn migrate(conn: &Connection) -> Result<(), String> {
    let version = get_db_version(conn)?;

    if version < 1 {
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS print_jobs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );"
        ).map_err(|e| e.to_string())?;
        set_db_version(conn, 1)?;
    }

    if version < 2 {
        // 扩展 print_jobs 字段
        conn.execute_batch("
            ALTER TABLE print_jobs ADD COLUMN status TEXT NOT NULL DEFAULT 'queued';
            ALTER TABLE print_jobs ADD COLUMN printer TEXT DEFAULT '';
            ALTER TABLE print_jobs ADD COLUMN print_type TEXT DEFAULT '';
            ALTER TABLE print_jobs ADD COLUMN source TEXT DEFAULT 'desktop';
            ALTER TABLE print_jobs ADD COLUMN copies INTEGER NOT NULL DEFAULT 1;
            ALTER TABLE print_jobs ADD COLUMN file_path TEXT DEFAULT '';
            ALTER TABLE print_jobs ADD COLUMN file_size INTEGER DEFAULT 0;
            ALTER TABLE print_jobs ADD COLUMN error_msg TEXT DEFAULT '';
            ALTER TABLE print_jobs ADD COLUMN finished_at TEXT DEFAULT NULL;
        ").map_err(|e| e.to_string())?;
        set_db_version(conn, 2)?;
    }

    if version < 3 {
        conn.execute_batch("
            CREATE TABLE IF NOT EXISTS system_logs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp TEXT NOT NULL DEFAULT (datetime('now')),
                level TEXT NOT NULL DEFAULT 'INFO',
                category TEXT NOT NULL DEFAULT 'system',
                message TEXT NOT NULL,
                logger TEXT DEFAULT ''
            );
            CREATE INDEX IF NOT EXISTS idx_logs_timestamp ON system_logs(timestamp DESC);
            CREATE INDEX IF NOT EXISTS idx_logs_category ON system_logs(category);
            CREATE INDEX IF NOT EXISTS idx_logs_level ON system_logs(level);
        ").map_err(|e| e.to_string())?;
        set_db_version(conn, 3)?;
    }

    if version < 4 {
        conn.execute_batch("
            CREATE TABLE IF NOT EXISTS app_config (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
        ").map_err(|e| e.to_string())?;
        set_db_version(conn, 4)?;
    }

    Ok(())
}
```

- [ ] **Step 2: 修改 init_db 调用 migrate**

将原来的 `CREATE TABLE` 替换为 `migrate(&conn)?;`

- [ ] **Step 3: cargo check**

```bash
cd src-tauri && cargo check
```

Expected: 编译通过。

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/db.rs
git commit -m "feat(db): add versioned migration system (V1-V4)"
```

---

### Task 2: 扩展实体定义

**Files:** Modify `src-tauri/src/entities.rs`

- [ ] **Step 1: 扩展 PrintJob 实体**

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrintJob {
    pub id: i64,
    pub name: String,
    pub status: String,        // queued, printing, done, failed, cancelled
    pub printer: String,
    pub print_type: String,    // PDF, IMG, TEXT, HTML
    pub source: String,        // desktop, mobile
    pub copies: i32,
    pub file_path: String,
    pub file_size: i64,
    pub error_msg: String,
    pub created_at: String,
    pub finished_at: Option<String>,
}

/// 创建打印任务的请求参数
#[derive(Debug, Clone, Deserialize)]
pub struct CreatePrintJobRequest {
    pub name: String,
    pub printer: Option<String>,
    pub print_type: Option<String>,
    pub source: Option<String>,
    pub copies: Option<i32>,
    pub file_path: Option<String>,
    pub file_size: Option<i64>,
}
```

- [ ] **Step 2: 新增 SystemLog 实体**

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemLog {
    pub id: i64,
    pub timestamp: String,
    pub level: String,      // INFO, WARN, ERROR, DEBUG
    pub category: String,   // service, print, upload, system
    pub message: String,
    pub logger: String,
}

/// 日志查询参数
#[derive(Debug, Clone, Deserialize)]
pub struct LogQuery {
    pub level: Option<String>,
    pub category: Option<String>,
    pub keyword: Option<String>,
    pub limit: Option<i64>,
}
```

- [ ] **Step 3: cargo check**

```bash
cd src-tauri && cargo check
```

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/entities.rs
git commit -m "feat(entities): extend PrintJob, add SystemLog and request types"
```

---

### Task 3: 扩展 Repository 层

**Files:** Modify `src-tauri/src/repos.rs`

- [ ] **Step 1: 更新 PrintJobRepo 的 SQL 查询**

所有 SELECT 语句需包含新增字段。`create` 方法接受 `CreatePrintJobRequest` 参数。新增 `update_status` 方法。

关键方法签名：
- `list(db, limit: Option<i64>) -> Result<Vec<PrintJob>>` — 支持分页
- `create(db, req: &CreatePrintJobRequest) -> Result<PrintJob>` — 完整创建
- `update_status(db, id: i64, status: &str, error_msg: Option<&str>) -> Result<()>` — 状态更新
- `get_by_id(db, id) -> Result<PrintJob>`
- `delete(db, id) -> Result<()>`
- `count_by_status(db, status: &str) -> Result<i64>` — 统计
- `count_today_completed(db) -> Result<i64>` — 今日完成数

- [ ] **Step 2: 新增 SystemLogRepo**

```rust
pub struct SystemLogRepo;

impl SystemLogRepo {
    pub fn insert(db: &Mutex<Connection>, level: &str, category: &str, message: &str, logger: &str) -> Result<(), String>;
    pub fn query(db: &Mutex<Connection>, query: &LogQuery) -> Result<Vec<SystemLog>, String>;
    pub fn clear(db: &Mutex<Connection>) -> Result<(), String>;
    pub fn count(db: &Mutex<Connection>) -> Result<i64, String>;
}
```

- [ ] **Step 3: 更新测试**

更新 `test_create_and_list` 等测试适配新字段。新增 SystemLogRepo 测试。

- [ ] **Step 4: cargo test**

```bash
cd src-tauri && cargo test
```

Expected: 所有测试通过。

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/repos.rs
git commit -m "feat(repos): extend PrintJobRepo fields, add SystemLogRepo with tests"
```

---

### Task 4: 新增 Tauri Commands

**Files:** Modify `src-tauri/src/commands.rs`, `src-tauri/src/lib.rs`

- [ ] **Step 1: 更新 print_jobs_create 签名**

接受完整的 `CreatePrintJobRequest` 参数（而非仅 `name`）。

- [ ] **Step 2: 新增状态/统计 commands**

```rust
#[tauri::command]
pub fn print_jobs_update_status(
    state: tauri::State<'_, AppState>,
    id: i64,
    status: String,
    error_msg: Option<String>,
) -> Result<(), String>;

#[tauri::command]
pub fn print_jobs_count_queue(state: tauri::State<'_, AppState>) -> Result<i64, String>;

#[tauri::command]
pub fn print_jobs_count_today(state: tauri::State<'_, AppState>) -> Result<i64, String>;
```

- [ ] **Step 3: 新增日志 commands**

```rust
#[tauri::command]
pub fn log_insert(
    state: tauri::State<'_, AppState>,
    level: String,
    category: String,
    message: String,
) -> Result<(), String>;

#[tauri::command]
pub fn log_query(
    state: tauri::State<'_, AppState>,
    level: Option<String>,
    category: Option<String>,
    keyword: Option<String>,
    limit: Option<i64>,
) -> Result<Vec<SystemLog>, String>;

#[tauri::command]
pub fn log_clear(state: tauri::State<'_, AppState>) -> Result<(), String>;
```

- [ ] **Step 4: 在 lib.rs 中注册新 commands**

在 `tauri::generate_handler![]` 中追加所有新 command。

- [ ] **Step 5: cargo check && cargo test**

```bash
cd src-tauri && cargo check && cargo test
```

Expected: 编译通过，所有测试通过。

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/commands.rs src-tauri/src/lib.rs
git commit -m "feat(commands): add print job status/stats and system log commands"
```

---

### Task 5: 提供内部日志记录工具函数

**Files:** Create `src-tauri/src/logger.rs`, Modify `src-tauri/src/lib.rs`

- [ ] **Step 1: 创建 logger.rs**

提供 Rust 端内部调用的日志记录函数，写入 system_logs 表：

```rust
use crate::db::AppState;
use crate::repos::SystemLogRepo;

pub fn log_info(state: &AppState, category: &str, message: &str) {
    let _ = SystemLogRepo::insert(state.db(), "INFO", category, message, "rust");
}

pub fn log_warn(state: &AppState, category: &str, message: &str) {
    let _ = SystemLogRepo::insert(state.db(), "WARN", category, message, "rust");
}

pub fn log_error(state: &AppState, category: &str, message: &str) {
    let _ = SystemLogRepo::insert(state.db(), "ERROR", category, message, "rust");
}
```

这些函数将被 Plan 5 (LAN HTTP) 和 Plan 7 (打印执行) 调用。

- [ ] **Step 2: 在 lib.rs 中注册 mod logger**

- [ ] **Step 3: cargo check**

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/logger.rs src-tauri/src/lib.rs
git commit -m "feat(logger): add internal Rust logging helpers writing to system_logs table"
```

---

### 验收标准

1. `cargo check` 编译通过
2. `cargo test` 所有测试通过（PrintJobRepo CRUD + SystemLogRepo CRUD）
3. 现有 `print_jobs` 表能自动迁移到 V2 schema（新增字段有默认值）
4. `system_logs` 表可通过 commands 插入和查询
5. `log_insert` command 接受 level/category/message，`log_query` 支持过滤
6. `print_jobs_count_queue` 和 `print_jobs_count_today` 返回正确统计
7. 旧数据库文件升级不丢失数据（迁移幂等）
