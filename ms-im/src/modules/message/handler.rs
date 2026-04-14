use axum::extract::{Path, Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use std::sync::Arc;

use fbc_starter::{RequestContext, R};

use crate::error::ImError;
use crate::state::ImState;

use super::model::{
    CursorPageResponse, MarkRequest, Message, MessageCursorQuery,
    SendMessageRequest,
};

/// 发送消息 POST /api/v1/messages
async fn send_message(
    State(state): State<Arc<ImState>>,
    context: RequestContext,
    Json(req): Json<SendMessageRequest>,
) -> Result<Json<R<Message>>, ImError> {
    let msg = state.message_service.send_message(context.user_id, req).await?;
    Ok(Json(R::ok_with_data(msg)))
}

/// 消息列表（游标分页） GET /api/v1/messages
async fn list_messages(
    State(state): State<Arc<ImState>>,
    context: RequestContext,
    Query(query): Query<MessageCursorQuery>,
) -> Result<Json<R<CursorPageResponse<Message>>>, ImError> {
    let result = state.message_service.list_messages(context.user_id, query).await?;
    Ok(Json(R::ok_with_data(result)))
}

/// 撤回消息 POST /api/v1/messages/{id}/recall
async fn recall_message(
    State(state): State<Arc<ImState>>,
    context: RequestContext,
    Path(id): Path<i64>,
) -> Result<Json<R<()>>, ImError> {
    state.message_service.recall_message(id, context.user_id).await?;
    Ok(Json(R::ok()))
}

/// 标记消息 POST /api/v1/messages/{id}/mark
async fn mark_message(
    State(state): State<Arc<ImState>>,
    context: RequestContext,
    Path(id): Path<i64>,
    Json(req): Json<MarkRequest>,
) -> Result<Json<R<serde_json::Value>>, ImError> {
    let active = state.message_service.toggle_mark(id, context.user_id, req.r#type).await?;
    Ok(Json(R::ok_with_data(serde_json::json!({ "active": active }))))
}

/// 消息模块路由
pub fn message_routes() -> Router<Arc<ImState>> {
    Router::new()
        .route("/", post(send_message))
        .route("/", get(list_messages))
        .route("/{id}/recall", post(recall_message))
        .route("/{id}/mark", post(mark_message))
}
