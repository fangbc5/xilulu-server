// 用户相关 HTTP 处理器

use crate::context::RequestContext;
use crate::error::{IdentityError, to_err};
use crate::modules::user::*;
use crate::state::AppState;
use axum::{
    extract::{Path, Query, State},
    response::Json,
};
use fbc_starter::{base::CursorPageBaseResp, R};
use std::sync::Arc;

/// 获取用户信息
#[utoipa::path(
    get,
    path = "/api/v1/identity/users/{id}",
    tag = "用户管理",
    params(("id" = i64, Path, description = "用户 ID")),
    responses((status = 200, description = "用户信息", body = R<UserInfo>))
)]
pub async fn get_user(
    State(app_state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<Json<R<UserInfo>>, IdentityError> {
    let user = app_state.user_service.get_user_info(id).await.map_err(to_err)?;
    Ok(Json(R::ok_with_data(UserInfo::from(user))))
}

/// 获取用户列表（分页）
#[utoipa::path(
    get,
    path = "/api/v1/identity/users",
    tag = "用户管理",
    params(
        ("cursor" = Option<u32>, Query, description = "游标"),
        ("page_size" = Option<u32>, Query, description = "每页条数"),
        ("search_key" = Option<String>, Query, description = "搜索关键词"),
        ("tenant_id" = Option<i64>, Query, description = "租户 ID"),
    ),
    responses((status = 200, description = "用户列表", body = R<CursorPageBaseResp<UserInfo>>))
)]
pub async fn list_users(
    State(app_state): State<Arc<AppState>>,
    Query(req): Query<ListUsersRequest>,
) -> Result<Json<R<CursorPageBaseResp<UserInfo>>>, IdentityError> {
    let page = req.page.cursor.unwrap_or(1);
    let page_size = req.page.page_size;

    let (users, total) = app_state
        .user_service
        .list_users(page, page_size, req.search_key.as_deref(), req.tenant_id)
        .await
        .map_err(to_err)?;

    let list: Vec<UserInfo> = users.into_iter().map(UserInfo::from).collect();
    let next_cursor = if list.len() as u32 >= page_size {
        Some((page as u32) * page_size)
    } else {
        None
    };
    let has_next = next_cursor.is_some() && ((page as i64) * (page_size as i64)) < total;

    Ok(Json(R::ok_with_data(CursorPageBaseResp::init(
        next_cursor,
        has_next,
        list,
        total,
    ))))
}

/// 创建用户（需要认证，管理员接口）
#[utoipa::path(
    post,
    path = "/api/v1/identity/users",
    tag = "用户管理",
    request_body = CreateUserRequest,
    responses((status = 200, description = "创建成功", body = R<CreateUserResponse>))
)]
pub async fn create_user(
    State(app_state): State<Arc<AppState>>,
    context: RequestContext,
    axum::Json(req): axum::Json<CreateUserRequest>,
) -> Result<Json<R<CreateUserResponse>>, IdentityError> {
    let user_id = app_state
        .user_service
        .create_user(
            &req.username,
            &req.password,
            req.email.as_deref(),
            req.mobile.as_deref(),
            req.nick_name.as_deref(),
            Some(context.user_id),
        )
        .await
        .map_err(to_err)?;
    Ok(Json(R::ok_with_data(CreateUserResponse { user_id })))
}

/// 更新用户
#[utoipa::path(
    put,
    path = "/api/v1/identity/users/{id}",
    tag = "用户管理",
    params(("id" = i64, Path, description = "用户 ID")),
    request_body = UpdateUserRequest,
    responses((status = 200, description = "更新成功", body = R<String>))
)]
pub async fn update_user(
    State(app_state): State<Arc<AppState>>,
    context: RequestContext,
    Path(id): Path<i64>,
    axum::Json(req): axum::Json<UpdateUserRequest>,
) -> Result<Json<R<()>>, IdentityError> {
    app_state
        .user_service
        .update_user(
            id,
            req.username.as_deref(),
            req.email.as_deref(),
            req.mobile.as_deref(),
            req.nick_name.as_deref(),
            Some(context.user_id),
        )
        .await
        .map_err(to_err)?;
    Ok(Json(R::ok()))
}

/// 删除用户
#[utoipa::path(
    delete,
    path = "/api/v1/identity/users/{id}",
    tag = "用户管理",
    params(("id" = i64, Path, description = "用户 ID")),
    responses((status = 200, description = "删除成功", body = R<String>))
)]
pub async fn delete_user(
    State(app_state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<Json<R<()>>, IdentityError> {
    app_state.user_service.delete_user(id).await.map_err(to_err)?;
    Ok(Json(R::ok()))
}

/// 修改密码
#[utoipa::path(
    put,
    path = "/api/v1/identity/users/{id}/password",
    tag = "用户管理",
    params(("id" = i64, Path, description = "用户 ID")),
    request_body = ChangePasswordRequest,
    responses((status = 200, description = "修改成功", body = R<String>))
)]
pub async fn change_password(
    State(app_state): State<Arc<AppState>>,
    context: RequestContext,
    Path(id): Path<i64>,
    axum::Json(req): axum::Json<ChangePasswordRequest>,
) -> Result<Json<R<()>>, IdentityError> {
    app_state
        .user_service
        .change_password(id, &req.old_password, &req.new_password, Some(context.user_id))
        .await
        .map_err(to_err)?;
    Ok(Json(R::ok()))
}

