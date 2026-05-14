# Tauri 2.0 持久化存储 实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 为 Tauri 2.0 桌面应用添加三层持久化存储能力：tauri-plugin-store（键值配置）、rusqlite（SQLite 业务数据）、std::fs + plugin-dialog（文件管理）。

**Architecture:** Rust 端通过 Tauri State 注入 SQLite 连接，command 层薄转发；键值配置由前端直接调 tauri-plugin-store；文件管理利用 Tauri 内置 `app_data_dir()` + std::fs。

**Tech Stack:** Tauri 2.0, rusqlite 0.31 (bundled), tauri-plugin-store 2, tauri-plugin-dialog 2, base64 0.22, serde/serde_json

**Files to create:**
- `src-tauri/src/db.rs` — AppState, DB 连接, 迁移
- `src-tauri/src/entities.rs` — PrintJob 实体
- `src-tauri/src/repos.rs` — PrintJobRepo CRUD
- `src-tauri/src/commands.rs` — 所有 Tauri commands

**Files to modify:**
- `src-tauri/Cargo.toml` — 添加 rusqlite, base64, tauri-plugin-store, tauri-plugin-dialog
- `package.json` — 添加 @tauri-apps/plugin-store, @tauri-apps/plugin-dialog
- `src-tauri/src/lib.rs` — 注册插件, manage AppState, 注册 commands

---

### Task 1: 添加依赖

**Files:**
- Modify: `src-tauri/Cargo.toml`
- Modify: `package.json`

- [ ] **Step 1: 添加 Cargo 依赖**

在 `src-tauri/Cargo.toml` 的 `[dependencies]` 中追加：

```toml
tauri-plugin-store = "2"
tauri-plugin-dialog = "2"
rusqlite = { version = "0.31", features = ["bundled"] }
base64 = "0.22"
```

- [ ] **Step 2: 添加 npm 依赖**

在 `package.json` 的 `dependencies` 中追加：

```json
"@tauri-apps/plugin-store": "^2",
"@tauri-apps/plugin-dialog": "^2"
```

- [ ] **Step 3: 安装依赖并验证编译**

```bash
pnpm install
cd src-tauri && cargo check
```

Expected: `cargo check` 通过。

- [ ] **Step 4: Commit**

```bash
cd D:/APP_WQ/PRINT_APP
git add src-tauri/Cargo.toml package.json pnpm-lock.yaml
git commit -m "chore(deps): add storage dependencies (rusqlite, store, dialog, base64)"
```

---

### Task 2: 创建数据库模块 (db.rs)

**Files:**
- Create: `src-tauri/src/db.rs`

- [ ] **Step 1: 编写 db.rs**

```rust
use rusqlite::Connection;
use std::sync::Mutex;
use tauri::AppHandle;

pub struct AppState {
    pub db: Mutex<Connection>,
}

impl AppState {
    pub fn db(&self) -> &Mutex<Connection> {
        &self.db
    }
}

pub fn init_db(app_handle: &AppHandle) -> Result<Connection, String> {
    let data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;

    std::fs::create_dir_all(&data_dir).map_err(|e| e.to_string())?;

    let db_path = data_dir.join("app.db");
    let conn = Connection::open(&db_path).map_err(|e| e.to_string())?;

    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS print_jobs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );"
    ).map_err(|e| e.to_string())?;

    Ok(conn)
}
```

- [ ] **Step 2: 验证编译**

```bash
cd src-tauri && cargo check
```

Expected: db.rs 模块编译通过。

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/db.rs
git commit -m "feat(db): add AppState, DB connection, and migration"
```

---

### Task 3: 创建实体模块 (entities.rs)

**Files:**
- Create: `src-tauri/src/entities.rs`

- [ ] **Step 1: 编写 entities.rs**

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrintJob {
    pub id: i64,
    pub name: String,
    pub created_at: String,
}
```

- [ ] **Step 2: 验证编译**

```bash
cd src-tauri && cargo check
```

Expected: 编译通过。

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/entities.rs
git commit -m "feat(entities): add PrintJob entity struct"
```

---

### Task 4: 创建仓储模块 (repos.rs) 含单元测试

**Files:**
- Create: `src-tauri/src/repos.rs`

- [ ] **Step 1: 编写 repos.rs（含测试）**

```rust
use rusqlite::Connection;
use std::sync::Mutex;
use crate::entities::PrintJob;

