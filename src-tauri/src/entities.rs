use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrintJob {
    pub id: i64,
    pub name: String,
    pub status: String,
    pub printer: String,
    pub print_type: String,
    pub source: String,
    pub copies: i32,
    pub file_path: String,
    pub file_size: i64,
    pub error_msg: String,
    pub created_at: String,
    pub finished_at: Option<String>,
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemLog {
    pub id: i64,
    pub timestamp: String,
    pub level: String,
    pub category: String,
    pub message: String,
    pub logger: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LogQuery {
    pub level: Option<String>,
    pub category: Option<String>,
    pub keyword: Option<String>,
    pub limit: Option<i64>,
}
