// 权限相关 HTTP 处理器

use crate::context::RequestContext;
use crate::error::{IdentityError, to_err};
use crate::modules::auth::*;
use crate::state::AppState;
use axum::{
    extract::{Path, State},
    response::Json,
};
use fbc_starter::{base::CursorPageBaseResp, R};
use std::sync::Arc;

// ========== 角色相关 Handlers ==========

/// 获取角色信息
#[utoipa::path(get, path = "/api/v1/identity/roles/{id}", tag = "角色管理",
    params(("id" = i64, Path, description = "角色 ID")),
    responses((status = 200, description = "角色信息", body = R<RoleInfo>)))]
pub async fn get_role(
    State(app_state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<Json<R<RoleInfo>>, IdentityError> {
    let role = app_state.role_service.get_role_info(id).await.map_err(to_err)?;
    Ok(Json(R::ok_with_data(RoleInfo::from(role))))
}

/// 创建角色
#[utoipa::path(post, path = "/api/v1/identity/roles", tag = "角色管理",
    request_body = CreateRoleRequest,
    responses((status = 200, description = "创建成功", body = R<CreateRoleResponse>)))]
pub async fn create_role(
    State(app_state): State<Arc<AppState>>,
    context: RequestContext,
    axum::Json(req): axum::Json<CreateRoleRequest>,
) -> Result<Json<R<CreateRoleResponse>>, IdentityError> {
    let role_id = app_state
        .role_service
        .create_role(
            &req.name, &req.code, req.tenant_id, None, None,
            req.remarks.as_deref(), req.state, None, None, Some(context.user_id),
        )
        .await
        .map_err(to_err)?;
    Ok(Json(R::ok_with_data(CreateRoleResponse { role_id })))
}

/// 更新角色
#[utoipa::path(put, path = "/api/v1/identity/roles/{id}", tag = "角色管理",
    params(("id" = i64, Path, description = "角色 ID")),
    request_body = UpdateRoleRequest,
    responses((status = 200, description = "更新成功", body = R<String>)))]
pub async fn update_role(
    State(app_state): State<Arc<AppState>>,
    context: RequestContext,
    Path(id): Path<i64>,
    axum::Json(req): axum::Json<UpdateRoleRequest>,
) -> Result<Json<R<()>>, IdentityError> {
    app_state.role_service.update_role(id, req.name.as_deref(), req.remarks.as_deref(), req.state, None, Some(context.user_id)).await.map_err(to_err)?;
    Ok(Json(R::ok()))
}

/// 删除角色
#[utoipa::path(delete, path = "/api/v1/identity/roles/{id}", tag = "角色管理",
    params(("id" = i64, Path, description = "角色 ID")),
    responses((status = 200, description = "删除成功", body = R<String>)))]
pub async fn delete_role(
    State(app_state): State<Arc<AppState>>,
    context: RequestContext,
    Path(id): Path<i64>,
) -> Result<Json<R<()>>, IdentityError> {
    app_state.role_service.delete_role(id, Some(context.user_id)).await.map_err(to_err)?;
    Ok(Json(R::ok()))
}

/// 获取租户的角色列表
#[utoipa::path(get, path = "/api/v1/identity/roles/tenant/{tenant_id}", tag = "角色管理",
    params(("tenant_id" = i64, Path, description = "租户 ID")),
    responses((status = 200, description = "角色列表", body = R<Vec<RoleInfo>>)))]
pub async fn get_tenant_roles(
    State(app_state): State<Arc<AppState>>,
    Path(tenant_id): Path<i64>,
) -> Result<Json<R<Vec<RoleInfo>>>, IdentityError> {
    let roles = app_state.role_service.get_tenant_roles(tenant_id).await.map_err(to_err)?;
    let infos = roles.into_iter().map(RoleInfo::from).collect();
    Ok(Json(R::ok_with_data(infos)))
}

/// 获取角色列表（分页）
#[utoipa::path(get, path = "/api/v1/identity/roles", tag = "角色管理",
    params(
        ("cursor" = Option<u32>, Query, description = "游标"),
        ("page_size" = Option<u32>, Query, description = "每页条数"),
        ("tenant_id" = Option<i64>, Query, description = "租户 ID"),
    ),
    responses((status = 200, description = "角色列表", body = R<CursorPageBaseResp<RoleInfo>>)))]