pub struct PrintJobRepo;

impl PrintJobRepo {
    pub fn list(db: &Mutex<Connection>) -> Result<Vec<PrintJob>, String> {
        let conn = db.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare("SELECT id, name, created_at FROM print_jobs ORDER BY id DESC")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |row| {
                Ok(PrintJob {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    created_at: row.get(2)?,
                })
            })
            .map_err(|e| e.to_string())?;
        let mut jobs = Vec::new();
        for row in rows {
            jobs.push(row.map_err(|e| e.to_string())?);
        }
        Ok(jobs)
    }

    pub fn create(db: &Mutex<Connection>, name: &str) -> Result<PrintJob, String> {
        let conn = db.lock().map_err(|e| e.to_string())?;
        conn.execute("INSERT INTO print_jobs (name) VALUES (?1)", [name])
            .map_err(|e| e.to_string())?;
        let id = conn.last_insert_rowid();
        let created_at: String = conn
            .query_row(
                "SELECT created_at FROM print_jobs WHERE id = ?1",
                [id],
                |row| row.get(0),
            )
            .map_err(|e| e.to_string())?;
        Ok(PrintJob {
            id,
            name: name.to_string(),
            created_at,
        })
    }

    pub fn get_by_id(db: &Mutex<Connection>, id: i64) -> Result<PrintJob, String> {
        let conn = db.lock().map_err(|e| e.to_string())?;
        conn.query_row(
            "SELECT id, name, created_at FROM print_jobs WHERE id = ?1",
            [id],
            |row| {
                Ok(PrintJob {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    created_at: row.get(2)?,
                })
            },
        )
        .map_err(|e| e.to_string())
    }

    pub fn delete(db: &Mutex<Connection>, id: i64) -> Result<(), String> {
        let conn = db.lock().map_err(|e| e.to_string())?;
        conn.execute("DELETE FROM print_jobs WHERE id = ?1", [id])
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn setup_test_db() -> Mutex<Connection> {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE print_jobs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );"
        ).unwrap();
        Mutex::new(conn)
    }

    #[test]
    fn test_create_and_list() {
        let db = setup_test_db();
        let job = PrintJobRepo::create(&db, "test-job").unwrap();
        assert_eq!(job.name, "test-job");
        assert!(job.id > 0);

        let list = PrintJobRepo::list(&db).unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].name, "test-job");
    }

    #[test]
    fn test_get_by_id() {
        let db = setup_test_db();
        let created = PrintJobRepo::create(&db, "find-me").unwrap();
        let found = PrintJobRepo::get_by_id(&db, created.id).unwrap();
        assert_eq!(found.name, "find-me");
    }

    #[test]
    fn test_get_by_id_not_found() {
        let db = setup_test_db();
        let result = PrintJobRepo::get_by_id(&db, 999);
        assert!(result.is_err());
    }

    #[test]
    fn test_delete() {
        let db = setup_test_db();
        let job = PrintJobRepo::create(&db, "to-delete").unwrap();
        PrintJobRepo::delete(&db, job.id).unwrap();
        let list = PrintJobRepo::list(&db).unwrap();
        assert!(list.is_empty());
    }
}
```

- [ ] **Step 2: 运行测试验证失败（repos 模块尚未集成）**

```bash
cd src-tauri && cargo test --lib repos::tests
```

Expected: 编译错误（repos 模块未注册在 lib.rs）。

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/repos.rs
git commit -m "feat(repos): add PrintJobRepo with CRUD and tests"
```

> **注意：** 此模块注册和测试运行将在 Task 6（更新 lib.rs）中完成。

---

### Task 5: 创建 Commands 模块 (commands.rs)

**Files:**
- Create: `src-tauri/src/commands.rs`

- [ ] **Step 1: 编写 commands.rs**

