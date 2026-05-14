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