pub async fn list_roles(
    State(app_state): State<Arc<AppState>>,
    axum::extract::Query(req): axum::extract::Query<ListRolesRequest>,
) -> Result<Json<R<CursorPageBaseResp<RoleInfo>>>, IdentityError> {
    let page = req.page.cursor.unwrap_or(1);
    let page_size = req.page.page_size;
    let (roles, total) = app_state.role_service.list_roles(page, page_size, req.tenant_id).await.map_err(to_err)?;
    let list: Vec<RoleInfo> = roles.into_iter().map(RoleInfo::from).collect();
    let next_cursor = if list.len() as u32 >= page_size { Some((page as u32) * page_size) } else { None };
    let is_last = next_cursor.is_none() || ((page as i64) * (page_size as i64)) >= total;
    Ok(Json(R::ok_with_data(CursorPageBaseResp::init(next_cursor, is_last, list, total))))
}

// ========== 资源相关 Handlers ==========

/// 获取资源信息
#[utoipa::path(get, path = "/api/v1/identity/resources/{id}", tag = "资源管理",
    params(("id" = i64, Path, description = "资源 ID")),
    responses((status = 200, description = "资源信息", body = R<ResourceInfo>)))]
pub async fn get_resource(
    State(app_state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<Json<R<ResourceInfo>>, IdentityError> {
    let resource = app_state.resource_service.get_resource_info(id).await.map_err(to_err)?;
    Ok(Json(R::ok_with_data(ResourceInfo::from(resource))))
}

/// 创建资源
#[utoipa::path(post, path = "/api/v1/identity/resources", tag = "资源管理",
    request_body = CreateResourceRequest,
    responses((status = 200, description = "创建成功", body = R<CreateResourceResponse>)))]
pub async fn create_resource(
    State(app_state): State<Arc<AppState>>,
    context: RequestContext,
    axum::Json(req): axum::Json<CreateResourceRequest>,
) -> Result<Json<R<CreateResourceResponse>>, IdentityError> {
    let resource_id = app_state.resource_service.create_resource(
        req.application_id, &req.code, &req.name, req.parent_id,
        req.resource_type.as_deref(), None, req.describe_.as_deref(),
        req.path.as_deref(), None, None, None, Some(context.user_id),
    ).await.map_err(to_err)?;
    Ok(Json(R::ok_with_data(CreateResourceResponse { resource_id })))
}

/// 更新资源
#[utoipa::path(put, path = "/api/v1/identity/resources/{id}", tag = "资源管理",
    params(("id" = i64, Path, description = "资源 ID")),
    request_body = UpdateResourceRequest,
    responses((status = 200, description = "更新成功", body = R<String>)))]
pub async fn update_resource(
    State(app_state): State<Arc<AppState>>,
    context: RequestContext,
    Path(id): Path<i64>,
    axum::Json(req): axum::Json<UpdateResourceRequest>,
) -> Result<Json<R<()>>, IdentityError> {
    app_state.resource_service.update_resource(
        id, req.name.as_deref(), req.describe_.as_deref(), req.path.as_deref(),
        None, None, None, Some(context.user_id),
    ).await.map_err(to_err)?;
    Ok(Json(R::ok()))
}

/// 删除资源
#[utoipa::path(delete, path = "/api/v1/identity/resources/{id}", tag = "资源管理",
    params(("id" = i64, Path, description = "资源 ID")),
    responses((status = 200, description = "删除成功", body = R<String>)))]
pub async fn delete_resource(
    State(app_state): State<Arc<AppState>>,
    context: RequestContext,
    Path(id): Path<i64>,
) -> Result<Json<R<()>>, IdentityError> {
    app_state.resource_service.delete_resource(id, Some(context.user_id)).await.map_err(to_err)?;
    Ok(Json(R::ok()))
}

