use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use fbc_starter::R;
use thiserror::Error;

/// Content 业务错误码（47xx 段）
pub mod code {
    /// 内容不存在
    pub const CONTENT_NOT_FOUND: i32 = 4701;
    /// 内容类型不存在或已禁用
    pub const SCHEMA_NOT_FOUND: i32 = 4702;
    /// ext_data 校验失败
    pub const EXT_DATA_INVALID: i32 = 4703;
    /// 参数错误
    pub const BAD_REQUEST: i32 = 4707;
    /// Block DSL 格式错误
    pub const BLOCK_DSL_INVALID: i32 = 4708;
    /// 关系深度超限
    pub const RELATION_DEPTH_EXCEEDED: i32 = 4709;
    /// 关系不存在
    pub const RELATION_NOT_FOUND: i32 = 4710;
    /// 乐观锁冲突
    pub const VERSION_CONFLICT: i32 = 4711;
    /// 内部错误（使用通用 5xxx 段）
    pub const INTERNAL_ERROR: i32 = 5001;
}

/// Content 业务错误
#[derive(Debug, Error)]
pub enum ContentError {
    /// 内容不存在
    #[error("{0}")]
    ContentNotFound(String),

    /// 内容类型 Schema 不存在或已禁用
    #[error("{0}")]
    SchemaNotFound(String),

    /// ext_data 校验不通过
    #[error("{0}")]
    ExtDataInvalid(String),

    /// 参数错误
    #[error("{0}")]
    BadRequest(String),

    /// Block DSL 格式错误
    #[error("{0}")]
    BlockDslInvalid(String),

    /// 关系深度超限
    #[error("{0}")]
    RelationDepthExceeded(String),

    /// 关系不存在
    #[error("{0}")]
    RelationNotFound(String),

    /// 乐观锁冲突
    #[error("{0}")]
    VersionConflict(String),

    /// 内部错误
    #[error("{0}")]
    InternalError(String),
}

impl From<anyhow::Error> for ContentError {
    fn from(err: anyhow::Error) -> Self {
        ContentError::InternalError(err.to_string())
    }
}

impl IntoResponse for ContentError {
    fn into_response(self) -> Response {
        let (status, err_code, message) = match &self {
            ContentError::ContentNotFound(msg) => {
                (StatusCode::NOT_FOUND, code::CONTENT_NOT_FOUND, msg.clone())
            }
            ContentError::SchemaNotFound(msg) => {
                (StatusCode::BAD_REQUEST, code::SCHEMA_NOT_FOUND, msg.clone())
            }
            ContentError::ExtDataInvalid(msg) => {
                (StatusCode::BAD_REQUEST, code::EXT_DATA_INVALID, msg.clone())
            }
            ContentError::BadRequest(msg) => {
                (StatusCode::BAD_REQUEST, code::BAD_REQUEST, msg.clone())
            }
            ContentError::BlockDslInvalid(msg) => {
                (StatusCode::BAD_REQUEST, code::BLOCK_DSL_INVALID, msg.clone())
            }
            ContentError::RelationDepthExceeded(msg) => {
                (StatusCode::BAD_REQUEST, code::RELATION_DEPTH_EXCEEDED, msg.clone())
            }
            ContentError::RelationNotFound(msg) => {
                (StatusCode::NOT_FOUND, code::RELATION_NOT_FOUND, msg.clone())
            }
            ContentError::VersionConflict(msg) => {
                (StatusCode::CONFLICT, code::VERSION_CONFLICT, msg.clone())
            }
            ContentError::InternalError(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, code::INTERNAL_ERROR, msg.clone())
            }
        };

        match status.as_u16() {
            400..=499 => tracing::warn!("⚠️ Content 错误 [{}]: {}", err_code, message),
            _ => tracing::error!("❌ Content 错误 [{}]: {}", err_code, message),
        }

        let body = Json(R::<String>::fail_with_code(err_code, message));
        (status, body).into_response()
    }
}
