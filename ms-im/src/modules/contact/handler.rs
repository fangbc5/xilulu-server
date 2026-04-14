use axum::extract::{Query, State};
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use std::sync::Arc;

use fbc_starter::{CursorPageBaseResp, RequestContext, R};

use crate::error::ImError;
use crate::state::ImState;

use super::model::{
    ContactRequest, ContactSettingRequest, ContactVO, ListContactsRequest, MarkReadRequest
};

/// 会话列表 GET /api/v1/contacts?cursor=xxx&page_size=20
async fn list_contacts(
    State(state): State<Arc<ImState>>,
    context: RequestContext,
    Query(req): Query<ListContactsRequest>,
) -> Result<Json<R<CursorPageBaseResp<ContactVO>>>, ImError> {
    let page_size = req.page.page_size;
    let (list, next_cursor, has_next) = state
        .contact_service
        .list_contacts(context.user_id, req.page.cursor, page_size)
        .await?;

    Ok(Json(R::ok_with_data(CursorPageBaseResp::init(
        next_cursor,
        has_next,
        list,
        0,
    ))))
}

/// 置顶 POST /api/v1/contacts/top
async fn set_top(
    State(state): State<Arc<ImState>>,
    context: RequestContext,
    Json(req): Json<ContactSettingRequest>,
) -> Result<Json<R<()>>, ImError> {
    state.contact_service.set_top(context.user_id, req.room_id, req.value).await?;
    Ok(Json(R::ok()))
}

/// 免打扰 POST /api/v1/contacts/mute
async fn set_mute(
    State(state): State<Arc<ImState>>,
    context: RequestContext,
    Json(req): Json<ContactSettingRequest>,
) -> Result<Json<R<()>>, ImError> {
    state.contact_service.set_mute(context.user_id, req.room_id, req.value).await?;
    Ok(Json(R::ok()))
}

/// 已读上报 POST /api/v1/contacts/read
async fn mark_read(
    State(state): State<Arc<ImState>>,
    context: RequestContext,
    Json(req): Json<MarkReadRequest>,
) -> Result<Json<R<()>>, ImError> {
    state.contact_service.mark_read(context.user_id, req.room_id, req.read_msg_id, req.diff_count).await?;
    Ok(Json(R::ok()))
}

/// 标为未读 POST /api/v1/contacts/unread
async fn mark_unread(
    State(state): State<Arc<ImState>>,
    context: RequestContext,
    Json(req): Json<ContactRequest>,
) -> Result<Json<R<()>>, ImError> {
    state.contact_service.mark_unread(context.user_id, req.room_id).await?;
    Ok(Json(R::ok()))
}

/// 删除会话 DELETE /api/v1/contacts
async fn delete_contact(
    State(state): State<Arc<ImState>>,
    context: RequestContext,
    Json(req): Json<ContactRequest>,
) -> Result<Json<R<()>>, ImError> {
    state.contact_service.delete_contact(context.user_id, req.room_id).await?;
    Ok(Json(R::ok()))
}

/// 会话模块路由
pub fn contact_routes() -> Router<Arc<ImState>> {
    Router::new()
        .route("/", get(list_contacts))
        .route("/", delete(delete_contact))
        .route("/top", post(set_top))
        .route("/mute", post(set_mute))
        .route("/read", post(mark_read))
        .route("/unread", post(mark_unread))
}
