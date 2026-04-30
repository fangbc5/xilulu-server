use axum::extract::{Path, Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use std::sync::Arc;

use fbc_starter::{RequestContext, R};

use crate::error::ImError;
use crate::state::ImState;

use super::model::{
    CursorPageResponse, MarkRequest, Message, MessageCursorQuery,
    SendMessageRequest, BatchLatestMessageRequest,
};

/// 发送消息
///
/// POST /api/v1/im/messages
#[utoipa::path(
    post,
    path = "/api/v1/im/messages",
    tag = "消息管理",
    request_body = SendMessageRequest,
    responses(
        (status = 200, description = "发送成功，返回消息", body = R<Message>),
    )
)]
pub async fn send_message(
    State(state): State<Arc<ImState>>,
    context: RequestContext,
    Json(req): Json<SendMessageRequest>,
) -> Result<Json<R<Message>>, ImError> {
    let msg = state.message_service.send_message(context.user_id, req).await?;
    Ok(Json(R::ok_with_data(msg)))
}

/// 消息列表（游标分页）
///
/// GET /api/v1/im/messages
#[utoipa::path(
    get,
    path = "/api/v1/im/messages",
    tag = "消息管理",
    params(
        ("room_id" = i64, Query, description = "房间 ID"),
        ("cursor" = Option<i64>, Query, description = "游标"),
        ("size" = Option<i64>, Query, description = "每页条数，默认 20"),
        ("fetch_mode" = Option<i16>, Query, description = "抓取方向 0历史 1新消息"),
    ),
    responses(
        (status = 200, description = "消息列表", body = R<CursorPageResponse<Message>>),
    )
)]
pub async fn list_messages(
    State(state): State<Arc<ImState>>,
    context: RequestContext,
    Query(query): Query<MessageCursorQuery>,
) -> Result<Json<R<CursorPageResponse<Message>>>, ImError> {
    let result = state.message_service.list_messages(context.user_id, query).await?;
    Ok(Json(R::ok_with_data(result)))
}

/// 撤回消息
///
/// POST /api/v1/im/messages/{id}/recall
#[utoipa::path(
    post,
    path = "/api/v1/im/messages/{id}/recall",
    tag = "消息管理",
    params(
        ("id" = i64, Path, description = "消息 ID"),
    ),
    responses(
        (status = 200, description = "撤回成功", body = R<String>),
    )
)]
pub async fn recall_message(
    State(state): State<Arc<ImState>>,
    context: RequestContext,
    Path(id): Path<i64>,
) -> Result<Json<R<()>>, ImError> {
    state.message_service.recall_message(id, context.user_id).await?;
    Ok(Json(R::ok()))
}

/// 标记消息
///
/// POST /api/v1/im/messages/{id}/mark
#[utoipa::path(
    post,
    path = "/api/v1/im/messages/{id}/mark",
    tag = "消息管理",
    params(
        ("id" = i64, Path, description = "消息 ID"),
    ),
    request_body = MarkRequest,
    responses(
        (status = 200, description = "标记成功", body = R<serde_json::Value>),
    )
)]
pub async fn mark_message(
    State(state): State<Arc<ImState>>,
    context: RequestContext,
    Path(id): Path<i64>,
    Json(req): Json<MarkRequest>,
) -> Result<Json<R<serde_json::Value>>, ImError> {
    let active = state.message_service.toggle_mark(id, context.user_id, req.r#type).await?;
    Ok(Json(R::ok_with_data(serde_json::json!({ "active": active }))))
}

/// 批量拉取多个房间的最新消息
///
/// POST /api/v1/im/messages/batch_latest
#[utoipa::path(
    post,
    path = "/api/v1/im/messages/batch_latest",
    tag = "消息管理",
    request_body = BatchLatestMessageRequest,
    responses(
        (status = 200, description = "批量消息结果", body = R<std::collections::HashMap<i64, Vec<Message>>>),
    )
)]
pub async fn batch_latest_messages(
    State(state): State<Arc<ImState>>,
    context: RequestContext,
    Json(req): Json<BatchLatestMessageRequest>,
) -> Result<Json<R<std::collections::HashMap<i64, Vec<Message>>>>, ImError> {
    let result = state.message_service.batch_latest_messages(context.user_id, req).await?;
    Ok(Json(R::ok_with_data(result)))
}

/// 消息模块路由
pub fn message_routes() -> Router<Arc<ImState>> {
    Router::new()
        .route("/", post(send_message))
        .route("/", get(list_messages))
        .route("/batch_latest", post(batch_latest_messages))
        .route("/{id}/recall", post(recall_message))
        .route("/{id}/mark", post(mark_message))
}
