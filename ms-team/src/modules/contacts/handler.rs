use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use fbc_starter::R;
use std::sync::Arc;

use crate::{error::OrganizationError, middleware::CurrentUser, state::AppState};
use super::model::dto::*;

/// 通讯录入口：组织信息 + 根部门列表
pub async fn contacts_entry(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    Query(query): Query<ContactsEntryQuery>,
) -> Result<Json<R<ContactsEntryResponse>>, OrganizationError> {
    let result = state.contacts_service.get_entry(current_user, query).await?;
    Ok(Json(R::ok_with_data(result)))
}

/// 部门展开：子部门 + 直属成员预览
pub async fn contacts_department(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    Path(dept_id): Path<i64>,
) -> Result<Json<R<ContactsDepartmentResponse>>, OrganizationError> {
    let result = state.contacts_service.get_department(current_user, dept_id).await?;
    Ok(Json(R::ok_with_data(result)))
}

/// 联系人详情：全部门关系 + 全岗位关系
pub async fn contacts_employee_detail(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    Path(employee_id): Path<i64>,
) -> Result<Json<R<ContactsEmployeeDetailResponse>>, OrganizationError> {
    let result = state.contacts_service.get_employee_detail(current_user, employee_id).await?;
    Ok(Json(R::ok_with_data(result)))
}

/// 全局搜索
pub async fn contacts_search(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    Query(query): Query<ContactsSearchQuery>,
) -> Result<Json<R<ContactsSearchResponse>>, OrganizationError> {
    let result = state.contacts_service.search(current_user, query).await?;
    Ok(Json(R::ok_with_data(result)))
}

/// 部门成员分页
pub async fn contacts_department_members(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    Path(dept_id): Path<i64>,
    Query(query): Query<ContactsMembersQuery>,
) -> Result<Json<R<ContactsMemberPageResponse>>, OrganizationError> {
    let result = state.contacts_service.get_department_members(current_user, dept_id, query).await?;
    Ok(Json(R::ok_with_data(result)))
}

/// 全量重建搜索索引（管理接口）
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
