use axum::{extract::State, response::Json};
use chrono::Local;
use serde_json::{json, Value};
use std::sync::Arc;

use crate::state::AppState;

/// 根路径处理器
pub async fn root() -> Json<Value> {
    Json(json!({
        "message": "欢迎使用 Web 服务器",
        "status": "running"
    }))
}

/// 健康检查处理器
pub async fn health_check(State(state): State<Arc<AppState>>) -> Json<Value> {
    let uptime = Local::now()
        .signed_duration_since(state.start_time)
        .num_seconds();

    Json(json!({
        "status": "ok",
        "uptime_seconds": uptime
    }))
}
