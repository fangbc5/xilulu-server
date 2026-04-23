use std::sync::Arc;
use axum::routing::{get, post, put};
use axum::Router;
use crate::state::OssState;
use crate::modules::file::handler;

/// 创建路由
///
/// 对标阿里云 OSS RESTful API 风格：
/// - 签名服务：POST /oss/signature
/// - 长效分享：GET /oss/share/{token}
/// - 对象操作：PUT/POST/GET/HEAD/DELETE /oss/{bucket}/*key
pub fn create_router(state: Arc<OssState>) -> Router {
    Router::new()
        // ---- 签名服务 ----
        .route("/oss/signature", post(handler::create_signature))
        // ---- 长效分享 302 入口 ----
        .route("/oss/share/{token}", get(handler::share_redirect))
        // ---- 对象操作（RESTful 核心）----
        .route(
            "/oss/{bucket}/{*key}",
            put(handler::put_object)
                .post(handler::post_object)
                .get(handler::get_object)
                .head(handler::head_object)
                .delete(handler::delete_object),
        )
        .with_state(state)
}
