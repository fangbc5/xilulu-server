use fbc_starter::AppError;
use thiserror::Error;

/// 媒体处理服务错误枚举
#[derive(Debug, Error)]
pub enum MediaError {
    #[error("数据库错误: {0}")]
    DatabaseFailed(String),

    #[error("S3 错误: {0}")]
    S3Failed(String),

    #[error("FFmpeg 错误: {0}")]
    FFmpegFailed(String),

    #[error("消息发送错误: {0}")]
    MessageFailed(String),

    #[error("任务锁定失败: {0}")]
    LockFailed(String),

    #[error("不支持的任务类型: {0}")]
    UnsupportedTaskType(String),

    #[error("内部处理错误: {0}")]
    InternalError(String),
}

impl From<MediaError> for AppError {
    fn from(err: MediaError) -> Self {
        match err {
            MediaError::UnsupportedTaskType(msg) => AppError::biz_error(4601, msg),
            MediaError::LockFailed(msg) => AppError::biz_error(4602, msg),
            _ => AppError::common_error(5001, err.to_string()),
        }
    }
}
