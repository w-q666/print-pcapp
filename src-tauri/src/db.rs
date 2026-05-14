use rusqlite::Connection;
use std::sync::Mutex;
use tauri::AppHandle;
use tauri::Manager;

pub struct AppState {
    pub db: Mutex<Connection>,
}

impl AppState {
    pub fn db(&self) -> &Mutex<Connection> {
        &self.db
    }
}

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
        // SQLite ALTER TABLE only supports one ADD COLUMN per statement
        conn.execute("ALTER TABLE print_jobs ADD COLUMN status TEXT NOT NULL DEFAULT 'queued'", [])
            .map_err(|e| e.to_string())?;
        conn.execute("ALTER TABLE print_jobs ADD COLUMN printer TEXT DEFAULT ''", [])
            .map_err(|e| e.to_string())?;
        conn.execute("ALTER TABLE print_jobs ADD COLUMN print_type TEXT DEFAULT ''", [])
            .map_err(|e| e.to_string())?;
        conn.execute("ALTER TABLE print_jobs ADD COLUMN source TEXT DEFAULT 'desktop'", [])
            .map_err(|e| e.to_string())?;
        conn.execute("ALTER TABLE print_jobs ADD COLUMN copies INTEGER NOT NULL DEFAULT 1", [])
            .map_err(|e| e.to_string())?;
        conn.execute("ALTER TABLE print_jobs ADD COLUMN file_path TEXT DEFAULT ''", [])
            .map_err(|e| e.to_string())?;
        conn.execute("ALTER TABLE print_jobs ADD COLUMN file_size INTEGER DEFAULT 0", [])
            .map_err(|e| e.to_string())?;
        conn.execute("ALTER TABLE print_jobs ADD COLUMN error_msg TEXT DEFAULT ''", [])
            .map_err(|e| e.to_string())?;
        conn.execute("ALTER TABLE print_jobs ADD COLUMN finished_at TEXT DEFAULT NULL", [])
            .map_err(|e| e.to_string())?;
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

pub fn init_db(app_handle: &AppHandle) -> Result<Connection, String> {
    let data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;

    std::fs::create_dir_all(&data_dir).map_err(|e| e.to_string())?;

    let db_path = data_dir.join("app.db");
    let conn = Connection::open(&db_path).map_err(|e| e.to_string())?;

    migrate(&conn)?;

    Ok(conn)
}
