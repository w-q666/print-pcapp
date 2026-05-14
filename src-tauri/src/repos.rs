use rusqlite::Connection;
use std::sync::Mutex;
use crate::entities::{CreatePrintJobRequest, LogQuery, PrintJob, SystemLog};

pub struct PrintJobRepo;

impl PrintJobRepo {
    pub fn list(db: &Mutex<Connection>, limit: Option<i64>) -> Result<Vec<PrintJob>, String> {
        let conn = db.lock().map_err(|e| e.to_string())?;
        let sql = match limit {
            Some(n) => format!(
                "SELECT id, name, status, printer, print_type, source, copies, \
                 file_path, file_size, error_msg, created_at, finished_at \
                 FROM print_jobs ORDER BY id DESC LIMIT {}",
                n
            ),
            None => "SELECT id, name, status, printer, print_type, source, copies, \
                     file_path, file_size, error_msg, created_at, finished_at \
                     FROM print_jobs ORDER BY id DESC"
                .to_string(),
        };
        let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |row| {
                Ok(PrintJob {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    status: row.get(2)?,
                    printer: row.get(3)?,
                    print_type: row.get(4)?,
                    source: row.get(5)?,
                    copies: row.get(6)?,
                    file_path: row.get(7)?,
                    file_size: row.get(8)?,
                    error_msg: row.get(9)?,
                    created_at: row.get(10)?,
                    finished_at: row.get(11)?,
                })
            })
            .map_err(|e| e.to_string())?;
        let mut jobs = Vec::new();
        for row in rows {
            jobs.push(row.map_err(|e| e.to_string())?);
        }
        Ok(jobs)
    }

    pub fn create(db: &Mutex<Connection>, req: &CreatePrintJobRequest) -> Result<PrintJob, String> {
        let conn = db.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO print_jobs (name, printer, print_type, source, copies, file_path, file_size) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![
                req.name,
                req.printer.as_deref().unwrap_or(""),
                req.print_type.as_deref().unwrap_or(""),
                req.source.as_deref().unwrap_or("desktop"),
                req.copies.unwrap_or(1),
                req.file_path.as_deref().unwrap_or(""),
                req.file_size.unwrap_or(0),
            ],
        )
        .map_err(|e| e.to_string())?;

        let id = conn.last_insert_rowid();
        conn.query_row(
            "SELECT id, name, status, printer, print_type, source, copies, \
             file_path, file_size, error_msg, created_at, finished_at \
             FROM print_jobs WHERE id = ?1",
            [id],
            |row| {
                Ok(PrintJob {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    status: row.get(2)?,
                    printer: row.get(3)?,
                    print_type: row.get(4)?,
                    source: row.get(5)?,
                    copies: row.get(6)?,
                    file_path: row.get(7)?,
                    file_size: row.get(8)?,
                    error_msg: row.get(9)?,
                    created_at: row.get(10)?,
                    finished_at: row.get(11)?,
                })
            },
        )
        .map_err(|e| e.to_string())
    }

    pub fn get_by_id(db: &Mutex<Connection>, id: i64) -> Result<PrintJob, String> {
        let conn = db.lock().map_err(|e| e.to_string())?;
        conn.query_row(
            "SELECT id, name, status, printer, print_type, source, copies, \
             file_path, file_size, error_msg, created_at, finished_at \
             FROM print_jobs WHERE id = ?1",
            [id],
            |row| {
                Ok(PrintJob {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    status: row.get(2)?,
                    printer: row.get(3)?,
                    print_type: row.get(4)?,
                    source: row.get(5)?,
                    copies: row.get(6)?,
                    file_path: row.get(7)?,
                    file_size: row.get(8)?,
                    error_msg: row.get(9)?,
                    created_at: row.get(10)?,
                    finished_at: row.get(11)?,
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

    pub fn update_status(
        db: &Mutex<Connection>,
        id: i64,
        status: &str,
        error_msg: Option<&str>,
    ) -> Result<(), String> {
        let conn = db.lock().map_err(|e| e.to_string())?;
        let finished_at = if status == "done" || status == "failed" || status == "cancelled" {
            "datetime('now')"
        } else {
            "NULL"
        };
        let sql = format!(
            "UPDATE print_jobs SET status = ?1, error_msg = ?2, finished_at = {} WHERE id = ?3",
            finished_at
        );
        conn.execute(&sql, rusqlite::params![status, error_msg.unwrap_or(""), id])
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn count_by_status(db: &Mutex<Connection>, status: &str) -> Result<i64, String> {
        let conn = db.lock().map_err(|e| e.to_string())?;
        conn.query_row(
            "SELECT COUNT(*) FROM print_jobs WHERE status = ?1",
            [status],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())
    }

    pub fn count_today_completed(db: &Mutex<Connection>) -> Result<i64, String> {
        let conn = db.lock().map_err(|e| e.to_string())?;
        conn.query_row(
            "SELECT COUNT(*) FROM print_jobs WHERE status = 'done' AND date(finished_at) = date('now')",
            [],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())
    }
}

// ── SystemLogRepo ──

pub struct SystemLogRepo;

impl SystemLogRepo {
    pub fn insert(
        db: &Mutex<Connection>,
        level: &str,
        category: &str,
        message: &str,
        logger: &str,
    ) -> Result<(), String> {
        let conn = db.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO system_logs (level, category, message, logger) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![level, category, message, logger],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn query(db: &Mutex<Connection>, q: &LogQuery) -> Result<Vec<SystemLog>, String> {
        let conn = db.lock().map_err(|e| e.to_string())?;

        let mut conditions: Vec<String> = Vec::new();
        let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

        if let Some(ref level) = q.level {
            conditions.push(format!("level = ?{}", params.len() + 1));
            params.push(Box::new(level.clone()));
        }
        if let Some(ref category) = q.category {
            conditions.push(format!("category = ?{}", params.len() + 1));
            params.push(Box::new(category.clone()));
        }
        if let Some(ref keyword) = q.keyword {
            conditions.push(format!("message LIKE ?{}", params.len() + 1));
            params.push(Box::new(format!("%{}%", keyword)));
        }

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };

        let limit_clause = match q.limit {
            Some(n) => format!("LIMIT {}", n),
            None => "LIMIT 200".to_string(),
        };

        let sql = format!(
            "SELECT id, timestamp, level, category, message, logger \
             FROM system_logs {} ORDER BY id DESC {}",
            where_clause, limit_clause
        );

        let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
        let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(param_refs.as_slice(), |row| {
                Ok(SystemLog {
                    id: row.get(0)?,
                    timestamp: row.get(1)?,
                    level: row.get(2)?,
                    category: row.get(3)?,
                    message: row.get(4)?,
                    logger: row.get(5)?,
                })
            })
            .map_err(|e| e.to_string())?;

        let mut logs = Vec::new();
        for row in rows {
            logs.push(row.map_err(|e| e.to_string())?);
        }
        Ok(logs)
    }

    pub fn clear(db: &Mutex<Connection>) -> Result<(), String> {
        let conn = db.lock().map_err(|e| e.to_string())?;
        conn.execute("DELETE FROM system_logs", [])
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn count(db: &Mutex<Connection>) -> Result<i64, String> {
        let conn = db.lock().map_err(|e| e.to_string())?;
        conn.query_row("SELECT COUNT(*) FROM system_logs", [], |row| row.get(0))
            .map_err(|e| e.to_string())
    }

    pub fn query_all(db: &Mutex<Connection>, q: &LogQuery) -> Result<Vec<SystemLog>, String> {
        let conn = db.lock().map_err(|e| e.to_string())?;

        let mut conditions: Vec<String> = Vec::new();
        let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

        if let Some(ref level) = q.level {
            conditions.push(format!("level = ?{}", params.len() + 1));
            params.push(Box::new(level.clone()));
        }
        if let Some(ref category) = q.category {
            conditions.push(format!("category = ?{}", params.len() + 1));
            params.push(Box::new(category.clone()));
        }
        if let Some(ref keyword) = q.keyword {
            conditions.push(format!("message LIKE ?{}", params.len() + 1));
            params.push(Box::new(format!("%{}%", keyword)));
        }

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };

        let sql = format!(
            "SELECT id, timestamp, level, category, message, logger \
             FROM system_logs {} ORDER BY id DESC",
            where_clause
        );

        let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
        let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(param_refs.as_slice(), |row| {
                Ok(SystemLog {
                    id: row.get(0)?,
                    timestamp: row.get(1)?,
                    level: row.get(2)?,
                    category: row.get(3)?,
                    message: row.get(4)?,
                    logger: row.get(5)?,
                })
            })
            .map_err(|e| e.to_string())?;

        let mut logs = Vec::new();
        for row in rows {
            logs.push(row.map_err(|e| e.to_string())?);
        }
        Ok(logs)
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
                status TEXT NOT NULL DEFAULT 'queued',
                printer TEXT DEFAULT '',
                print_type TEXT DEFAULT '',
                source TEXT DEFAULT 'desktop',
                copies INTEGER NOT NULL DEFAULT 1,
                file_path TEXT DEFAULT '',
                file_size INTEGER DEFAULT 0,
                error_msg TEXT DEFAULT '',
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                finished_at TEXT DEFAULT NULL
            );
            CREATE TABLE system_logs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp TEXT NOT NULL DEFAULT (datetime('now')),
                level TEXT NOT NULL DEFAULT 'INFO',
                category TEXT NOT NULL DEFAULT 'system',
                message TEXT NOT NULL,
                logger TEXT DEFAULT ''
            );"
        ).unwrap();
        Mutex::new(conn)
    }

    #[test]
    fn test_create_and_list() {
        let db = setup_test_db();
        let req = CreatePrintJobRequest {
            name: "test-job".to_string(),
            printer: Some("HP-1020".to_string()),
            print_type: Some("PDF".to_string()),
            source: None,
            copies: Some(2),
            file_path: Some("/tmp/test.pdf".to_string()),
            file_size: Some(1024),
        };
        let job = PrintJobRepo::create(&db, &req).unwrap();
        assert_eq!(job.name, "test-job");
        assert_eq!(job.printer, "HP-1020");
        assert_eq!(job.copies, 2);
        assert_eq!(job.status, "queued");
        assert!(job.id > 0);

        let list = PrintJobRepo::list(&db, None).unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].name, "test-job");

        let limited = PrintJobRepo::list(&db, Some(0)).unwrap();
        assert_eq!(limited.len(), 0);
    }

    #[test]
    fn test_get_by_id() {
        let db = setup_test_db();
        let req = CreatePrintJobRequest {
            name: "find-me".to_string(),
            printer: None,
            print_type: None,
            source: None,
            copies: None,
            file_path: None,
            file_size: None,
        };
        let created = PrintJobRepo::create(&db, &req).unwrap();
        let found = PrintJobRepo::get_by_id(&db, created.id).unwrap();
        assert_eq!(found.name, "find-me");
        assert_eq!(found.source, "desktop");
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
        let req = CreatePrintJobRequest {
            name: "to-delete".to_string(),
            printer: None,
            print_type: None,
            source: None,
            copies: None,
            file_path: None,
            file_size: None,
        };
        let job = PrintJobRepo::create(&db, &req).unwrap();
        PrintJobRepo::delete(&db, job.id).unwrap();
        let list = PrintJobRepo::list(&db, None).unwrap();
        assert!(list.is_empty());
    }

    #[test]
    fn test_update_status() {
        let db = setup_test_db();
        let req = CreatePrintJobRequest {
            name: "status-test".to_string(),
            printer: None,
            print_type: None,
            source: None,
            copies: None,
            file_path: None,
            file_size: None,
        };
        let job = PrintJobRepo::create(&db, &req).unwrap();
        assert_eq!(job.status, "queued");

        PrintJobRepo::update_status(&db, job.id, "printing", None).unwrap();
        let updated = PrintJobRepo::get_by_id(&db, job.id).unwrap();
        assert_eq!(updated.status, "printing");
        assert!(updated.finished_at.is_none());

        PrintJobRepo::update_status(&db, job.id, "failed", Some("Paper jam")).unwrap();
        let failed = PrintJobRepo::get_by_id(&db, job.id).unwrap();
        assert_eq!(failed.status, "failed");
        assert_eq!(failed.error_msg, "Paper jam");
        assert!(failed.finished_at.is_some());
    }

    #[test]
    fn test_count_by_status() {
        let db = setup_test_db();
        let req = CreatePrintJobRequest {
            name: "count-test".to_string(),
            printer: None,
            print_type: None,
            source: None,
            copies: None,
            file_path: None,
            file_size: None,
        };
        PrintJobRepo::create(&db, &req).unwrap();
        PrintJobRepo::create(&db, &req).unwrap();

        let count = PrintJobRepo::count_by_status(&db, "queued").unwrap();
        assert_eq!(count, 2);

        let count_none = PrintJobRepo::count_by_status(&db, "done").unwrap();
        assert_eq!(count_none, 0);
    }

    #[test]
    fn test_count_today_completed() {
        let db = setup_test_db();
        let req = CreatePrintJobRequest {
            name: "today-test".to_string(),
            printer: None,
            print_type: None,
            source: None,
            copies: None,
            file_path: None,
            file_size: None,
        };
        let job = PrintJobRepo::create(&db, &req).unwrap();
        PrintJobRepo::update_status(&db, job.id, "done", None).unwrap();

        let count = PrintJobRepo::count_today_completed(&db).unwrap();
        assert_eq!(count, 1);
    }

    // ── SystemLogRepo tests ──

    #[test]
    fn test_log_insert_and_query() {
        let db = setup_test_db();
        SystemLogRepo::insert(&db, "INFO", "system", "App started", "rust").unwrap();
        SystemLogRepo::insert(&db, "ERROR", "print", "Printer offline", "rust").unwrap();

        let all = SystemLogRepo::query(
            &db,
            &LogQuery { level: None, category: None, keyword: None, limit: None },
        ).unwrap();
        assert_eq!(all.len(), 2);

        let errors = SystemLogRepo::query(
            &db,
            &LogQuery { level: Some("ERROR".to_string()), category: None, keyword: None, limit: None },
        ).unwrap();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].message, "Printer offline");

        let by_keyword = SystemLogRepo::query(
            &db,
            &LogQuery { level: None, category: None, keyword: Some("started".to_string()), limit: None },
        ).unwrap();
        assert_eq!(by_keyword.len(), 1);
    }

    #[test]
    fn test_log_count_and_clear() {
        let db = setup_test_db();
        SystemLogRepo::insert(&db, "INFO", "system", "msg1", "rust").unwrap();
        SystemLogRepo::insert(&db, "INFO", "system", "msg2", "rust").unwrap();

        let count = SystemLogRepo::count(&db).unwrap();
        assert_eq!(count, 2);

        SystemLogRepo::clear(&db).unwrap();
        let count_after = SystemLogRepo::count(&db).unwrap();
        assert_eq!(count_after, 0);
    }
}
