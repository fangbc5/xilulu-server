use std::sync::Arc;
use axum::routing::{get, post, put};
use axum::{Json, Router};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::modules::file::handler;
use crate::modules::file::model::dto::*;
use crate::state::OssState;

/// OpenAPI 文档定义
///
/// 自动收集所有标注了 `#[utoipa::path]` 的 handler
#[derive(OpenApi)]
#[openapi(
    info(
        title = "ms-oss 统一文件中台",
        version = "0.1.0",
        description = "Xilulu 微服务生态的对象存储统一中台服务。\n\n提供预签名上传/下载、分片上传、图片实时处理、视频截帧分发、长效分享链接等能力。\n\n对标阿里云 OSS RESTful API 风格。",
        contact(name = "fangbc5@gmail.com"),
    ),
    tags(
        (name = "签名服务", description = "统一签名入口，支持上传/下载/分享三种模式"),
        (name = "分享服务", description = "JWT 长效分享链接，突破 S3 预签名 7 天限制"),
        (name = "对象操作", description = "PUT/GET/HEAD/DELETE 对象操作"),
        (name = "分片上传", description = "大文件分片上传全流程"),
    ),
    paths(
        handler::create_signature,
        handler::share_redirect,
        handler::put_object,
        handler::post_object,
        handler::get_object,
        handler::head_object,
        handler::delete_object,
    ),
    components(schemas(
        SignatureRequest,
        SignatureUploadResponse,
        SignatureDownloadResponse,
        SignatureShareResponse,
        PutObjectResponse,
        ObjectInfoResponse,
        MultipartInitResponse,
        PartUrlInfo,
        CompleteMultipartRequest,
        PartInfo,
        ListPartsResponse,
        PartDetail,
    ))
)]
pub struct ApiDoc;

/// 创建路由
///
/// 对标阿里云 OSS RESTful API 风格：
/// - 签名服务：POST /oss/signature
/// - 长效分享：GET /oss/share/{token}
/// - 对象操作：PUT/POST/GET/HEAD/DELETE /oss/{bucket}/*key
/// - API 文档：/swagger-ui
pub fn create_router(state: Arc<OssState>) -> Router {
    Router::new()
        // ---- Swagger UI（内嵌资源，无需 CDN）----
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
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
