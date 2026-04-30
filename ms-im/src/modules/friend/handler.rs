use axum::extract::{Path, Query, State};
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use std::sync::Arc;

use fbc_starter::{CursorPageBaseResp, RequestContext, R};

use crate::error::ImError;
use crate::state::ImState;
use crate::client::identity::IdentityClient;

use super::repository::{ApplyRepo, FriendRepo};
use super::model::{
    ApplyRequest, ApplyVO, DeleteFriendRequest, FriendSearchVO, FriendVO, ListFriendsRequest,
    SearchFriendRequest,
};

/// 好友列表（分页）
///
/// GET /api/v1/im/friends?cursor=1&page_size=20
#[utoipa::path(
    get,
    path = "/api/v1/im/friends",
    tag = "好友管理",
    params(
        ("cursor" = Option<u32>, Query, description = "游标"),
        ("page_size" = Option<u32>, Query, description = "每页条数"),
    ),
    responses(
        (status = 200, description = "好友列表", body = R<CursorPageBaseResp<FriendVO>>),
    )
)]
pub async fn list_friends(
    State(state): State<Arc<ImState>>,
    context: RequestContext,
    Query(req): Query<ListFriendsRequest>,
) -> Result<Json<R<CursorPageBaseResp<FriendVO>>>, ImError> {
    let page_size = req.page.page_size;

    let (list, next_cursor, has_next) = state
        .friend_service
        .list_friends(context.user_id, req.page.cursor, page_size)
        .await?;

    Ok(Json(R::ok_with_data(CursorPageBaseResp::init(
        next_cursor,
        !has_next,
        list,
        0,
    ))))
}

/// 删除好友
///
/// DELETE /api/v1/im/friends
#[utoipa::path(
    delete,
    path = "/api/v1/im/friends",
    tag = "好友管理",
    request_body = DeleteFriendRequest,
    responses(
        (status = 200, description = "删除成功", body = R<String>),
    )
)]
pub async fn delete_friend(
    State(state): State<Arc<ImState>>,
    context: RequestContext,
    Json(req): Json<DeleteFriendRequest>,
) -> Result<Json<R<()>>, ImError> {
    state.friend_service.delete_friend(context.user_id, req.friend_uid).await?;
    Ok(Json(R::ok()))
}

/// 发送好友申请
///
/// POST /api/v1/im/friends/applies
#[utoipa::path(
    post,
    path = "/api/v1/im/friends/applies",
    tag = "好友管理",
    request_body = ApplyRequest,
    responses(
        (status = 200, description = "申请成功，返回申请 ID", body = R<serde_json::Value>),
    )
)]
pub async fn apply(
    State(state): State<Arc<ImState>>,
    context: RequestContext,
    Json(req): Json<ApplyRequest>,
) -> Result<Json<R<serde_json::Value>>, ImError> {
    let id = state.friend_service.apply(context.user_id, req.target_id, req.msg).await?;
    Ok(Json(R::ok_with_data(serde_json::json!({ "apply_id": id }))))
}

/// 收到的申请列表
///
/// GET /api/v1/im/friends/applies
#[utoipa::path(
    get,
    path = "/api/v1/im/friends/applies",
    tag = "好友管理",
    responses(
        (status = 200, description = "申请列表", body = R<Vec<ApplyVO>>),
    )
)]
pub async fn list_applies(
    State(state): State<Arc<ImState>>,
    context: RequestContext,
) -> Result<Json<R<Vec<ApplyVO>>>, ImError> {
    let list = state.friend_service.list_applies(context.user_id).await?;
    Ok(Json(R::ok_with_data(list)))
}

/// 同意好友申请
///
/// POST /api/v1/im/friends/applies/{id}/approve
#[utoipa::path(
    post,
    path = "/api/v1/im/friends/applies/{id}/approve",
    tag = "好友管理",
    params(
        ("id" = i64, Path, description = "申请 ID"),
    ),
    responses(
        (status = 200, description = "同意成功", body = R<String>),
    )
)]
pub async fn approve(
    State(state): State<Arc<ImState>>,
    context: RequestContext,
    Path(id): Path<i64>,
) -> Result<Json<R<()>>, ImError> {
    state.friend_service.approve(
        id,
        context.user_id,
        &state.room_service,
        &state.contact_service,
    ).await?;
    Ok(Json(R::ok()))
}

/// 拒绝好友申请
///
/// POST /api/v1/im/friends/applies/{id}/reject
#[utoipa::path(
    post,
    path = "/api/v1/im/friends/applies/{id}/reject",
    tag = "好友管理",
    params(
        ("id" = i64, Path, description = "申请 ID"),
    ),
    responses(
        (status = 200, description = "拒绝成功", body = R<String>),
    )
)]
pub async fn reject(
    State(state): State<Arc<ImState>>,
    context: RequestContext,
    Path(id): Path<i64>,
) -> Result<Json<R<()>>, ImError> {
    state.friend_service.reject(id, context.user_id).await?;
    Ok(Json(R::ok()))
}

/// 搜索好友
///
/// GET /api/v1/im/friends/search-user?keyword=xxx
#[utoipa::path(
    get,
    path = "/api/v1/im/friends/search-user",
    tag = "好友管理",
    params(
        ("keyword" = String, Query, description = "搜索关键词"),
    ),
    responses(
        (status = 200, description = "搜索结果", body = R<FriendSearchVO>),
        (status = 400, description = "参数错误"),
    )
)]
pub async fn search_user(
    State(state): State<Arc<ImState>>,
    context: RequestContext,
    Query(req): Query<SearchFriendRequest>,
) -> Result<Json<R<FriendSearchVO>>, ImError> {
    let keyword = req.keyword.trim();
    if keyword.is_empty() {
        return Err(ImError::InvalidParam("搜索关键词不能为空".to_string()));
    }

    // 1. 调用 Identity 服务查找用户基础信息
    let user_brief = IdentityClient::search_user(keyword)
        .await
        .map_err(|e| ImError::SystemError(format!("调用 Identity 服务失败: {}", e)))?;

    let user_brief = match user_brief {
        Some(u) => u,
        None => return Err(ImError::InvalidParam("用户不存在".to_string())),
    };

    if user_brief.id == context.user_id {
        return Ok(Json(R::ok_with_data(FriendSearchVO {
            id: user_brief.id,
            nick_name: user_brief.nick_name,
            avatar: user_brief.avatar,
            is_friend: false,
            is_applying: false,
        })));
    }

    // 2. 查询好友关系状态
    let is_friend = FriendRepo::is_friend(state.db_pool.mysql_pool(), context.user_id, user_brief.id).await?;
    let is_applying = ApplyRepo::find_pending(state.db_pool.mysql_pool(), context.user_id, user_brief.id)
        .await?
        .is_some();

    // 3. 返回复合结果
    Ok(Json(R::ok_with_data(FriendSearchVO {
        id: user_brief.id,
        nick_name: user_brief.nick_name,
        avatar: user_brief.avatar,
        is_friend,
        is_applying,
    })))
}

/// 好友模块路由
pub fn friend_routes() -> Router<Arc<ImState>> {
    Router::new()
        .route("/search-user", get(search_user))
        .route("/", get(list_friends))
        .route("/", delete(delete_friend))
        .route("/applies", post(apply))
        .route("/applies", get(list_applies))
        .route("/applies/{id}/approve", post(approve))
        .route("/applies/{id}/reject", post(reject))
}