/// 重置密码
#[utoipa::path(
    put,
    path = "/api/v1/identity/users/{id}/password/reset",
    tag = "用户管理",
    params(("id" = i64, Path, description = "用户 ID")),
    request_body = ResetPasswordRequest,
    responses((status = 200, description = "重置成功", body = R<String>))
)]
pub async fn reset_password(
    State(app_state): State<Arc<AppState>>,
    context: RequestContext,
    Path(id): Path<i64>,
    axum::Json(req): axum::Json<ResetPasswordRequest>,
) -> Result<Json<R<()>>, IdentityError> {
    app_state
        .user_service
        .reset_password(id, &req.new_password, Some(context.user_id))
        .await
        .map_err(to_err)?;
    Ok(Json(R::ok()))
}

/// 添加用户到租户
#[utoipa::path(
    post,
    path = "/api/v1/identity/users/{id}/tenants",
    tag = "用户管理",
    params(("id" = i64, Path, description = "用户 ID")),
    request_body = AddUserToTenantRequest,
    responses((status = 200, description = "添加成功", body = R<String>))
)]
pub async fn add_user_to_tenant(
    State(app_state): State<Arc<AppState>>,
    context: RequestContext,
    Path(user_id): Path<i64>,
    axum::Json(req): axum::Json<AddUserToTenantRequest>,
) -> Result<Json<R<()>>, IdentityError> {
    app_state
        .user_tenant_service
        .add_user_to_tenant(user_id, req.tenant_id, false, Some(context.user_id))
        .await
        .map_err(to_err)?;
    Ok(Json(R::ok()))
}

/// 从租户移除用户
#[utoipa::path(
    delete,
    path = "/api/v1/identity/users/{id}/tenants",
    tag = "用户管理",
    params(("id" = i64, Path, description = "用户 ID")),
    responses((status = 200, description = "移除成功", body = R<String>))
)]
pub async fn remove_user_from_tenant(
    State(app_state): State<Arc<AppState>>,
    Path(user_id): Path<i64>,
    axum::Json(req): axum::Json<serde_json::Value>,
) -> Result<Json<R<()>>, IdentityError> {
    let tenant_id = req.get("tenant_id").and_then(|v| v.as_i64()).unwrap_or(0);
    app_state
        .user_tenant_service
        .remove_user_from_tenant(user_id, tenant_id)
        .await
        .map_err(to_err)?;
    Ok(Json(R::ok()))
}

/// 设置默认租户
#[utoipa::path(
    put,
    path = "/api/v1/identity/users/{id}/tenants/default",
    tag = "用户管理",
    params(("id" = i64, Path, description = "用户 ID")),
    request_body = SetDefaultTenantRequest,
    responses((status = 200, description = "设置成功", body = R<String>))
)]
pub async fn set_default_tenant(
    State(app_state): State<Arc<AppState>>,
    context: RequestContext,
    Path(user_id): Path<i64>,
    axum::Json(req): axum::Json<SetDefaultTenantRequest>,
) -> Result<Json<R<()>>, IdentityError> {
    app_state
        .user_tenant_service
        .set_default_tenant(user_id, req.tenant_id, Some(context.user_id))
        .await
        .map_err(to_err)?;
    Ok(Json(R::ok()))
}

/// 获取用户的租户列表
#[utoipa::path(
    get,
    path = "/api/v1/identity/users/{id}/tenants",
    tag = "用户管理",
    params(("id" = i64, Path, description = "用户 ID")),
    responses((status = 200, description = "租户列表", body = R<Vec<UserTenantInfo>>))
)]
pub async fn get_user_tenants(
    State(app_state): State<Arc<AppState>>,
    Path(user_id): Path<i64>,
) -> Result<Json<R<Vec<UserTenantInfo>>>, IdentityError> {
    let rels = app_state
        .user_tenant_service
        .get_user_tenants(user_id)
        .await
        .map_err(to_err)?;
    let infos = rels.into_iter().map(UserTenantInfo::from).collect();
    Ok(Json(R::ok_with_data(infos)))
}

// ========== 用户角色相关 Handlers ==========

