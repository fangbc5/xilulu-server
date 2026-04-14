use axum::http::HeaderName;
use axum::{
    extract::Request,
    http::{HeaderValue, StatusCode},
    middleware::Next,
    response::Response,
};
use std::time::Instant;
use tower_http::cors::{Any, CorsLayer};

use crate::config::Config;

// =====================================================================
// CORS 中间件
// =====================================================================

/// 创建 CORS 中间件
pub fn create_cors_layer(config: &Config) -> CorsLayer {
    let allow_credentials = config.cors.allow_credentials;

    // 当 allow_credentials 为 true 时，不能使用 * 作为 allowed_headers 或 allowed_origins
    // 需要明确指定允许的值

    let mut cors = CorsLayer::new().allow_methods(
        config
            .cors
            .allowed_methods
            .iter()
            .map(|m| m.parse().unwrap())
            .collect::<Vec<_>>(),
    );

    // 处理 allowed_headers
    if config.cors.allowed_headers.contains(&"*".to_string()) {
        if allow_credentials {
            // 当允许凭证时，使用常见的请求头列表
            cors = cors.allow_headers([
                HeaderName::from_static("content-type"),
                HeaderName::from_static("authorization"),
                HeaderName::from_static("x-requested-with"),
                HeaderName::from_static("accept"),
                HeaderName::from_static("origin"),
            ]);
        } else {
            cors = cors.allow_headers(Any);
        }
    } else {
        // 解析指定的请求头
        let headers: Result<Vec<_>, _> = config
            .cors
            .allowed_headers
            .iter()
            .map(|h| HeaderName::from_bytes(h.as_bytes()))
            .collect();
        if let Ok(headers) = headers {
            cors = cors.allow_headers(headers);
        }
    }

    // 处理 allowed_origins
    // 注意：当 allowed_origins 为 * 时，allow_credentials 必须为 false（CORS 规范要求）
    if config.cors.allowed_origins.contains(&"*".to_string()) {
        if allow_credentials {
            tracing::warn!(
                "CORS: allow_credentials=true 与 allowed_origins=* 不兼容，已自动禁用 allow_credentials"
            );
        }
        cors = cors.allow_origin(Any).allow_credentials(false);
    } else {
        let origins: Result<Vec<_>, _> = config
            .cors
            .allowed_origins
            .iter()
            .map(|o| o.parse())
            .collect();
        if let Ok(origins) = origins {
            cors = cors.allow_origin(origins.into_iter().collect::<Vec<_>>());
        }
        cors = cors.allow_credentials(allow_credentials);
    }

    cors
}

// =====================================================================
// Trace Context 工具函数（W3C traceparent 标准）
// =====================================================================

/// 从 traceparent header 解析 trace_id
/// 格式: 00-{trace_id(32hex)}-{parent_span_id(16hex)}-{flags(2hex)}
fn parse_trace_id(traceparent: &str) -> Option<String> {
    let parts: Vec<&str> = traceparent.split('-').collect();
    if parts.len() >= 3 && parts[1].len() == 32 {
        Some(parts[1].to_string())
    } else {
        None
    }
}

/// 生成 trace_id（32位 hex = 128 bit，符合 W3C 标准）
fn generate_trace_id() -> String {
    let id = uuid::Uuid::new_v4();
    id.as_simple().to_string() // 32 hex chars without hyphens
}

/// 生成 span_id（16位 hex = 64 bit，符合 W3C 标准）
fn generate_span_id() -> String {
    let id = uuid::Uuid::new_v4();
    id.as_simple().to_string()[..16].to_string()
}

/// 判断路径是否应跳过日志记录
fn should_skip_logging(path: &str) -> bool {
    matches!(path, "/" | "/health" | "/favicon.ico")
}

/// 根据耗时获取慢请求级别标签
/// 返回 (级别标签, 是否需要告警)
fn slow_request_level(duration_ms: u128) -> Option<&'static str> {
    match duration_ms {
        10_000.. => Some("🔴 CRITICAL >10s"),
        5_000..=9_999 => Some("🔴 VERY_SLOW >5s"),
        3_000..=4_999 => Some("🟠 SLOW >3s"),
        2_000..=2_999 => Some("🟠 SLOW >2s"),
        1_000..=1_999 => Some("🟡 SLOW >1s"),
        500..=999 => Some("🟡 SLOW >500ms"),
        200..=499 => Some("🟢 SLOW >200ms"),
        _ => None,
    }
}

// =====================================================================
// 生产级 HTTP 请求日志中间件
// =====================================================================