/// 获取应用下的资源列表
#[utoipa::path(get, path = "/api/v1/identity/resources/application/{app_id}", tag = "资源管理",
    params(("app_id" = i64, Path, description = "应用 ID")),
    responses((status = 200, description = "资源列表", body = R<Vec<ResourceInfo>>)))]
pub async fn get_application_resources(
    State(app_state): State<Arc<AppState>>,
    Path(app_id): Path<i64>,
) -> Result<Json<R<Vec<ResourceInfo>>>, IdentityError> {
    let resources = app_state.resource_service.get_application_resources(app_id).await.map_err(to_err)?;
    let infos = resources.into_iter().map(ResourceInfo::from).collect();
    Ok(Json(R::ok_with_data(infos)))
}

/// 获取资源列表（分页）
#[utoipa::path(get, path = "/api/v1/identity/resources", tag = "资源管理",
    params(
        ("cursor" = Option<u32>, Query, description = "游标"),
        ("page_size" = Option<u32>, Query, description = "每页条数"),
        ("application_id" = Option<i64>, Query, description = "应用 ID"),
        ("tenant_id" = Option<i64>, Query, description = "租户 ID"),
        ("search_key" = Option<String>, Query, description = "搜索关键词"),
    ),
    responses((status = 200, description = "资源列表", body = R<CursorPageBaseResp<ResourceInfo>>)))]
pub async fn list_resources(
    State(app_state): State<Arc<AppState>>,
    axum::extract::Query(req): axum::extract::Query<ListResourcesRequest>,
) -> Result<Json<R<CursorPageBaseResp<ResourceInfo>>>, IdentityError> {
    let page = req.page.cursor.unwrap_or(1);
    let page_size = req.page.page_size;
    let (resources, total) = app_state.resource_service.list_resources(page, page_size, req.application_id, req.tenant_id, req.search_key.as_deref()).await.map_err(to_err)?;
    let list: Vec<ResourceInfo> = resources.into_iter().map(ResourceInfo::from).collect();
    let next_cursor = if list.len() as u32 >= page_size { Some((page as u32) * page_size) } else { None };
    let is_last = next_cursor.is_none() || ((page as i64) * (page_size as i64)) >= total;
    Ok(Json(R::ok_with_data(CursorPageBaseResp::init(next_cursor, is_last, list, total))))
}

/// 获取当前用户的菜单资源
#[utoipa::path(get, path = "/api/v1/identity/resources/menus", tag = "资源管理",
    params(("application_id" = i64, Query, description = "应用 ID")),
    responses((status = 200, description = "菜单列表", body = R<Vec<ResourceInfo>>)))]
pub async fn get_current_user_menus(
    State(app_state): State<Arc<AppState>>,
    context: RequestContext,
    axum::extract::Query(req): axum::extract::Query<GetUserMenusRequest>,
) -> Result<Json<R<Vec<ResourceInfo>>>, IdentityError> {
    let user_id = context.user_id;
    let tenant_id = context.tenant_id.ok_or_else(|| IdentityError::InvalidParam("缺少租户ID".to_string()))?;
    let resources = app_state.permission_service.get_user_menus(user_id, tenant_id, req.application_id).await.map_err(to_err)?;
    let list: Vec<ResourceInfo> = resources.into_iter().map(ResourceInfo::from).collect();
    Ok(Json(R::ok_with_data(list)))
}

/// 获取当前用户在指定菜单下的子资源，并按类型分类
#[utoipa::path(get, path = "/api/v1/identity/resources/menu-resources", tag = "资源管理",
    params(
        ("application_id" = i64, Query, description = "应用 ID"),
        ("menu_id" = i64, Query, description = "菜单资源 ID"),
    ),
    responses((status = 200, description = "菜单子资源", body = R<MenuResourcesByType>)))]