/// 获取用户在租户下的角色列表
#[utoipa::path(
    get,
    path = "/api/v1/identity/users/{user_id}/tenants/{tenant_id}/roles",
    tag = "用户管理",
    params(
        ("user_id" = i64, Path, description = "用户 ID"),
        ("tenant_id" = i64, Path, description = "租户 ID"),
    ),
    responses((status = 200, description = "角色列表", body = R<Vec<UserRoleInfo>>))
)]
pub async fn get_user_roles(
    State(app_state): State<Arc<AppState>>,
    Path((user_id, tenant_id)): Path<(i64, i64)>,
) -> Result<Json<R<Vec<UserRoleInfo>>>, IdentityError> {
    let roles = app_state
        .user_role_service
        .get_user_roles(user_id, tenant_id)
        .await
        .map_err(to_err)?;
    let infos = roles.into_iter().map(UserRoleInfo::from).collect();
    Ok(Json(R::ok_with_data(infos)))
}

/// 为用户分配角色
#[utoipa::path(
    post,
    path = "/api/v1/identity/users/{user_id}/tenants/{tenant_id}/roles",
    tag = "用户管理",
    params(
        ("user_id" = i64, Path, description = "用户 ID"),
        ("tenant_id" = i64, Path, description = "租户 ID"),
    ),
    request_body = AssignRoleToUserRequest,
    responses((status = 200, description = "分配成功", body = R<String>))
)]
pub async fn assign_role_to_user(
    State(app_state): State<Arc<AppState>>,
    context: RequestContext,
    Path((user_id, tenant_id)): Path<(i64, i64)>,
    axum::Json(req): axum::Json<AssignRoleToUserRequest>,
) -> Result<Json<R<()>>, IdentityError> {
    app_state
        .user_role_service
        .assign_role_to_user(user_id, req.role_id, tenant_id, Some(context.user_id))
        .await
        .map_err(to_err)?;
    Ok(Json(R::ok()))
}

/// 批量为用户分配角色
#[utoipa::path(
    post,
    path = "/api/v1/identity/users/{user_id}/tenants/{tenant_id}/roles/batch",
    tag = "用户管理",
    params(
        ("user_id" = i64, Path, description = "用户 ID"),
        ("tenant_id" = i64, Path, description = "租户 ID"),
    ),
    request_body = BatchAssignRolesToUserRequest,
    responses((status = 200, description = "批量分配成功", body = R<String>))
)]
pub async fn batch_assign_roles_to_user(
    State(app_state): State<Arc<AppState>>,
    context: RequestContext,
    Path((user_id, tenant_id)): Path<(i64, i64)>,
    axum::Json(req): axum::Json<BatchAssignRolesToUserRequest>,
) -> Result<Json<R<()>>, IdentityError> {
    app_state
        .user_role_service
        .batch_assign_roles_to_user(user_id, req.role_ids, tenant_id, Some(context.user_id))
        .await
        .map_err(to_err)?;
    Ok(Json(R::ok()))
}

/// 移除用户角色
#[utoipa::path(
    delete,
    path = "/api/v1/identity/users/{user_id}/tenants/{tenant_id}/roles/{role_id}",
    tag = "用户管理",
    params(
        ("user_id" = i64, Path, description = "用户 ID"),
        ("tenant_id" = i64, Path, description = "租户 ID"),
        ("role_id" = i64, Path, description = "角色 ID"),
    ),
    responses((status = 200, description = "移除成功", body = R<String>))
)]
pub async fn remove_role_from_user(
    State(app_state): State<Arc<AppState>>,
    Path((user_id, tenant_id, role_id)): Path<(i64, i64, i64)>,
) -> Result<Json<R<()>>, IdentityError> {
    app_state
        .user_role_service
        .remove_role_from_user(user_id, role_id, tenant_id)
        .await
        .map_err(to_err)?;
    Ok(Json(R::ok()))
}

/// 获取用户总数
#[utoipa::path(
    get,
    path = "/api/v1/identity/users/count",
    tag = "用户管理",
    responses((status = 200, description = "用户总数", body = R<i64>))
)]
pub async fn get_user_count(
    State(app_state): State<Arc<AppState>>,
) -> Result<Json<R<i64>>, IdentityError> {
    let count = app_state.user_service.get_user_count().await.map_err(to_err)?;
    Ok(Json(R::ok_with_data(count)))
}

/// 获取活跃用户数
#[utoipa::path(
    get,
    path = "/api/v1/identity/users/count/active",
    tag = "用户管理",
    params(("days" = Option<u32>, Query, description = "统计天数")),
    responses((status = 200, description = "活跃用户数", body = R<i64>))
)]
pub async fn get_active_user_count(
    State(app_state): State<Arc<AppState>>,
    Query(req): Query<GetActiveUserCountRequest>,
) -> Result<Json<R<i64>>, IdentityError> {
    let count = app_state.user_service.get_active_user_count(req.days).await.map_err(to_err)?;
    Ok(Json(R::ok_with_data(count)))
}
