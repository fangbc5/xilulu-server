use axum::extract::{Path, State};
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use std::sync::Arc;

use fbc_starter::{RequestContext, R};

use crate::error::ImError;
use crate::state::ImState;

use super::model::{
    AddMemberRequest, CreateGroupRequest, TransferOwnerRequest, UpdateGroupRequest,
    GroupMember, RoomGroup,
};

/// 创建群聊 POST /api/v1/rooms/groups
async fn create_group(
    State(state): State<Arc<ImState>>,
    context: RequestContext,
    Json(req): Json<CreateGroupRequest>,
) -> Result<Json<R<serde_json::Value>>, ImError> {
    // 1. 创建群聊房间和成员
    let (room_id, all_uids) = state.room_service.create_group_room(
        context.user_id,
        req.name,
        req.member_uids,
    ).await?;

    // 2. 跨模块编排：为所有成员创建会话
    for uid in all_uids {
        let _ = state.contact_service.create_contact(uid, room_id).await;
    }

    Ok(Json(R::ok_with_data(serde_json::json!({ "room_id": room_id }))))
}

/// 群详情 GET /api/v1/rooms/groups/{id}
async fn group_info(
    State(state): State<Arc<ImState>>,
    Path(id): Path<i64>,
) -> Result<Json<R<Option<RoomGroup>>>, ImError> {
    let info = state.room_service.get_group_info(id).await?;
    Ok(Json(R::ok_with_data(info)))
}

/// 更新群信息 POST /api/v1/rooms/groups/{id}
async fn update_group(
    State(state): State<Arc<ImState>>,
    context: RequestContext,
    Path(id): Path<i64>,
    Json(req): Json<UpdateGroupRequest>,
) -> Result<Json<R<()>>, ImError> {
    state.room_service.update_group_info(id, context.user_id, req.name, req.notice).await?;
    Ok(Json(R::ok()))
}

/// 群成员列表 GET /api/v1/rooms/groups/{id}/members
async fn list_members(
    State(state): State<Arc<ImState>>,
    Path(id): Path<i64>,
) -> Result<Json<R<Vec<GroupMember>>>, ImError> {
    let list = state.room_service.list_group_members(id).await?;
    Ok(Json(R::ok_with_data(list)))
}

/// 添加群成员 POST /api/v1/rooms/groups/{id}/members
async fn add_member(
    State(state): State<Arc<ImState>>,
    context: RequestContext,
    Path(id): Path<i64>,
    Json(req): Json<AddMemberRequest>,
) -> Result<Json<R<()>>, ImError> {
    // 1. 添加群成员
    state.room_service.add_group_member(id, context.user_id, req.uid).await?;

    // 2. 跨模块编排：创建会话
    let _ = state.contact_service.create_contact(req.uid, id).await;

    Ok(Json(R::ok()))
}

/// 移除群成员 DELETE /api/v1/rooms/groups/{id}/members/{uid}
async fn remove_member(
    State(state): State<Arc<ImState>>,
    context: RequestContext,
    Path((id, uid)): Path<(i64, i64)>,
) -> Result<Json<R<()>>, ImError> {
    state.room_service.remove_group_member(id, context.user_id, uid).await?;
    Ok(Json(R::ok()))
}

/// 退出群聊 POST /api/v1/rooms/groups/{id}/quit
async fn quit_group(
    State(state): State<Arc<ImState>>,
    context: RequestContext,
    Path(id): Path<i64>,
) -> Result<Json<R<()>>, ImError> {
    state.room_service.quit_group(id, context.user_id).await?;
    Ok(Json(R::ok()))
}

/// 解散群聊 POST /api/v1/rooms/groups/{id}/dissolve
async fn dissolve_group(
    State(state): State<Arc<ImState>>,
    context: RequestContext,
    Path(id): Path<i64>,
) -> Result<Json<R<()>>, ImError> {
    // 1. 解散群聊并获取成员列表
    let member_uids = state.room_service.dissolve_group(id, context.user_id).await?;

    // 2. 跨模块编排：删除所有成员的会话
    for uid in member_uids {
        let _ = state.contact_service.delete_contact(uid, id).await;
    }

    Ok(Json(R::ok()))
}

/// 转让群主 POST /api/v1/rooms/groups/{id}/transfer
async fn transfer_owner(
    State(state): State<Arc<ImState>>,
    context: RequestContext,
    Path(id): Path<i64>,
    Json(req): Json<TransferOwnerRequest>,
) -> Result<Json<R<()>>, ImError> {
    state.room_service.transfer_owner(id, context.user_id, req.new_owner_uid).await?;
    Ok(Json(R::ok()))
}

/// 房间模块路由
pub fn room_routes() -> Router<Arc<ImState>> {
    Router::new()
        .route("/groups", post(create_group))
        .route("/groups/{id}", get(group_info))
        .route("/groups/{id}", post(update_group))
        .route("/groups/{id}/members", get(list_members))
        .route("/groups/{id}/members", post(add_member))
        .route("/groups/{id}/members/{uid}", delete(remove_member))
        .route("/groups/{id}/quit", post(quit_group))
        .route("/groups/{id}/dissolve", post(dissolve_group))
        .route("/groups/{id}/transfer", post(transfer_owner))
}
