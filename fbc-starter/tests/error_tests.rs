//! 错误类型测试
//!
//! 验证 AppError 的各种变体正确映射 HTTP 状态码和响应体

use axum::http::StatusCode;
use axum::response::IntoResponse;
use fbc_starter::error::{AppError, ErrorCategory};

/// 测试 ErrorCategory Display 实现
#[test]
fn test_error_category_display() {
    assert_eq!(ErrorCategory::Biz.to_string(), "业务错误");
    assert_eq!(ErrorCategory::Common.to_string(), "通用错误");
    assert_eq!(ErrorCategory::Custom.to_string(), "自定义错误");
}

/// 测试 ErrorCategory 相等性
#[test]
fn test_error_category_equality() {
    assert_eq!(ErrorCategory::Biz, ErrorCategory::Biz);
    assert_ne!(ErrorCategory::Biz, ErrorCategory::Common);
}

/// 测试工厂方法创建 Categorized 错误
#[test]
fn test_biz_error_factory() {
    let err = AppError::biz_error(1001, "用户不存在".to_string());
    let msg = err.to_string();
    assert!(msg.contains("1001"));
    assert!(msg.contains("用户不存在"));
    assert!(msg.contains("业务错误"));
}

#[test]
fn test_common_error_factory() {
    let err = AppError::common_error(5000, "系统繁忙".to_string());
    let msg = err.to_string();
    assert!(msg.contains("5000"));
    assert!(msg.contains("系统繁忙"));
    assert!(msg.contains("通用错误"));
}

#[test]
fn test_customer_error_factory() {
    let err = AppError::customer_error(9999, "自定义".to_string());
    let msg = err.to_string();
    assert!(msg.contains("9999"));
    assert!(msg.contains("自定义"));
}

/// 测试 Categorized 错误返回 HTTP 200 OK
#[tokio::test]
async fn test_categorized_error_returns_ok_status() {
    let err = AppError::biz_error(1001, "test".to_string());
    let response = err.into_response();
    assert_eq!(response.status(), StatusCode::OK);
}

/// 测试 NotFound 返回 404
#[tokio::test]
async fn test_not_found_error() {
    let err = AppError::NotFound;
    let response = err.into_response();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

/// 测试 Unauthorized 返回 401
#[tokio::test]
async fn test_unauthorized_error() {
    let err = AppError::Unauthorized;
    let response = err.into_response();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

/// 测试 Forbidden 返回 403
#[tokio::test]
async fn test_forbidden_error() {
    let err = AppError::Forbidden;
    let response = err.into_response();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

/// 测试 BadRequest 返回 400
#[tokio::test]
async fn test_bad_request_error() {
    let err = AppError::BadRequest("参数错误".to_string());
    let response = err.into_response();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

/// 测试 ServiceUnavailable 返回 503
#[tokio::test]
async fn test_service_unavailable_error() {
    let err = AppError::ServiceUnavailable("ms-user".to_string());
    let response = err.into_response();
    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
}

/// 测试 Internal 返回 500
#[tokio::test]
async fn test_internal_error() {
    let err = AppError::Internal(anyhow::anyhow!("内部错误"));
    let response = err.into_response();
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

/// 测试 IO 错误返回 500
#[tokio::test]
async fn test_io_error() {
    let err = AppError::Io(std::io::Error::new(std::io::ErrorKind::NotFound, "file not found"));
    let response = err.into_response();
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

/// 测试地址解析错误返回 400
#[tokio::test]
async fn test_addr_parse_error() {
    let err: AppError = "not_a_valid_addr".parse::<std::net::SocketAddr>()
        .unwrap_err()
        .into();
    let response = err.into_response();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
