use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};
use serde::{Deserialize, Serialize};

/// 当前登录用户信息
/// 从请求头中提取（由网关或认证中间件注入）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrentUser {
    /// 用户ID
    pub user_id: i64,
    /// 租户ID
    pub tenant_id: i64,
    /// 组织ID（可选，如果请求中指定了组织）
    pub org_id: Option<i64>,
    /// 员工ID（可选）
    pub employee_id: Option<i64>,
}

impl<S> FromRequestParts<S> for CurrentUser
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // 从请求头中提取用户信息（通常由网关或认证中间件注入）
        let user_id = parts
            .headers
            .get("X-User-Id")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<i64>().ok())
            .ok_or((
                StatusCode::UNAUTHORIZED,
                "Missing or invalid X-User-Id header",
            ))?;

        let tenant_id = parts
            .headers
            .get("X-Tenant-Id")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<i64>().ok())
            .ok_or((
                StatusCode::UNAUTHORIZED,
                "Missing or invalid X-Tenant-Id header",
            ))?;

        let org_id = parts
            .headers
            .get("X-Org-Id")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<i64>().ok());

        let employee_id = parts
            .headers
            .get("X-Employee-Id")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<i64>().ok());

        Ok(CurrentUser {
            user_id,
            tenant_id,
            org_id,
            employee_id,
        })
    }
}