```rust
use std::fs;
use tauri::AppHandle;
use crate::db::AppState;
use crate::entities::PrintJob;
use crate::repos::PrintJobRepo;

// ── PrintJob commands ──

#[tauri::command]
pub fn print_jobs_list(state: tauri::State<'_, AppState>) -> Result<Vec<PrintJob>, String> {
    PrintJobRepo::list(state.db())
}

#[tauri::command]
pub fn print_jobs_create(
    state: tauri::State<'_, AppState>,
    name: String,
) -> Result<PrintJob, String> {
    PrintJobRepo::create(state.db(), &name)
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
    PrintJobRepo::delete(state.db(), id)
}

// ── File commands ──

#[tauri::command]
pub fn file_save(
    app_handle: AppHandle,
    name: String,
    bytes: Vec<u8>,
) -> Result<String, String> {
    let data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;
    let files_dir = data_dir.join("files");
    fs::create_dir_all(&files_dir).map_err(|e| e.to_string())?;
    let path = files_dir.join(&name);
    fs::write(&path, &bytes).map_err(|e| e.to_string())?;
    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
pub fn file_read(app_handle: AppHandle, name: String) -> Result<String, String> {
    let data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;
    let path = data_dir.join("files").join(&name);
    let bytes = fs::read(&path).map_err(|e| e.to_string())?;
    use base64::Engine;
    Ok(base64::engine::general_purpose::STANDARD.encode(&bytes))
}

#[tauri::command]
pub fn file_delete(app_handle: AppHandle, name: String) -> Result<(), String> {
    let data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;
    let path = data_dir.join("files").join(&name);
    fs::remove_file(&path).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn file_list(app_handle: AppHandle) -> Result<Vec<String>, String> {
    let data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;
    let files_dir = data_dir.join("files");
    if !files_dir.exists() {
        return Ok(Vec::new());
    }
    let mut names = Vec::new();
    let entries = fs::read_dir(&files_dir).map_err(|e| e.to_string())?;
    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        if entry.file_type().map_err(|e| e.to_string())?.is_file() {
            names.push(entry.file_name().to_string_lossy().to_string());
        }
    }
    Ok(names)
}
```

- [ ] **Step 2: 验证编译**

```bash
cd src-tauri && cargo check
```

Expected: 编译错误（commands 模块未注册，预期行为 — 将在下一步 lib.rs 中注册）。

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/commands.rs
git commit -m "feat(commands): add print job and file management commands"
```

---

### Task 6: 更新 lib.rs 集成所有模块

**Files:**
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: 重写 lib.rs**

将 `src-tauri/src/lib.rs` 替换为：

```rust
mod commands;
mod db;
mod entities;
mod repos;

use db::{init_db, AppState};

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
```

- [ ] **Step 2: 运行 cargo check 验证编译**

```bash
cd src-tauri && cargo check
```

Expected: 编译通过。

- [ ] **Step 3: 运行单元测试**

```bash
cd src-tauri && cargo test
```

Expected: 所有 repos::tests 测试通过。

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/lib.rs
git commit -m "feat(lib): integrate storage modules, register plugins and commands"
```

---

### Task 7: 前端验证 — 端到端确认

**Files:**
- Modify: `src/App.vue`（临时加验证代码）

- [ ] **Step 1: 在 App.vue 中添加存储验证脚本**

在 `<script setup>` 中追加验证逻辑（保留原有 greet 代码不变）：

```typescript
import { onMounted } from "vue";
import { Store } from "@tauri-apps/plugin-store";

// 验证 tauri-plugin-store
onMounted(async () => {
  const store = await Store.load("settings.json");
  await store.set("lastOpen", new Date().toISOString());
  const val = await store.get("lastOpen");
  console.log("[store] lastOpen:", val);

  // 验证 file commands
  try {
    await invoke("file_save", { name: "test.txt", bytes: Array.from(new TextEncoder().encode("hello")) });
    const files: string[] = await invoke("file_list");
    console.log("[files] list:", files);
    const content: string = await invoke("file_read", { name: "test.txt" });
    console.log("[files] read test.txt:", content);
  } catch (e) {
    console.error("[files] error:", e);
  }

  // 验证 print_jobs commands
  try {
    const job: any = await invoke("print_jobs_create", { name: "测试打印任务" });
    console.log("[db] created:", job);
    const jobs: any[] = await invoke("print_jobs_list");
    console.log("[db] list:", jobs);
    await invoke("print_jobs_delete", { id: job.id });
    console.log("[db] deleted, remaining:", (await invoke("print_jobs_list") as any[]).length);
  } catch (e) {
    console.error("[db] error:", e);
  }
});
```

- [ ] **Step 2: 运行 pnpm build 验证类型检查**

```bash
pnpm build
```

Expected: vue-tsc 和 vite build 均通过。

- [ ] **Step 3: Commit（验证通过后）**

```bash
git add src/App.vue
git commit -m "feat(app): add storage verification on mount"
```
