use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

/// 错误分类（业务/通用/自定义）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCategory {
    /// 业务逻辑错误
    Biz,
    /// 通用错误
    Common,
    /// 自定义错误
    Custom,
}

impl std::fmt::Display for ErrorCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorCategory::Biz => write!(f, "业务错误"),
            ErrorCategory::Common => write!(f, "通用错误"),
            ErrorCategory::Custom => write!(f, "自定义错误"),
        }
    }
}

/// 应用错误类型
#[derive(Error, Debug)]
pub enum AppError {
    #[error("内部服务器错误: {0}")]
    Internal(#[from] anyhow::Error),

    #[error("未找到资源")]
    NotFound,

    #[error("未授权访问")]
    Unauthorized,

    #[error("禁止访问")]
    Forbidden,

    #[error("请求参数错误: {0}")]
    BadRequest(String),

    #[error("配置错误: {0}")]
    Config(#[from] config::ConfigError),

    /// 服务不可用（无可用实例）
    #[error("服务不可用: {0}")]
    ServiceUnavailable(String),

    /// 数据库错误（需要启用 mysql/postgres/sqlite 任一特性）
    #[cfg(any(feature = "mysql", feature = "postgres", feature = "sqlite"))]
    #[error("数据库错误: {0}")]
    Database(#[from] sqlx::Error),

    /// 数据库连接池未初始化错误
    #[cfg(any(feature = "mysql", feature = "postgres", feature = "sqlite"))]
    #[error("数据库连接池未初始化")]
    DatabaseNotInitialized,

    /// Redis 错误（需要启用 redis 特性）
    #[cfg(feature = "redis")]
    #[error("Redis 错误: {0}")]
    Redis(#[from] redis::RedisError),

    /// Redis 客户端未初始化错误
    #[cfg(feature = "redis")]
    #[error("Redis 客户端未初始化")]
    RedisNotInitialized,

    /// 分类错误（合并原有的 BizError / CommonError / CustomerError）
    ///
    /// # 参数
    /// - `0`: 业务状态码
    /// - `1`: 错误消息
    /// - `2`: 错误分类
    #[error("{2}: [{0}] {1}")]
    Categorized(i32, String, ErrorCategory),

    /// IO 错误
    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),

    /// 地址解析错误
    #[error("地址解析错误: {0}")]
    Addr(#[from] std::net::AddrParseError),

    /// Redis 连接池错误
    #[cfg(feature = "redis")]
    #[error("Redis 连接池错误: {0}")]
    RedisPool(#[from] deadpool_redis::PoolError),

    /// Kafka 错误
    #[cfg(feature = "kafka")]
    #[error("Kafka 错误: {0}")]
    Kafka(#[from] rdkafka::error::KafkaError),
}

/// 应用结果类型
pub type AppResult<T> = Result<T, AppError>;

impl AppError {
    /// 创建业务错误（向后兼容 API）
    pub fn biz_error(status: i32, message: String) -> Self {
        Self::Categorized(status, message, ErrorCategory::Biz)
    }

    /// 创建通用错误（向后兼容 API）
    pub fn common_error(status: i32, message: String) -> Self {
        Self::Categorized(status, message, ErrorCategory::Common)
    }

    /// 创建自定义错误（向后兼容 API）
    pub fn customer_error(status: i32, message: String) -> Self {
        Self::Categorized(status, message, ErrorCategory::Custom)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::NotFound => (StatusCode::NOT_FOUND, self.to_string()),
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, self.to_string()),
            AppError::Forbidden => (StatusCode::FORBIDDEN, self.to_string()),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::Config(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            AppError::ServiceUnavailable(msg) => {
                tracing::warn!("服务不可用: {}", msg);
                (StatusCode::SERVICE_UNAVAILABLE, format!("服务不可用: {}", msg))
            }
            #[cfg(any(feature = "mysql", feature = "postgres", feature = "sqlite"))]
            AppError::Database(e) => {
                tracing::error!("数据库错误: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("数据库错误: {}", e),
                )
            }
            #[cfg(any(feature = "mysql", feature = "postgres", feature = "sqlite"))]
            AppError::DatabaseNotInitialized => {
                tracing::error!("数据库连接池未初始化");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "数据库连接池未初始化".to_string(),
                )
            }
            #[cfg(feature = "redis")]
            AppError::Redis(e) => {
                tracing::error!("Redis 错误: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Redis 错误: {}", e),
                )
            }
            #[cfg(feature = "redis")]
            AppError::RedisNotInitialized => {
                tracing::error!("Redis 客户端未初始化");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Redis 客户端未初始化".to_string(),
                )
            }
            AppError::Internal(e) => {
                tracing::error!("内部错误: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "内部服务器错误".to_string(),
                )
            }
            AppError::Categorized(status, msg, category) => {
                let display_msg = format!("{}: [{}] {}", category, status, msg);
                tracing::warn!("{}", display_msg);
                (StatusCode::OK, display_msg)
            }
            AppError::Io(e) => {
                tracing::error!("IO 错误: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, format!("IO 错误: {}", e))
            }
            AppError::Addr(e) => {
                tracing::error!("地址解析错误: {:?}", e);
                (StatusCode::BAD_REQUEST, format!("地址解析错误: {}", e))
            }
            #[cfg(feature = "redis")]
            AppError::RedisPool(e) => {
                tracing::error!("Redis 连接池错误: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Redis 连接池错误: {}", e),
                )
            }
            #[cfg(feature = "kafka")]
            AppError::Kafka(e) => {
                tracing::error!("Kafka 错误: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Kafka 错误: {}", e),
                )
            }
        };

        let body = Json(json!({
            "error": error_message,
            "status": status.as_u16(),
        }));

        (status, body).into_response()
    }
}
