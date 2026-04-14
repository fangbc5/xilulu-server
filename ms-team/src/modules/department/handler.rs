use super::model::dto::{
    CreateDepartmentRequest, DepartmentResponse, DepartmentTreeNode, ListDepartmentsQuery,
    UpdateDepartmentRequest,
};
use super::model::entity::Department;
use crate::error::OrganizationError;
use crate::middleware::CurrentUser;
use crate::state::AppState;
use axum::extract::{Path, Query, State};
use axum::Json;
use fbc_starter::base::CursorPageBaseResp;
use fbc_starter::R;
use serde::Deserialize;
use std::sync::Arc;
use validator::Validate;

#[derive(Debug, Deserialize)]
pub struct GetRootsQuery {
    pub org_id: i64,
}

/// 转换实体为响应
fn to_response(dept: Department) -> DepartmentResponse {
    DepartmentResponse {
        id: dept.id.unwrap_or(0),
        tenant_id: dept.tenant_id,
        org_id: dept.org_id,
        parent_id: dept.parent_id,
        code: dept.code,
        name: dept.name,
        full_name: dept.full_name,
        path: dept.path,
        level: dept.level,
        leader_employee_id: dept.leader_employee_id,
        sort_order: dept.sort_order,
        status: dept.status,
        total_employee_count: None,
        employee_count: None,
    }
}

/// 创建部门
pub async fn create_department(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    Json(req): Json<CreateDepartmentRequest>,
) -> Result<Json<R<i64>>, OrganizationError> {
    req.validate()
        .map_err(|e| OrganizationError::ParamFormatError(e.to_string()))?;

    let id = state
        .department_service
        .create(current_user.tenant_id, req, Some(current_user.user_id))
        .await?;

    Ok(Json(R::ok_with_data(id)))
}

/// 获取部门详情
pub async fn get_department(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<Json<R<DepartmentResponse>>, OrganizationError> {
    let dept = state
        .department_service
        .get_by_id(id)
        .await?;

    Ok(Json(R::ok_with_data(to_response(dept))))
}

/// 获取部门列表
pub async fn list_departments(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    Query(query): Query<ListDepartmentsQuery>,
) -> Result<Json<R<CursorPageBaseResp<DepartmentResponse>>>, OrganizationError> {
    let page = query.page.cursor.unwrap_or(1);
    let page_size = query.page.page_size;

    let (depts, total) = state
        .department_service
        .find_page(current_user.tenant_id, query)
        .await?;

    let responses: Vec<DepartmentResponse> = depts.into_iter().map(to_response).collect();
    let has_next = (page as i64 * page_size as i64) < total;
    let next_cursor = if has_next { Some(page + 1) } else { None };

    Ok(Json(R::ok_with_data(CursorPageBaseResp::init(
        next_cursor,
        has_next,
        responses,
        total,
    ))))
}

/// 获取部门树
pub async fn get_department_tree(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListDepartmentsQuery>,
) -> Result<Json<R<Vec<DepartmentTreeNode>>>, OrganizationError> {
    let org_id = query.org_id.ok_or_else(|| {
        OrganizationError::BusinessConflict("org_id is required for tree view".to_string())
    })?;

    let tree = state
        .department_service
        .get_tree(org_id)
        .await?;

    Ok(Json(R::ok_with_data(tree)))
}

/// 更新部门
pub async fn update_department(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    Path(id): Path<i64>,
    Json(req): Json<UpdateDepartmentRequest>,
) -> Result<Json<R<()>>, OrganizationError> {
    req.validate()
        .map_err(|e| OrganizationError::ParamFormatError(e.to_string()))?;

    state
        .department_service
        .update(id, req, Some(current_user.user_id))
        .await?;

    Ok(Json(R::ok()))
}


/// 删除部门
pub async fn delete_department(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<Json<R<()>>, OrganizationError> {
    state
        .department_service
        .delete(id)
        .await?;

    Ok(Json(R::ok()))
}

/// 获取根部门列表（带员工数统计）
pub async fn get_roots(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    Query(query): Query<GetRootsQuery>,
) -> Result<Json<R<Vec<DepartmentResponse>>>, OrganizationError> {
    let roots = state
        .department_service
        .get_roots(query.org_id, current_user.tenant_id)
        .await?;

    Ok(Json(R::ok_with_data(roots)))
}

/// 获取子部门列表（带员工数统计）
pub async fn get_children(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    Path(parent_id): Path<i64>,
) -> Result<Json<R<Vec<DepartmentResponse>>>, OrganizationError> {
    let children = state
        .department_service
        .get_children(parent_id, current_user.tenant_id)
        .await?;

    Ok(Json(R::ok_with_data(children)))
}

