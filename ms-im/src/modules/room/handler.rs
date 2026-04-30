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

/// 创建群聊
///
/// POST /api/v1/im/rooms/groups
#[utoipa::path(
    post,
    path = "/api/v1/im/rooms/groups",
    tag = "群聊管理",
    request_body = CreateGroupRequest,
    responses(
        (status = 200, description = "创建成功，返回 room_id", body = R<serde_json::Value>),
    )
)]
pub async fn create_group(
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

/// 群详情
///
/// GET /api/v1/im/rooms/groups/{id}
#[utoipa::path(
    get,
    path = "/api/v1/im/rooms/groups/{id}",
    tag = "群聊管理",
    params(
        ("id" = i64, Path, description = "群聊房间 ID"),
    ),
    responses(
        (status = 200, description = "群详情", body = R<Option<RoomGroup>>),
    )
)]
pub async fn group_info(
    State(state): State<Arc<ImState>>,
    Path(id): Path<i64>,
) -> Result<Json<R<Option<RoomGroup>>>, ImError> {
    let info = state.room_service.get_group_info(id).await?;
    Ok(Json(R::ok_with_data(info)))
}

/// 更新群信息
///
/// POST /api/v1/im/rooms/groups/{id}
#[utoipa::path(
    post,
    path = "/api/v1/im/rooms/groups/{id}",
    tag = "群聊管理",
    params(
        ("id" = i64, Path, description = "群聊房间 ID"),
    ),
    request_body = UpdateGroupRequest,
    responses(
        (status = 200, description = "更新成功", body = R<String>),
    )
)]
pub async fn update_group(
    State(state): State<Arc<ImState>>,
    context: RequestContext,
    Path(id): Path<i64>,
    Json(req): Json<UpdateGroupRequest>,
) -> Result<Json<R<()>>, ImError> {
    state.room_service.update_group_info(id, context.user_id, req.name, req.notice).await?;
    Ok(Json(R::ok()))
}

/// 群成员列表
///
/// GET /api/v1/im/rooms/groups/{id}/members
#[utoipa::path(
    get,
    path = "/api/v1/im/rooms/groups/{id}/members",
    tag = "群聊管理",
    params(
        ("id" = i64, Path, description = "群聊房间 ID"),
    ),
    responses(
        (status = 200, description = "群成员列表", body = R<Vec<GroupMember>>),
    )
)]
pub async fn list_members(
    State(state): State<Arc<ImState>>,
    Path(id): Path<i64>,
) -> Result<Json<R<Vec<GroupMember>>>, ImError> {
    let list = state.room_service.list_group_members(id).await?;
    Ok(Json(R::ok_with_data(list)))
}

/// 添加群成员
///
/// POST /api/v1/im/rooms/groups/{id}/members
#[utoipa::path(
    post,
    path = "/api/v1/im/rooms/groups/{id}/members",
    tag = "群聊管理",
    params(
        ("id" = i64, Path, description = "群聊房间 ID"),
    ),
    request_body = AddMemberRequest,
    responses(
        (status = 200, description = "添加成功", body = R<String>),
    )
)]
pub async fn add_member(
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

/// 移除群成员
///
/// DELETE /api/v1/im/rooms/groups/{id}/members/{uid}
#[utoipa::path(
    delete,
    path = "/api/v1/im/rooms/groups/{id}/members/{uid}",
    tag = "群聊管理",
    params(
        ("id" = i64, Path, description = "群聊房间 ID"),
        ("uid" = i64, Path, description = "被移除的用户 ID"),
    ),
    responses(
        (status = 200, description = "移除成功", body = R<String>),
    )
)]
pub async fn remove_member(
    State(state): State<Arc<ImState>>,
    context: RequestContext,
    Path((id, uid)): Path<(i64, i64)>,
) -> Result<Json<R<()>>, ImError> {
    state.room_service.remove_group_member(id, context.user_id, uid).await?;
    Ok(Json(R::ok()))
}

/// 退出群聊
///
/// POST /api/v1/im/rooms/groups/{id}/quit
#[utoipa::path(
    post,
    path = "/api/v1/im/rooms/groups/{id}/quit",
    tag = "群聊管理",
    params(
        ("id" = i64, Path, description = "群聊房间 ID"),
    ),
    responses(
        (status = 200, description = "退出成功", body = R<String>),
    )
)]
pub async fn quit_group(
    State(state): State<Arc<ImState>>,
    context: RequestContext,
    Path(id): Path<i64>,
) -> Result<Json<R<()>>, ImError> {
    state.room_service.quit_group(id, context.user_id).await?;
    Ok(Json(R::ok()))
}

/// 解散群聊
///
/// POST /api/v1/im/rooms/groups/{id}/dissolve
#[utoipa::path(
    post,
    path = "/api/v1/im/rooms/groups/{id}/dissolve",
    tag = "群聊管理",
    params(
        ("id" = i64, Path, description = "群聊房间 ID"),
    ),
    responses(
        (status = 200, description = "解散成功", body = R<String>),
    )
)]
pub async fn dissolve_group(
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

/// 转让群主
///
/// POST /api/v1/im/rooms/groups/{id}/transfer
#[utoipa::path(
    post,
    path = "/api/v1/im/rooms/groups/{id}/transfer",
    tag = "群聊管理",
    params(
        ("id" = i64, Path, description = "群聊房间 ID"),
    ),
    request_body = TransferOwnerRequest,
    responses(
        (status = 200, description = "转让成功", body = R<String>),
    )
)]
pub async fn transfer_owner(
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
