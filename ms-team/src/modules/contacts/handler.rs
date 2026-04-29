use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use fbc_starter::R;
use std::sync::Arc;

use super::model::dto::*;
use crate::{error::OrganizationError, middleware::CurrentUser, state::AppState};

/// 通讯录入口：组织信息 + 根部门列表
///
/// GET /api/v1/team/contacts/entry
#[utoipa::path(
    get,
    path = "/api/v1/team/contacts/entry",
    tag = "通讯录",
    params(
        ("org_id" = i64, Query, description = "组织 ID"),
    ),
    responses(
        (status = 200, description = "通讯录入口数据", body = R<ContactsEntryResponse>),
    )
)]
pub async fn contacts_entry(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    Query(query): Query<ContactsEntryQuery>,
) -> Result<Json<R<ContactsEntryResponse>>, OrganizationError> {
    let result = state
        .contacts_service
        .get_entry(current_user, query)
        .await?;
    Ok(Json(R::ok_with_data(result)))
}

/// 部门展开：子部门 + 直属成员预览
///
/// GET /api/v1/team/contacts/departments/{dept_id}
#[utoipa::path(
    get,
    path = "/api/v1/team/contacts/departments/{dept_id}",
    tag = "通讯录",
    params(
        ("dept_id" = i64, Path, description = "部门 ID"),
    ),
    responses(
        (status = 200, description = "部门展开数据", body = R<ContactsDepartmentResponse>),
        (status = 404, description = "部门不存在"),
    )
)]
pub async fn contacts_department(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    Path(dept_id): Path<i64>,
) -> Result<Json<R<ContactsDepartmentResponse>>, OrganizationError> {
    let result = state
        .contacts_service
        .get_department(current_user, dept_id)
        .await?;
    Ok(Json(R::ok_with_data(result)))
}

/// 联系人详情：全部门关系 + 全岗位关系
///
/// GET /api/v1/team/contacts/employees/{id}
#[utoipa::path(
    get,
    path = "/api/v1/team/contacts/employees/{id}",
    tag = "通讯录",
    params(
        ("id" = i64, Path, description = "员工 ID"),
    ),
    responses(
        (status = 200, description = "联系人详情", body = R<ContactsEmployeeDetailResponse>),
        (status = 404, description = "员工不存在"),
    )
)]
pub async fn contacts_employee_detail(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    Path(employee_id): Path<i64>,
) -> Result<Json<R<ContactsEmployeeDetailResponse>>, OrganizationError> {
    let result = state
        .contacts_service
        .get_employee_detail(current_user, employee_id)
        .await?;
    Ok(Json(R::ok_with_data(result)))
}

/// 全局搜索
///
/// GET /api/v1/team/contacts/search
#[utoipa::path(
    get,
    path = "/api/v1/team/contacts/search",
    tag = "通讯录",
    params(
        ("org_id" = i64, Query, description = "组织 ID"),
        ("keyword" = String, Query, description = "搜索关键词"),
        ("page" = Option<i64>, Query, description = "页码，默认 1"),
        ("page_size" = Option<i64>, Query, description = "每页条数，默认 20，最大 50"),
    ),
    responses(
        (status = 200, description = "搜索结果", body = R<ContactsSearchResponse>),
    )
)]
pub async fn contacts_search(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    Query(query): Query<ContactsSearchQuery>,
) -> Result<Json<R<ContactsSearchResponse>>, OrganizationError> {
    let result = state.contacts_service.search(current_user, query).await?;
    Ok(Json(R::ok_with_data(result)))
}

/// 部门成员分页
///
/// GET /api/v1/team/contacts/departments/{dept_id}/members
#[utoipa::path(
    get,
    path = "/api/v1/team/contacts/departments/{dept_id}/members",
    tag = "通讯录",
    params(
        ("dept_id" = i64, Path, description = "部门 ID"),
        ("include_children" = Option<bool>, Query, description = "是否包含子部门成员，默认 false"),
        ("page" = Option<i64>, Query, description = "页码，默认 1"),
        ("page_size" = Option<i64>, Query, description = "每页条数，默认 20"),
    ),
    responses(
        (status = 200, description = "部门成员分页", body = R<ContactsMemberPageResponse>),
    )
)]
pub async fn contacts_department_members(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    Path(dept_id): Path<i64>,
    Query(query): Query<ContactsMembersQuery>,
) -> Result<Json<R<ContactsMemberPageResponse>>, OrganizationError> {
    let result = state
        .contacts_service
        .get_department_members(current_user, dept_id, query)
        .await?;
    Ok(Json(R::ok_with_data(result)))
}

/// 全量重建搜索索引（管理接口）
///
/// POST /api/v1/team/admin/search/rebuild
#[utoipa::path(
    post,
    path = "/api/v1/team/admin/search/rebuild",
    tag = "管理",
    params(
        ("org_id" = Option<i64>, Query, description = "限定重建范围的组织 ID，不传则重建全部"),
    ),
    responses(
        (status = 202, description = "重建任务已提交", body = R<String>),
    )
)]
pub async fn rebuild_search_index(
    State(state): State<Arc<AppState>>,
    _current_user: CurrentUser,
    Query(query): Query<RebuildIndexQuery>,
) -> Result<(StatusCode, Json<R<()>>), OrganizationError> {
    // 异步后台执行重建
    let contacts_service = state.contacts_service.clone();
    tokio::spawn(async move {
        let _ = contacts_service.rebuild_index(query.org_id).await;
    });

    Ok((StatusCode::ACCEPTED, Json(R::ok())))
}
