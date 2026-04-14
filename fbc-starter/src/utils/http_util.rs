use crate::constants::{HEADER_KEY_TENANT_ID, HEADER_KEY_UID};
use crate::error::{AppError, AppResult};
use axum::http::HeaderMap;

/// 从请求头中获取用户 ID
pub fn get_uid_from_headers(headers: &HeaderMap) -> AppResult<i64> {
    let uid_str = headers
        .get(HEADER_KEY_UID)
        .and_then(|v| v.to_str().ok())
        .ok_or(AppError::Unauthorized)?;

    uid_str
        .parse::<i64>()
        .map_err(|_| AppError::BadRequest("无效的用户ID".to_string()))
}

/// 从请求头中获取租户 ID
pub fn get_tanent_id_from_headers(headers: &HeaderMap) -> AppResult<i64> {
    let tenant_id_str = headers
        .get(HEADER_KEY_TENANT_ID)
        .and_then(|v| v.to_str().ok())
        .ok_or(AppError::Unauthorized)?;

    tenant_id_str
        .parse::<i64>()
        .map_err(|_| AppError::BadRequest("无效的租户ID".to_string()))
}
