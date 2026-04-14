// IM 服务错误类型
// 错误码段位: 43xx

use axum::{Json, http::StatusCode, response::{IntoResponse, Response}};
use fbc_starter::R;
use thiserror::Error;

/// IM 错误码
pub mod error_code {
    // === 好友模块 430x ===
    /// 不能添加自己为好友
    pub const CANNOT_ADD_SELF: i32 = 4301;
    /// 已经是好友
    pub const ALREADY_FRIEND: i32 = 4302;
    /// 已有待处理的申请
    pub const PENDING_APPLY_EXISTS: i32 = 4303;
    /// 申请不存在
    pub const APPLY_NOT_FOUND: i32 = 4304;
    /// 申请已处理
    pub const APPLY_ALREADY_HANDLED: i32 = 4305;
    /// 无权操作
    pub const PERMISSION_DENIED: i32 = 4306;

    // === 群组模块 431x ===
    /// 群不存在
    pub const GROUP_NOT_FOUND: i32 = 4310;
    /// 用户已在群中
    pub const ALREADY_IN_GROUP: i32 = 4311;
    /// 不在群中
    pub const NOT_IN_GROUP: i32 = 4312;
    /// 群主不能退出
    pub const OWNER_CANNOT_QUIT: i32 = 4313;
    /// 不能移除自己
    pub const CANNOT_REMOVE_SELF: i32 = 4314;
    /// 群名不能为空
    pub const GROUP_NAME_EMPTY: i32 = 4315;
    /// 群已解散
    pub const GROUP_DISSOLVED: i32 = 4316;
    /// 群名过长
    pub const GROUP_NAME_TOO_LONG: i32 = 4317;
    /// 不能转让给自己
    pub const CANNOT_TRANSFER_TO_SELF: i32 = 4318;

    // === 会话模块 432x ===
    /// 会话不存在
    pub const CONTACT_NOT_FOUND: i32 = 4320;

    // === 消息模块 433x ===
    /// 消息不存在
    pub const MESSAGE_NOT_FOUND: i32 = 4330;
    /// 消息已撤回
    pub const MESSAGE_ALREADY_RECALLED: i32 = 4331;

    // === 系统级 5xxx ===
    /// 数据库错误
    pub const DATABASE_ERROR: i32 = 5001;
    /// RPC 调用失败
    pub const RPC_ERROR: i32 = 5002;
}

/// IM 服务错误类型
#[derive(Debug, Error)]
pub enum ImError {
    // === 好友模块 ===
    /// 不能添加自己为好友
    #[error("不能添加自己为好友")]
    CannotAddSelf,
    /// 已经是好友
    #[error("已经是好友了")]
    AlreadyFriend,
    /// 已有待处理的好友申请
    #[error("已有待处理的好友申请")]
    PendingApplyExists,
    /// 申请不存在
    #[error("申请不存在")]
    ApplyNotFound,
    /// 申请已处理
    #[error("申请已处理")]
    ApplyAlreadyHandled,
    /// 无权操作
    #[error("无权操作: {0}")]
    PermissionDenied(String),

    // === 群组模块 ===
    /// 群不存在
    #[error("群不存在")]
    GroupNotFound,
    /// 用户已在群中
    #[error("该用户已在群中")]
    AlreadyInGroup,
    /// 不在群中
    #[error("你不在该群中")]
    NotInGroup,
    /// 群主不能退出
    #[error("群主不能退出群聊，请先转让群主")]
    OwnerCannotQuit,
    /// 不能移除自己
    #[error("不能移除自己，请使用退出群聊")]
    CannotRemoveSelf,
    /// 群名不能为空
    #[error("群名不能为空")]
    GroupNameEmpty,
    /// 群已解散
    #[error("该群已解散")]
    #[allow(dead_code)]
    GroupDissolved,
    /// 群名过长
    #[error("群名不能超过32个字符")]
    GroupNameTooLong,
    /// 不能转让给自己
    #[error("不能转让群主给自己")]
    CannotTransferToSelf,

    // === 会话模块 ===
    /// 会话不存在
    #[error("会话不存在")]
    ContactNotFound,

    // === 消息模块 ===
    /// 消息不存在
    #[error("消息不存在")]
    MessageNotFound,
    /// 消息已撤回
    #[error("消息已撤回")]
    MessageAlreadyRecalled,

