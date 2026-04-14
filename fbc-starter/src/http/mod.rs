/// HTTP 相关模块
///
/// 包含路由、处理器和中间件
pub mod handlers;
pub mod middleware;

// 重新导出常用类型
pub use handlers::{health_check, root};
pub use middleware::{create_cors_layer, request_logging_middleware, grpc_log_request, grpc_log_response};
