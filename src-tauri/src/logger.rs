use crate::db::AppState;
use crate::repos::SystemLogRepo;

pub fn log_debug(state: &AppState, category: &str, source: &str, message: &str) {
    let _ = SystemLogRepo::insert(state.db(), "DEBUG", category, message, source);
}

pub fn log_info(state: &AppState, category: &str, source: &str, message: &str) {
    let _ = SystemLogRepo::insert(state.db(), "INFO", category, message, source);
}

pub fn log_warn(state: &AppState, category: &str, source: &str, message: &str) {
    let _ = SystemLogRepo::insert(state.db(), "WARN", category, message, source);
}

pub fn log_error(state: &AppState, category: &str, source: &str, message: &str) {
    let _ = SystemLogRepo::insert(state.db(), "ERROR", category, message, source);
}