    // === 系统级 ===
    /// 数据库错误
    #[error("数据库错误: {0}")]
    DatabaseError(String),
    /// RPC 调用失败
    #[error("RPC 调用失败: {0}")]
    RpcError(String),
    /// 参数无效
    #[error("参数无效: {0}")]
    InvalidParam(String),
    /// 系统异常
    #[error("系统异常: {0}")]
    SystemError(String),
}

impl IntoResponse for ImError {
    fn into_response(self) -> Response {
        use error_code::*;
        let (status, code, message) = match &self {
            ImError::CannotAddSelf => (StatusCode::BAD_REQUEST, CANNOT_ADD_SELF, self.to_string()),
            ImError::AlreadyFriend => (StatusCode::CONFLICT, ALREADY_FRIEND, self.to_string()),
            ImError::PendingApplyExists => (StatusCode::CONFLICT, PENDING_APPLY_EXISTS, self.to_string()),
            ImError::ApplyNotFound => (StatusCode::NOT_FOUND, APPLY_NOT_FOUND, self.to_string()),
            ImError::ApplyAlreadyHandled => (StatusCode::CONFLICT, APPLY_ALREADY_HANDLED, self.to_string()),
            ImError::PermissionDenied(_) => (StatusCode::FORBIDDEN, PERMISSION_DENIED, self.to_string()),
            ImError::GroupNotFound => (StatusCode::NOT_FOUND, GROUP_NOT_FOUND, self.to_string()),
            ImError::AlreadyInGroup => (StatusCode::CONFLICT, ALREADY_IN_GROUP, self.to_string()),
            ImError::NotInGroup => (StatusCode::NOT_FOUND, NOT_IN_GROUP, self.to_string()),
            ImError::OwnerCannotQuit => (StatusCode::FORBIDDEN, OWNER_CANNOT_QUIT, self.to_string()),
            ImError::CannotRemoveSelf => (StatusCode::BAD_REQUEST, CANNOT_REMOVE_SELF, self.to_string()),
            ImError::GroupNameEmpty => (StatusCode::BAD_REQUEST, GROUP_NAME_EMPTY, self.to_string()),
            ImError::GroupDissolved => (StatusCode::GONE, GROUP_DISSOLVED, self.to_string()),
            ImError::GroupNameTooLong => (StatusCode::BAD_REQUEST, GROUP_NAME_TOO_LONG, self.to_string()),
            ImError::CannotTransferToSelf => (StatusCode::BAD_REQUEST, CANNOT_TRANSFER_TO_SELF, self.to_string()),
            ImError::ContactNotFound => (StatusCode::NOT_FOUND, CONTACT_NOT_FOUND, self.to_string()),
            ImError::MessageNotFound => (StatusCode::NOT_FOUND, MESSAGE_NOT_FOUND, self.to_string()),
            ImError::MessageAlreadyRecalled => (StatusCode::CONFLICT, MESSAGE_ALREADY_RECALLED, self.to_string()),
            ImError::DatabaseError(_) => (StatusCode::INTERNAL_SERVER_ERROR, DATABASE_ERROR, self.to_string()),
            ImError::RpcError(_) => (StatusCode::INTERNAL_SERVER_ERROR, RPC_ERROR, self.to_string()),
            ImError::InvalidParam(_) => (StatusCode::BAD_REQUEST, 4000, self.to_string()),
            ImError::SystemError(_) => (StatusCode::INTERNAL_SERVER_ERROR, 5000, self.to_string()),
        };

        // 记录日志
        match status.as_u16() {
            400..=499 => tracing::warn!("⚠️ IM 错误 [{}]: {}", code, message),
            _ => tracing::error!("❌ IM 错误 [{}]: {}", code, message),
        }

        let body = Json(R::<String>::fail_with_code(code, message));
        (status, body).into_response()
    }
}

/// 从 sqlx::Error 转换
impl From<sqlx::Error> for ImError {
    fn from(err: sqlx::Error) -> Self {
        ImError::DatabaseError(err.to_string())
    }
}

/// 从 sqlxplus::SqlxPlusError 转换
impl From<sqlxplus::SqlxPlusError> for ImError {
    fn from(err: sqlxplus::SqlxPlusError) -> Self {
        ImError::DatabaseError(err.to_string())
    }
}

/// IM 服务结果类型
#[allow(dead_code)]
pub type ImResult<T> = Result<T, ImError>;
