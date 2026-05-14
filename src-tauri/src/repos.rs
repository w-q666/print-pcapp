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
