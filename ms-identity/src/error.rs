/// 身份服务错误码
pub mod error_code {
    /// 用户不存在
    pub const USER_NOT_FOUND: i32 = 4001;
    /// 密码错误
    pub const PASSWORD_ERROR: i32 = 4002;
    /// 用户已禁用
    pub const USER_DISABLED: i32 = 4003;
    /// 租户不存在
    pub const TENANT_NOT_FOUND: i32 = 4006;
    /// 角色不存在
    pub const ROLE_NOT_FOUND: i32 = 4008;
    /// 资源不存在
    pub const RESOURCE_NOT_FOUND: i32 = 4009;

    /// 用户名已存在
    pub const USERNAME_EXISTS: i32 = 4011;
    /// 邮箱已存在
    pub const EMAIL_EXISTS: i32 = 4012;
    /// 手机号已存在
    pub const MOBILE_EXISTS: i32 = 4015;

    /// 用户租户关系已存在
    pub const USER_TENANT_REL_EXISTS: i32 = 4013;
    /// 用户租户关系不存在
    pub const USER_TENANT_REL_NOT_FOUND: i32 = 4014;
    /// 套餐不存在
    pub const PLAN_NOT_FOUND: i32 = 4018;
    /// 套餐已存在
    pub const PLAN_EXISTS: i32 = 4019;
    /// 套餐权益不存在
    pub const PLAN_ENTITLEMENT_NOT_FOUND: i32 = 4020;
    /// 租户订阅不存在
    pub const TENANT_SUBSCRIPTION_NOT_FOUND: i32 = 4021;

    /// 租户用量不存在
    pub const TENANT_USAGE_NOT_FOUND: i32 = 4023;
    /// 参数无效
    pub const INVALID_PARAM: i32 = 4024;
    /// 密码加密错误
    pub const PASSWORD_ENCRYPT_ERROR: i32 = 5001;
    /// Token 生成错误
    pub const TOKEN_GENERATE_ERROR: i32 = 5002;
    /// 数据库错误
    pub const DATABASE_ERROR: i32 = 5003;
    /// 业务错误
    pub const BUSINESS_ERROR: i32 = 5004;
}

use fbc_starter::R;
use thiserror::Error;

/// 身份服务错误枚举
#[derive(Debug, Error)]
pub enum IdentityError {
    /// 用户不存在
    #[error("用户不存在")]
    UserNotFound,

    /// 密码错误
    #[error("密码错误: {0}")]
    PasswordError(String),

    /// 用户已禁用
    #[error("用户已禁用")]
    UserDisabled,

    /// 租户不存在
    #[error("租户不存在")]
    TenantNotFound,

    /// 角色不存在
    #[error("角色不存在")]
    RoleNotFound,

    /// 资源不存在
    #[error("资源不存在")]
    ResourceNotFound,

    /// 应用不存在
    #[error("应用不存在")]
    ApplicationNotFound,



    /// 用户名已存在
    #[error("用户名已存在")]
    UsernameExists,

    /// 邮箱已存在
    #[error("邮箱已存在")]
    EmailExists,

    /// 密码太短
    #[error("密码长度至少需要 {0} 位")]
    PasswordTooShort(usize),

    /// 手机号已存在
    #[error("手机号已存在")]
    MobileExists,

    /// 用户租户关系已存在
    #[error("用户租户关系已存在")]
    UserTenantRelExists,

    /// 用户租户关系不存在
    #[error("用户租户关系不存在")]
    UserTenantRelNotFound,

    /// 套餐不存在
    #[error("套餐不存在")]
    PlanNotFound,

    /// 套餐已存在
    #[error("套餐已存在")]
    PlanExists,

    /// 套餐权益不存在
    #[error("套餐权益不存在")]
    PlanEntitlementNotFound,

    /// 租户订阅不存在
    #[error("租户订阅不存在")]
    TenantSubscriptionNotFound,



    /// 租户用量不存在
    #[error("租户用量不存在")]
    TenantUsageNotFound,

    /// 密码加密错误
    #[error("密码加密错误: {0}")]
    PasswordEncryptError(String),

    /// Token 生成错误
    #[error("Token 生成错误: {0}")]
    TokenGenerateError(String),

    /// 数据库错误
    #[error("数据库错误: {0}")]
    DatabaseError(String),

    /// 业务错误
    #[error("业务错误: {0}")]
    BusinessError(String),

    /// 参数无效
    #[error("{0}")]
    InvalidParam(String),
}

