use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::Json;
use fbc_starter::{CursorPageBaseResp, RequestContext, R};

use crate::error::ContentError;
use crate::state::ContentState;
use super::model::dto::*;

/// 创建内容
///
/// POST /api/v1/contents
#[utoipa::path(
    post,
    path = "/api/v1/contents",
    tag = "内容管理",
    request_body = CreateContentReq,
    responses(
        (status = 200, description = "创建成功，返回内容 ID", body = R<i64>),
        (status = 400, description = "参数校验失败"),
    )
)]
pub async fn create_content(
    State(state): State<Arc<ContentState>>,
    ctx: RequestContext,
    Json(req): Json<CreateContentReq>,
) -> Result<Json<R<i64>>, ContentError> {
    let content_id = state
        .content_service
        .create_content(ctx.user_id, req)
        .await?;
    Ok(Json(R::ok_with_data(content_id)))
}

/// 获取内容详情（详情页永远查 DB，保证绝对实时）
///
/// GET /api/v1/contents/{id}
#[utoipa::path(
    get,
    path = "/api/v1/contents/{id}",
    tag = "内容管理",
    params(
        ("id" = i64, Path, description = "内容 ID"),
    ),
    responses(
        (status = 200, description = "内容详情", body = R<ContentDetailResp>),
        (status = 404, description = "内容不存在"),
    )
)]
pub async fn get_content(
    State(state): State<Arc<ContentState>>,
    Path(id): Path<i64>,
) -> Result<Json<R<ContentDetailResp>>, ContentError> {
    let detail = state.content_service.get_content_detail(id).await?;
    Ok(Json(R::ok_with_data(detail)))
}

/// 变更内容状态（发布/下架/删除）
///
/// PUT /api/v1/contents/{id}/status
#[utoipa::path(
    put,
    path = "/api/v1/contents/{id}/status",
    tag = "内容管理",
    params(
        ("id" = i64, Path, description = "内容 ID"),
    ),
    request_body = ChangeStatusReq,
    responses(
        (status = 200, description = "状态变更成功", body = R<String>),
        (status = 404, description = "内容不存在"),
        (status = 409, description = "版本号冲突"),
    )
)]
pub async fn change_status(
    State(state): State<Arc<ContentState>>,
    Path(id): Path<i64>,
    Json(req): Json<ChangeStatusReq>,
) -> Result<Json<R<String>>, ContentError> {
    state.content_service.change_status(id, req).await?;
    Ok(Json(R::ok_with_data("状态变更成功".to_string())))
}

/// 逻辑删除内容
///
/// DELETE /api/v1/contents/{id}
#[utoipa::path(
    delete,
    path = "/api/v1/contents/{id}",
    tag = "内容管理",
    params(
        ("id" = i64, Path, description = "内容 ID"),
    ),
    responses(
        (status = 200, description = "删除成功", body = R<String>),
        (status = 404, description = "内容不存在"),
    )
)]
pub async fn delete_content(
    State(state): State<Arc<ContentState>>,
    Path(id): Path<i64>,
) -> Result<Json<R<String>>, ContentError> {
    state.content_service.delete_content(id).await?;
    Ok(Json(R::ok_with_data("删除成功".to_string())))
}

/// 搜索内容（查 Meilisearch）
///
/// GET /api/v1/contents/search
#[utoipa::path(
    get,
    path = "/api/v1/contents/search",
    tag = "内容搜索",
    params(
        ("keyword" = Option<String>, Query, description = "搜索关键词"),
        ("content_type" = Option<String>, Query, description = "内容类型筛选"),
        ("author_id" = Option<i64>, Query, description = "作者 ID 筛选"),
        ("sort_by" = Option<String>, Query, description = "排序字段，如 published_at:desc"),
        ("page_size" = Option<u32>, Query, description = "每页条数（默认 10）"),
        ("cursor" = Option<u32>, Query, description = "游标（首页不传）"),
    ),
    responses(
        (status = 200, description = "搜索结果（游标分页）", body = R<CursorPageBaseResp<super::search::port::SearchDocument>>),
    )
)]
pub async fn search_contents(
    State(state): State<Arc<ContentState>>,
    Query(req): Query<SearchContentReq>,
) -> Result<Json<R<CursorPageBaseResp<super::search::port::SearchDocument>>>, ContentError> {
    let (hits, next_cursor, has_next, total) = state.content_service.search_contents(req).await?;
    Ok(Json(R::ok_with_data(CursorPageBaseResp::init(
        next_cursor,
        has_next,
        hits,
        total as i64,
    ))))
}

/// 创建关系
///
/// POST /api/v1/contents/{id}/relations
#[utoipa::path(
    post,
    path = "/api/v1/contents/{id}/relations",
    tag = "内容关系",
    params(
        ("id" = i64, Path, description = "源内容 ID"),
    ),
    request_body = CreateRelationReq,
    responses(
        (status = 200, description = "关系创建成功，返回关系 ID", body = R<i64>),
        (status = 404, description = "源或目标内容不存在"),
        (status = 400, description = "关系深度超限"),
    )
)]
pub async fn create_relation(
    State(state): State<Arc<ContentState>>,
    Path(id): Path<i64>,
    Json(req): Json<CreateRelationReq>,
) -> Result<Json<R<i64>>, ContentError> {
    let relation_id = state.content_service.create_relation(id, req).await?;
    Ok(Json(R::ok_with_data(relation_id)))
}

/// 查询关系列表
///
/// GET /api/v1/contents/{id}/relations?type=comment&cursor=0&page_size=20
#[utoipa::path(
    get,
    path = "/api/v1/contents/{id}/relations",
    tag = "内容关系",
    params(
        ("id" = i64, Path, description = "目标内容 ID"),
        ("type" = String, Query, description = "关系类型（comment / reply / attach / quote / collection）"),
        ("page_size" = Option<u32>, Query, description = "每页条数（默认 10）"),
        ("cursor" = Option<u32>, Query, description = "游标（首页不传）"),
    ),
    responses(
        (status = 200, description = "关系列表（游标分页）", body = R<CursorPageBaseResp<ContentRelationResp>>),
    )
)]
pub async fn get_relations(
    State(state): State<Arc<ContentState>>,
    Path(id): Path<i64>,
    Query(params): Query<RelationQueryParams>,
) -> Result<Json<R<CursorPageBaseResp<ContentRelationResp>>>, ContentError> {
    let page_size = params.page.page_size;
    let (list, next_cursor, has_next) = state
        .content_service
        .get_relations(id, &params.relation_type, params.page.cursor, page_size)
        .await?;
        
    Ok(Json(R::ok_with_data(CursorPageBaseResp::init(
        next_cursor,
        has_next,
        list,
        0, // 关系查询不需要 total
    ))))
}
