use axum::{
    extract::{FromRequestParts, OptionalFromRequestParts, Request},
    http::{request::Parts, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};

/// 请求上下文（网关透传的用户信息）
///
/// 网关验证 JWT 后，将用户信息通过 HTTP 头透传到下游服务：
/// - `X-User-Id`: 用户 ID
/// - `X-Tenant-Id`: 租户 ID（可选）
/// - `X-Username`: 用户名
#[derive(Debug, Clone)]
pub struct RequestContext {
    /// 用户 ID
    pub user_id: i64,
    /// 租户 ID（可选，无租户时为 None）
    pub tenant_id: Option<i64>,
    /// 用户名
    pub username: String,
}

/// 中间件：从网关透传的 X-User-* Header 中解析用户上下文
///
/// 将解析后的 `RequestContext` 注入到请求扩展中，
/// 下游 handler 可通过 Axum extractor 直接使用。
///
/// 注意：如果请求头中没有 X-User-Id（白名单接口），
/// 中间件仍然放行但不注入上下文。Handler 需要自行判断。
pub async fn user_context_middleware(request: Request, next: Next) -> Response {
    let mut request = request;

    // 尝试从 header 解析用户上下文
    let user_id = request
        .headers()
        .get("X-User-Id")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<i64>().ok());

    if let Some(user_id) = user_id {
        let tenant_id = request
            .headers()
            .get("X-Tenant-Id")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<i64>().ok());

        let username = request
            .headers()
            .get("X-Username")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();

        let ctx = RequestContext {
            user_id,
            tenant_id,
            username,
        };

        request.extensions_mut().insert(ctx);
    }

    next.run(request).await
}

/// Axum Extractor：从请求扩展中提取 RequestContext（必须认证）
///
/// 使用方式：
/// ```rust
/// async fn handler(ctx: RequestContext) -> impl IntoResponse {
///     format!("Hello, user {}", ctx.user_id)
/// }
/// ```
///
/// 如果请求未经网关认证（无 X-User-Id header），返回 401。
impl<S> FromRequestParts<S> for RequestContext
where
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<RequestContext>()
            .cloned()
            .ok_or_else(|| (StatusCode::UNAUTHORIZED, "Missing user context").into_response())
    }
}

/// Axum 0.8 可选提取器：支持 `Option<RequestContext>` 参数
///
/// 使用方式：
/// ```rust
/// async fn handler(ctx: Option<RequestContext>) -> impl IntoResponse {
///     match ctx {
///         Some(c) => format!("Hello, user {}", c.user_id),
///         None => "Hello, anonymous".to_string(),
///     }
/// }
/// ```
///
/// 当请求未附带用户上下文时返回 `Ok(None)`，不会触发 401 错误。
impl<S> OptionalFromRequestParts<S> for RequestContext
where
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> Result<Option<Self>, Self::Rejection> {
        Ok(parts.extensions.get::<RequestContext>().cloned())
    }
}

