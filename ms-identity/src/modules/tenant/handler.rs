// 租户相关 HTTP 处理器

use crate::context::RequestContext;
use crate::error::{IdentityError, to_err};
use crate::modules::auth::ApplicationInfo;
use crate::modules::tenant::*;
use crate::state::AppState;
use axum::{
    extract::{Path, Query, State},
    response::Json,
};
use fbc_starter::{base::CursorPageBaseResp, R};
use std::sync::Arc;

/// 获取租户列表（分页）
#[utoipa::path(get, path = "/api/v1/identity/tenants", tag = "租户管理",
    params(
        ("cursor" = Option<u32>, Query, description = "游标"),
        ("page_size" = Option<u32>, Query, description = "每页条数"),
        ("search_key" = Option<String>, Query, description = "搜索关键词"),
    ),
    responses((status = 200, description = "租户列表", body = R<CursorPageBaseResp<TenantInfo>>)))]
pub async fn list_tenants(
    State(app_state): State<Arc<AppState>>,
    Query(req): Query<ListTenantsRequest>,
) -> Result<Json<R<CursorPageBaseResp<TenantInfo>>>, IdentityError> {
    let page = req.page.cursor.unwrap_or(1);
    let page_size = req.page.page_size;
    let (tenants, total) = app_state.tenant_service.list_tenants(page, page_size, req.search_key.as_deref()).await.map_err(to_err)?;
    let list: Vec<TenantInfo> = tenants.into_iter().map(TenantInfo::from).collect();
    let next_cursor = if list.len() as u32 >= page_size { Some((page as u32) * page_size) } else { None };
    let has_next = next_cursor.is_some() && ((page as i64) * (page_size as i64)) < total;
    Ok(Json(R::ok_with_data(CursorPageBaseResp::init(next_cursor, has_next, list, total))))
}

/// 获取租户信息
#[utoipa::path(get, path = "/api/v1/identity/tenants/{id}", tag = "租户管理",
    params(("id" = i64, Path, description = "租户 ID")),
    responses((status = 200, description = "租户信息", body = R<TenantInfo>)))]
pub async fn get_tenant(
    State(app_state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<Json<R<TenantInfo>>, IdentityError> {
    let tenant = app_state.tenant_service.get_tenant_info(id).await.map_err(to_err)?;
    Ok(Json(R::ok_with_data(TenantInfo::from(tenant))))
}

/// 创建租户
#[utoipa::path(post, path = "/api/v1/identity/tenants", tag = "租户管理",
    request_body = CreateTenantRequest,
    responses((status = 200, description = "创建成功", body = R<CreateTenantResponse>)))]
pub async fn create_tenant(
    State(app_state): State<Arc<AppState>>,
    context: RequestContext,
    axum::Json(req): axum::Json<CreateTenantRequest>,
) -> Result<Json<R<CreateTenantResponse>>, IdentityError> {
    let tenant_id = app_state.tenant_service.create_tenant(
        &req.name, &req.contact_name, req.contact_mobile.as_deref(),
        req.package_id, req.expire_time, req.account_count,
        req.website.as_deref(), Some(context.user_id),
        req.tenant_type,
    ).await.map_err(to_err)?;
    Ok(Json(R::ok_with_data(CreateTenantResponse { tenant_id })))
}

/// 更新租户
#[utoipa::path(put, path = "/api/v1/identity/tenants/{id}", tag = "租户管理",
    params(("id" = i64, Path, description = "租户 ID")),
    request_body = UpdateTenantRequest,
    responses((status = 200, description = "更新成功", body = R<String>)))]
pub async fn update_tenant(
    State(app_state): State<Arc<AppState>>,
    context: RequestContext,
    Path(id): Path<i64>,
    axum::Json(req): axum::Json<UpdateTenantRequest>,
) -> Result<Json<R<()>>, IdentityError> {
    app_state.tenant_service.update_tenant(id, req.name.as_deref(), req.contact_name.as_deref(), req.contact_mobile.as_deref(), req.website.as_deref(), req.status, Some(context.user_id)).await.map_err(to_err)?;
    Ok(Json(R::ok()))
}

/// 删除租户
#[utoipa::path(delete, path = "/api/v1/identity/tenants/{id}", tag = "租户管理",
    params(("id" = i64, Path, description = "租户 ID")),
    responses((status = 200, description = "删除成功", body = R<String>)))]
pub async fn delete_tenant(
    State(app_state): State<Arc<AppState>>,
    context: RequestContext,
    Path(id): Path<i64>,
) -> Result<Json<R<()>>, IdentityError> {
    app_state.tenant_service.delete_tenant(id, Some(context.user_id)).await.map_err(to_err)?;
    Ok(Json(R::ok()))
}

/// 添加应用到租户
#[utoipa::path(post, path = "/api/v1/identity/tenants/{id}/applications", tag = "租户管理",
    params(("id" = i64, Path, description = "租户 ID")),
    request_body = AddApplicationToTenantRequest,
    responses((status = 200, description = "添加成功", body = R<String>)))]
pub async fn add_application_to_tenant(
    State(app_state): State<Arc<AppState>>,
    context: RequestContext,
    Path(tenant_id): Path<i64>,
    axum::Json(req): axum::Json<AddApplicationToTenantRequest>,
) -> Result<Json<R<()>>, IdentityError> {
    app_state.tenant_application_service.add_application_to_tenant(tenant_id, req.application_id, None, Some(context.user_id)).await.map_err(to_err)?;
    Ok(Json(R::ok()))
}

/// 从租户移除应用
#[utoipa::path(delete, path = "/api/v1/identity/tenants/{id}/applications", tag = "租户管理",
    params(("id" = i64, Path, description = "租户 ID")),
    responses((status = 200, description = "移除成功", body = R<String>)))]
pub async fn remove_application_from_tenant(
    State(app_state): State<Arc<AppState>>,
    Path(tenant_id): Path<i64>,
    axum::Json(req): axum::Json<serde_json::Value>,
) -> Result<Json<R<()>>, IdentityError> {
    let application_id = req.get("application_id").and_then(|v| v.as_i64()).unwrap_or(0);
    app_state.tenant_application_service.remove_application_from_tenant(tenant_id, application_id).await.map_err(to_err)?;
    Ok(Json(R::ok()))
}

/// 获取租户的应用列表
#[utoipa::path(get, path = "/api/v1/identity/tenants/{id}/applications", tag = "租户管理",
    params(("id" = i64, Path, description = "租户 ID")),
    responses((status = 200, description = "应用列表", body = R<Vec<ApplicationInfo>>)))]
pub async fn get_tenant_applications(
    State(app_state): State<Arc<AppState>>,
    Path(tenant_id): Path<i64>,
) -> Result<Json<R<Vec<ApplicationInfo>>>, IdentityError> {
    let infos = app_state.tenant_application_service.get_tenant_applications(tenant_id).await.map_err(to_err)?;
    Ok(Json(R::ok_with_data(infos)))
}

/// 获取租户总数
#[utoipa::path(get, path = "/api/v1/identity/tenants/count", tag = "租户管理",
    responses((status = 200, description = "租户总数", body = R<i64>)))]
pub async fn get_tenant_count(
    State(app_state): State<Arc<AppState>>,
) -> Result<Json<R<i64>>, IdentityError> {
    let count = app_state.tenant_service.get_tenant_count().await.map_err(to_err)?;
    Ok(Json(R::ok_with_data(count)))
}
