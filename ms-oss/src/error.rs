use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use fbc_starter::R;
use thiserror::Error;

/// OSS 业务错误码（45xx 段）
pub mod code {
    /// 文件不存在
    pub const FILE_NOT_FOUND: i32 = 4501;
    /// 文件大小不匹配
    pub const FILE_SIZE_MISMATCH: i32 = 4502;
    /// 预签名失败
    pub const PRESIGN_FAILED: i32 = 4503;
    /// 上传回调失败
    pub const CALLBACK_FAILED: i32 = 4504;
    /// Bucket 不存在
    pub const BUCKET_NOT_FOUND: i32 = 4505;
    /// Provider 不支持
    pub const UNSUPPORTED_PROVIDER: i32 = 4506;
    /// 参数错误
    pub const BAD_REQUEST: i32 = 4507;
    /// 文件类型不允许
    pub const FILE_TYPE_NOT_ALLOWED: i32 = 4508;
    /// 文件大小超出限制
    pub const FILE_TOO_LARGE: i32 = 4509;
    /// x-oss-process 解析失败
    pub const PROCESS_PARSE_ERROR: i32 = 4510;
    /// Style 不存在
    pub const STYLE_NOT_FOUND: i32 = 4511;
    /// 分享链接无效或过期
    pub const SHARE_INVALID: i32 = 4512;
    /// 分片上传操作失败
    pub const MULTIPART_ERROR: i32 = 4513;
    /// 内部错误（使用通用 5xxx 段）
    pub const INTERNAL_ERROR: i32 = 5001;
}

#[derive(Debug, Error)]
pub enum OssError {
    /// 文件不存在（404）
    #[error("{0}")]
    FileNotFound(String),

    /// 文件大小不匹配
    #[error("{0}")]
    FileSizeMismatch(String),

    /// 预签名失败
    #[error("{0}")]
    PresignFailed(String),

    /// 上传回调失败
    #[error("{0}")]
    CallbackFailed(String),

    /// 参数错误
    #[error("{0}")]
    BadRequest(String),

    /// 文件类型不允许
    #[error("{0}")]
    FileTypeNotAllowed(String),

    /// 文件大小超出限制
    #[error("{0}")]
    FileTooLarge(String),

    /// x-oss-process 解析失败
    #[error("{0}")]
    ProcessParseError(String),

    /// Style 不存在
    #[error("{0}")]
    StyleNotFound(String),

    /// 分享链接无效或过期
    #[error("{0}")]
    ShareInvalid(String),

    /// 分片操作失败
    #[error("{0}")]
    MultipartError(String),

    /// 内部错误
    #[error("{0}")]
    InternalError(String),
}

impl From<anyhow::Error> for OssError {
    fn from(err: anyhow::Error) -> Self {
        OssError::InternalError(err.to_string())
    }
}

impl IntoResponse for OssError {
    fn into_response(self) -> Response {
        let (status, err_code, message) = match &self {
            OssError::FileNotFound(msg) => {
                (StatusCode::NOT_FOUND, code::FILE_NOT_FOUND, msg.clone())
            }
            OssError::FileSizeMismatch(msg) => (
                StatusCode::BAD_REQUEST,
                code::FILE_SIZE_MISMATCH,
                msg.clone(),
            ),
            OssError::PresignFailed(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                code::PRESIGN_FAILED,
                msg.clone(),
            ),
            OssError::CallbackFailed(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                code::CALLBACK_FAILED,
                msg.clone(),
            ),
            OssError::BadRequest(msg) => (StatusCode::BAD_REQUEST, code::BAD_REQUEST, msg.clone()),
            OssError::FileTypeNotAllowed(msg) => (
                StatusCode::BAD_REQUEST,
                code::FILE_TYPE_NOT_ALLOWED,
                msg.clone(),
            ),
            OssError::FileTooLarge(msg) => {
                (StatusCode::BAD_REQUEST, code::FILE_TOO_LARGE, msg.clone())
            }
            OssError::ProcessParseError(msg) => (
                StatusCode::BAD_REQUEST,
                code::PROCESS_PARSE_ERROR,
                msg.clone(),
            ),
            OssError::StyleNotFound(msg) => {
                (StatusCode::NOT_FOUND, code::STYLE_NOT_FOUND, msg.clone())
            }
            OssError::ShareInvalid(msg) => {
                (StatusCode::FORBIDDEN, code::SHARE_INVALID, msg.clone())
            }
            OssError::MultipartError(msg) => {
                (StatusCode::BAD_REQUEST, code::MULTIPART_ERROR, msg.clone())
            }
            OssError::InternalError(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                code::INTERNAL_ERROR,
                msg.clone(),
            ),
        };

        match status.as_u16() {
            400..=499 => tracing::warn!("⚠️ OSS 错误 [{}]: {}", err_code, message),
            _ => tracing::error!("❌ OSS 错误 [{}]: {}", err_code, message),
        }

        let body = Json(R::<String>::fail_with_code(err_code, message));
        (status, body).into_response()
    }
}