impl IdentityError {
    /// 获取错误码
    pub fn code(&self) -> i32 {
        use error_code::*;
        match self {
            IdentityError::UserNotFound => USER_NOT_FOUND,
            IdentityError::PasswordError(_) => PASSWORD_ERROR,
            IdentityError::UserDisabled => USER_DISABLED,
            IdentityError::TenantNotFound => TENANT_NOT_FOUND,
            IdentityError::RoleNotFound => ROLE_NOT_FOUND,
            IdentityError::ResourceNotFound => RESOURCE_NOT_FOUND,
            IdentityError::ApplicationNotFound => RESOURCE_NOT_FOUND,

            IdentityError::UsernameExists => USERNAME_EXISTS,
            IdentityError::EmailExists => EMAIL_EXISTS,
            IdentityError::MobileExists => MOBILE_EXISTS,
            IdentityError::PasswordTooShort(_) => PASSWORD_ERROR,
            IdentityError::UserTenantRelExists => USER_TENANT_REL_EXISTS,
            IdentityError::UserTenantRelNotFound => USER_TENANT_REL_NOT_FOUND,
            IdentityError::PlanNotFound => PLAN_NOT_FOUND,
            IdentityError::PlanExists => PLAN_EXISTS,
            IdentityError::PlanEntitlementNotFound => PLAN_ENTITLEMENT_NOT_FOUND,
            IdentityError::TenantSubscriptionNotFound => TENANT_SUBSCRIPTION_NOT_FOUND,

            IdentityError::TenantUsageNotFound => TENANT_USAGE_NOT_FOUND,
            IdentityError::PasswordEncryptError(_) => PASSWORD_ENCRYPT_ERROR,
            IdentityError::TokenGenerateError(_) => TOKEN_GENERATE_ERROR,
            IdentityError::DatabaseError(_) => DATABASE_ERROR,
            IdentityError::BusinessError(_) => BUSINESS_ERROR,
            IdentityError::InvalidParam(_) => INVALID_PARAM,
        }
    }
}

/// 从数据库错误转换
impl From<sqlx::Error> for IdentityError {
    fn from(err: sqlx::Error) -> Self {
        IdentityError::DatabaseError(err.to_string())
    }
}

/// 从 argon2 错误转换
impl From<argon2::Error> for IdentityError {
    fn from(err: argon2::Error) -> Self {
        IdentityError::PasswordEncryptError(err.to_string())
    }
}

/// 从 jsonwebtoken 错误转换
impl From<jsonwebtoken::errors::Error> for IdentityError {
    fn from(err: jsonwebtoken::errors::Error) -> Self {
        IdentityError::TokenGenerateError(err.to_string())
    }
}

/// 从 anyhow::Error 转换（尝试 downcast，失败则包装为 BusinessError）
impl IdentityError {
    pub fn from_anyhow(err: anyhow::Error) -> Self {
        match err.downcast::<IdentityError>() {
            Ok(identity_err) => identity_err,
            Err(other) => IdentityError::BusinessError(other.to_string()),
        }
    }
}

/// anyhow::Error → IdentityError 快捷转换（供 handler 层 .map_err(to_err) 使用）
pub fn to_err(e: anyhow::Error) -> IdentityError {
    IdentityError::from_anyhow(e)
}

// ==================== IntoResponse ====================

use axum::{Json, http::StatusCode, response::{IntoResponse, Response}};

impl IntoResponse for IdentityError {
    fn into_response(self) -> Response {
        let status = match &self {
            // 4xx
            IdentityError::UserNotFound
            | IdentityError::TenantNotFound
            | IdentityError::RoleNotFound
            | IdentityError::ResourceNotFound
            | IdentityError::ApplicationNotFound
            | IdentityError::UserTenantRelNotFound
            | IdentityError::PlanNotFound
            | IdentityError::PlanEntitlementNotFound
            | IdentityError::TenantSubscriptionNotFound
            | IdentityError::TenantUsageNotFound => StatusCode::NOT_FOUND,

            IdentityError::InvalidParam(_)
            | IdentityError::PasswordTooShort(_) => StatusCode::BAD_REQUEST,
         

            IdentityError::PasswordError(_) => StatusCode::UNAUTHORIZED,

            IdentityError::UserDisabled => StatusCode::FORBIDDEN,

            IdentityError::UsernameExists
            | IdentityError::EmailExists
            | IdentityError::MobileExists
            | IdentityError::UserTenantRelExists
            | IdentityError::PlanExists => StatusCode::CONFLICT,

            // 5xx
            IdentityError::PasswordEncryptError(_)
            | IdentityError::TokenGenerateError(_)
            | IdentityError::DatabaseError(_)
            | IdentityError::BusinessError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let code = self.code();
        let message = self.to_string();

        match status.as_u16() {
            400..=499 => tracing::warn!("⚠️ Identity 错误 [{}]: {}", code, message),
            _ => tracing::error!("❌ Identity 错误 [{}]: {}", code, message),
        }

        let body = Json(R::<String>::fail_with_code(code, message));
        (status, body).into_response()
    }
}