pub async fn get_current_user_menu_resources(
    State(app_state): State<Arc<AppState>>,
    context: RequestContext,
    axum::extract::Query(req): axum::extract::Query<GetMenuResourcesRequest>,
) -> Result<Json<R<MenuResourcesByType>>, IdentityError> {
    let user_id = context.user_id;
    let tenant_id = context.tenant_id.ok_or_else(|| IdentityError::InvalidParam("缺少租户ID".to_string()))?;
    let resources = app_state.permission_service.get_user_menu_resources(user_id, tenant_id, req.application_id, req.menu_id).await.map_err(to_err)?;
    let mut resp = MenuResourcesByType::default();
    for res in resources {
        let info = ResourceInfo::from(res);
        match info.resource_type.as_deref() {
            Some("20") => resp.menus.push(info),
            Some("40") => resp.buttons.push(info),
            Some("50") => resp.fields.push(info),
            Some("60") => resp.data.push(info),
            _ => resp.buttons.push(info),
        }
    }
    Ok(Json(R::ok_with_data(resp)))
}

// ========== 应用相关 Handlers ==========

/// 获取应用信息
#[utoipa::path(get, path = "/api/v1/identity/applications/{id}", tag = "应用管理",
    params(("id" = i64, Path, description = "应用 ID")),
    responses((status = 200, description = "应用信息", body = R<ApplicationInfo>)))]
pub async fn get_application(
    State(app_state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<Json<R<ApplicationInfo>>, IdentityError> {
    let app = app_state.application_service.get_application_info(id).await.map_err(to_err)?;
    Ok(Json(R::ok_with_data(ApplicationInfo::from(app))))
}

/// 创建应用
#[utoipa::path(post, path = "/api/v1/identity/applications", tag = "应用管理",
    request_body = CreateApplicationRequest,
    responses((status = 200, description = "创建成功", body = R<CreateApplicationResponse>)))]
pub async fn create_application(
    State(app_state): State<Arc<AppState>>,
    context: RequestContext,
    axum::Json(req): axum::Json<CreateApplicationRequest>,
) -> Result<Json<R<CreateApplicationResponse>>, IdentityError> {
    let app_id = app_state.application_service.create_application(
        &req.name, &req.app_key, req.r#type.as_deref(), req.app_secret.as_deref(),
        req.version.as_deref(), req.redirect.as_deref(), req.introduce.as_deref(),
        req.remark.as_deref(), req.url.as_deref(), req.is_general, req.is_visible,
        req.sort_value, Some(context.user_id),
    ).await.map_err(to_err)?;
    Ok(Json(R::ok_with_data(CreateApplicationResponse { application_id: app_id })))
}

/// 更新应用
#[utoipa::path(put, path = "/api/v1/identity/applications/{id}", tag = "应用管理",
    params(("id" = i64, Path, description = "应用 ID")),
    request_body = UpdateApplicationRequest,
    responses((status = 200, description = "更新成功", body = R<String>)))]
pub async fn update_application(
    State(app_state): State<Arc<AppState>>,
    context: RequestContext,
    Path(id): Path<i64>,
    axum::Json(req): axum::Json<UpdateApplicationRequest>,
) -> Result<Json<R<()>>, IdentityError> {
    app_state.application_service.update_application(
        id, req.name.as_deref(), req.r#type.as_deref(), req.version.as_deref(),
        req.redirect.as_deref(), req.introduce.as_deref(), req.remark.as_deref(),
        req.url.as_deref(), req.is_general, req.is_visible, req.sort_value, Some(context.user_id),
    ).await.map_err(to_err)?;
    Ok(Json(R::ok()))
}

/// 删除应用
#[utoipa::path(delete, path = "/api/v1/identity/applications/{id}", tag = "应用管理",
    params(("id" = i64, Path, description = "应用 ID")),
    responses((status = 200, description = "删除成功", body = R<String>)))]
pub async fn delete_application(
    State(app_state): State<Arc<AppState>>,
    context: RequestContext,
    Path(id): Path<i64>,
) -> Result<Json<R<()>>, IdentityError> {
    app_state.application_service.delete_application(id, Some(context.user_id)).await.map_err(to_err)?;
    Ok(Json(R::ok()))
}

/// 获取应用列表（分页）
#[utoipa::path(get, path = "/api/v1/identity/applications", tag = "应用管理",
    params(
        ("cursor" = Option<u32>, Query, description = "游标"),
        ("page_size" = Option<u32>, Query, description = "每页条数"),
        ("search_key" = Option<String>, Query, description = "搜索关键词"),
    ),
    responses((status = 200, description = "应用列表", body = R<CursorPageBaseResp<ApplicationInfo>>)))]
