use std::sync::Arc;

use axum::routing::{delete, get, post, put};
use axum::Router;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::modules::content;
use crate::modules::content::model::dto::*;
use crate::modules::content::search::port::SearchDocument;
use crate::modules::content::model::domain::Block;
use crate::state::ContentState;

/// OpenAPI 文档定义
///
/// 自动收集所有标注了 `#[utoipa::path]` 的 handler
#[derive(OpenApi)]
#[openapi(
    info(
        title = "ms-content 内容中台",
        version = "0.1.0",
        description = "Xilulu 微服务生态的统一内容管理中台。\n\n支持多内容类型（文章、动态、评论）、Block DSL 结构化正文、内容关系图谱、全文搜索等能力。",
        contact(name = "fangbc5@gmail.com"),
    ),
    tags(
        (name = "内容管理", description = "内容 CRUD（创建/查询/状态变更/删除）"),
        (name = "内容搜索", description = "全文搜索（Meilisearch）"),
        (name = "内容关系", description = "内容关系图谱（评论/回复/引用/收藏）"),
    ),
    paths(
        content::create_content,
        content::get_content,
        content::change_status,
        content::delete_content,
        content::search_contents,
        content::create_relation,
        content::get_relations,
    ),
    components(schemas(
        CreateContentReq,
        ChangeStatusReq,
        CreateRelationReq,
        ContentDetailResp,
        ContentStatsResp,
        ContentRelationResp,
        SearchDocument,
        Block,
    ))
)]
pub struct ApiDoc;

/// 创建 HTTP 路由
pub fn create_router(state: Arc<ContentState>) -> Router {
    Router::new()
        // ---- Swagger UI（内嵌资源，无需 CDN）----
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .nest(
            "/api/v1/contents",
            Router::new()
                // 搜索（必须在 /{id} 之前注册，否则 "search" 会被当作 id）
                .route("/search", get(content::search_contents))
                // CRUD
                .route("/", post(content::create_content))
                .route("/{id}", get(content::get_content))
                .route("/{id}", delete(content::delete_content))
                .route("/{id}/status", put(content::change_status))
                // 关系
                .route("/{id}/relations", post(content::create_relation))
                .route("/{id}/relations", get(content::get_relations)),
        )
        .with_state(state)
}