/// 生产级 HTTP 请求日志中间件
///
/// 功能：
/// - 提取/生成 W3C traceparent trace_id（分布式链路追踪）
/// - 记录请求方法、路径、状态码、耗时
/// - 多级慢请求告警（200ms/500ms/1s/2s/3s/5s/10s）
/// - 自动跳过健康检查等路径
/// - trace_id/span_id 传递到下游（通过 response header）
pub async fn request_logging_middleware(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let method = request.method().clone();
    let uri = request.uri().path().to_string();
    let version = format!("{:?}", request.version());

    // 跳过健康检查等路径
    if should_skip_logging(&uri) {
        return Ok(next.run(request).await);
    }

    // 提取或生成 trace_id（从 traceparent header）
    let trace_id = request
        .headers()
        .get("traceparent")
        .and_then(|v| v.to_str().ok())
        .and_then(parse_trace_id)
        .unwrap_or_else(generate_trace_id);

    // 取 trace_id 前 8 位用于简短显示
    let trace_short = &trace_id[..8.min(trace_id.len())];

    // 生成本服务的 span_id
    let span_id = generate_span_id();
    let span_short = &span_id[..8.min(span_id.len())];

    // 提取 user-agent（简化）
    let user_agent = request
        .headers()
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(|ua| {
            // 截取前 50 个字符
            if ua.len() > 50 {
                format!("{}...", &ua[..50])
            } else {
                ua.to_string()
            }
        })
        .unwrap_or_default();

    let start = Instant::now();

    // 记录请求开始
    tracing::info!(
        "→ {} {} {} [trace={}|span={}]",
        method,
        uri,
        version,
        trace_short,
        span_short,
    );

    // 执行请求
    let response = next.run(request).await;
    let duration = start.elapsed();
    let duration_ms = duration.as_millis();
    let status = response.status();

    // 构建 traceparent 用于传递（带本地 span_id）
    let traceparent_value = format!("00-{}-{}-01", trace_id, span_id);

    // 记录请求完成（含状态码和耗时）
    if status.is_server_error() {
        tracing::error!(
            "← {} {} {} {}ms [trace={}|span={}] ua=\"{}\"",
            status.as_u16(),
            method,
            uri,
            duration_ms,
            trace_short,
            span_short,
            user_agent,
        );
    } else if status.is_client_error() {
        tracing::warn!(
            "← {} {} {} {}ms [trace={}|span={}]",
            status.as_u16(),
            method,
            uri,
            duration_ms,
            trace_short,
            span_short,
        );
    } else {
        tracing::info!(
            "← {} {} {} {}ms [trace={}|span={}]",
            status.as_u16(),
            method,
            uri,
            duration_ms,
            trace_short,
            span_short,
        );
    }

    // 多级慢请求告警
    if let Some(level) = slow_request_level(duration_ms) {
        tracing::warn!(
            "{} {}ms {} {} [trace={}|span={}] ua=\"{}\"",
            level,
            duration_ms,
            method,
            uri,
            trace_short,
            span_short,
            user_agent,
        );
    }

    // 将 traceparent 注入响应头（方便客户端和前端调试）
    let mut response = response;
    if let Ok(v) = HeaderValue::from_str(&traceparent_value) {
        response.headers_mut().insert("traceparent", v);
    }
    if let Ok(v) = HeaderValue::from_str(trace_short) {
        response
            .headers_mut()
            .insert("x-trace-id", v);
    }

    Ok(response)
}

// =====================================================================
// gRPC 请求日志（供 server.rs 中 fallback_service 调用）
// =====================================================================

/// 记录 gRPC 请求日志（请求开始）
/// 返回 (trace_id_short, span_id_short, start_time)
pub fn grpc_log_request(req: &axum::http::Request<impl std::any::Any>) -> (String, String, Instant) {
    let path = req.uri().path().to_string();

    // 从 gRPC metadata 中提取 traceparent（HTTP/2 header）
    let trace_id = req
        .headers()
        .get("traceparent")
        .and_then(|v| v.to_str().ok())
        .and_then(parse_trace_id)
        .unwrap_or_else(generate_trace_id);

    let trace_short = trace_id[..8.min(trace_id.len())].to_string();
    let span_id = generate_span_id();
    let span_short = span_id[..8.min(span_id.len())].to_string();

    tracing::info!(
        "→ gRPC {} [trace={}|span={}]",
        path,
        trace_short,
        span_short,
    );

    (trace_short, span_short, Instant::now())
}

/// 记录 gRPC 请求日志（请求完成）
pub fn grpc_log_response(
    path: &str,
    status: StatusCode,
    trace_short: &str,
    span_short: &str,
    start: Instant,
) {
    let duration_ms = start.elapsed().as_millis();

    if status.is_success() {
        tracing::info!(
            "← gRPC {} {} {}ms [trace={}|span={}]",
            status.as_u16(),
            path,
            duration_ms,
            trace_short,
            span_short,
        );
    } else {
        tracing::warn!(
            "← gRPC {} {} {}ms [trace={}|span={}]",
            status.as_u16(),
            path,
            duration_ms,
            trace_short,
            span_short,
        );
    }

    // 多级慢请求告警
    if let Some(level) = slow_request_level(duration_ms) {
        tracing::warn!(
            "{} {}ms gRPC {} [trace={}|span={}]",
            level,
            duration_ms,
            path,
            trace_short,
            span_short,
        );
    }
}
