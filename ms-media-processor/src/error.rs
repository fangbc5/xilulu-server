use fbc_starter::AppError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MediaError {
    #[error("Database error: {0}")]
    DatabaseFailed(String),

    #[error("S3 error: {0}")]
    S3Failed(String),

    #[error("FFmpeg error: {0}")]
    FFmpegFailed(String),

    #[error("Message error: {0}")]
    MessageFailed(String),

    #[error("Task not found or locked: {0}")]
    LockFailed(String),

    #[error("Internal processing error: {0}")]
    InternalError(String),
}

impl Into<AppError> for MediaError {
    fn into(self) -> AppError {
        AppError::Internal(anyhow::anyhow!(self.to_string()))
    }
}
