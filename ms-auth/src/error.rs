use axum::{Json, http::StatusCode, response::{IntoResponse, Response}};
use fbc_starter::R;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AuthError {
    /// 未授权（401）- 用户名密码错误、token 无效等
    #[error("未授权: {0}")]
    Unauthorized(String),
    /// 禁止访问（403）- 无权限、账户被禁用等
    #[error("禁止访问: {0}")]
    Forbidden(String),
    /// 请求参数错误（400）- 参数缺失、格式错误等
    #[error("请求参数错误: {0}")]
    BadRequest(String),
    /// 请求过于频繁（429）- 验证码/登录频率限制
    #[error("请求过于频繁: {0}")]
    TooManyRequests(String),
    /// 下游服务不可用（502）- identity/其他微服务调用失败
    #[error("服务调用失败: {0}")]
    ServiceUnavailable(String),
    /// 服务内部错误（500）- 意外的内部异常
    #[error("服务内部错误: {0}")]
    InternalError(String),
}

impl From<::sa_token_core::SaTokenError> for AuthError {
    fn from(err: ::sa_token_core::SaTokenError) -> Self {
        match err {
            ::sa_token_core::SaTokenError::NotLogin => {
                AuthError::Unauthorized("用户未登录".to_string())
            }
            ::sa_token_core::SaTokenError::PermissionDenied
            | ::sa_token_core::SaTokenError::PermissionDeniedDetail(_) => {
                AuthError::Forbidden("权限不足".to_string())
            }
            ::sa_token_core::SaTokenError::RoleDenied(_) => {
                AuthError::Forbidden("角色权限不足".to_string())
            }
            _ => AuthError::InternalError(format!("认证异常: {}", err)),
        }
    }
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, code, message) = match &self {
            AuthError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, 401, msg.clone()),
            AuthError::Forbidden(msg) => (StatusCode::FORBIDDEN, 403, msg.clone()),
            AuthError::BadRequest(msg) => (StatusCode::BAD_REQUEST, 400, msg.clone()),
            AuthError::TooManyRequests(msg) => (StatusCode::TOO_MANY_REQUESTS, 429, msg.clone()),
            AuthError::ServiceUnavailable(msg) => (StatusCode::BAD_GATEWAY, 502, msg.clone()),
            AuthError::InternalError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, 500, msg.clone()),
        };

        // 记录错误日志（4xx 用 warn，5xx 用 error）
        match status.as_u16() {
            400..=499 => tracing::warn!("⚠️ 认证错误 [{}]: {}", code, message),
            _ => tracing::error!("❌ 认证错误 [{}]: {}", code, message),
        }

        let body = Json(R::<String>::fail_with_code(code, message));

        (status, body).into_response()
    }
}