pub async fn list_applications(
    State(app_state): State<Arc<AppState>>,
    axum::extract::Query(req): axum::extract::Query<ListApplicationsRequest>,
) -> Result<Json<R<CursorPageBaseResp<ApplicationInfo>>>, IdentityError> {
    let page = req.page.cursor.unwrap_or(1);
    let page_size = req.page.page_size;
    let (apps, total) = app_state.application_service.list_applications(page, page_size, req.search_key.as_deref()).await.map_err(to_err)?;
    let list: Vec<ApplicationInfo> = apps.into_iter().map(ApplicationInfo::from).collect();
    let next_cursor = if list.len() as u32 >= page_size { Some((page as u32) * page_size) } else { None };
    let has_next = next_cursor.is_some() && ((page as i64) * (page_size as i64)) < total;
    Ok(Json(R::ok_with_data(CursorPageBaseResp::init(next_cursor, has_next, list, total))))
}

// ========== 权限相关 Handlers ==========

/// 分配资源到角色
#[utoipa::path(post, path = "/api/v1/identity/roles/{id}/resources", tag = "权限管理",
    params(("id" = i64, Path, description = "角色 ID")),
    request_body = AssignResourceToRoleRequest,
    responses((status = 200, description = "分配成功", body = R<String>)))]
pub async fn assign_resource_to_role(
    State(app_state): State<Arc<AppState>>,
    context: RequestContext,
    Path(role_id): Path<i64>,
    axum::Json(req): axum::Json<AssignResourceToRoleRequest>,
) -> Result<Json<R<()>>, IdentityError> {
    app_state.permission_service.assign_resource_to_role(role_id, req.resource_id, Some(context.user_id)).await.map_err(to_err)?;
    Ok(Json(R::ok()))
}

/// 从角色移除资源
#[utoipa::path(delete, path = "/api/v1/identity/roles/{id}/resources", tag = "权限管理",
    params(("id" = i64, Path, description = "角色 ID")),
    responses((status = 200, description = "移除成功", body = R<String>)))]
pub async fn remove_resource_from_role(
    State(app_state): State<Arc<AppState>>,
    Path(role_id): Path<i64>,
    axum::Json(req): axum::Json<serde_json::Value>,
) -> Result<Json<R<()>>, IdentityError> {
    let resource_id = req.get("resource_id").and_then(|v| v.as_i64()).unwrap_or(0);
    app_state.permission_service.remove_resource_from_role(role_id, resource_id).await.map_err(to_err)?;
    Ok(Json(R::ok()))
}

/// 获取角色的资源列表
#[utoipa::path(get, path = "/api/v1/identity/roles/{id}/resources", tag = "权限管理",
    params(("id" = i64, Path, description = "角色 ID")),
    responses((status = 200, description = "资源列表", body = R<Vec<ResourceInfo>>)))]
pub async fn get_role_resources(
    State(app_state): State<Arc<AppState>>,
    Path(role_id): Path<i64>,
) -> Result<Json<R<Vec<ResourceInfo>>>, IdentityError> {
    let resources = app_state.permission_service.get_role_resources(role_id).await.map_err(to_err)?;
    let infos = resources.into_iter().map(ResourceInfo::from).collect();
    Ok(Json(R::ok_with_data(infos)))
}

/// 检查权限（暂时禁用，等待重新设计）
#[utoipa::path(post, path = "/api/v1/identity/check-permission", tag = "权限管理",
    request_body = CheckPermissionRequest,
    responses((status = 200, description = "权限检查结果", body = R<CheckPermissionResponse>)))]
pub async fn check_permission(
    State(_app_state): State<Arc<AppState>>,
    axum::extract::Extension(_context): axum::extract::Extension<crate::context::RequestContext>,
    axum::Json(_req): axum::Json<CheckPermissionRequest>,
) -> Json<R<CheckPermissionResponse>> {
    // TODO: 权限检查功能需要重新设计，暂时返回 false
    Json(R::ok_with_data(CheckPermissionResponse { allowed: false }))
}
