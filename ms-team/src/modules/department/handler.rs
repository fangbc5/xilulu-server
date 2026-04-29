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
///
/// POST /api/v1/team/departments
#[utoipa::path(
    post,
    path = "/api/v1/team/departments",
    tag = "部门管理",
    request_body = CreateDepartmentRequest,
    responses(
        (status = 200, description = "创建成功，返回部门 ID", body = R<i64>),
        (status = 400, description = "参数校验失败"),
    )
)]
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
///
/// GET /api/v1/team/departments/{id}
#[utoipa::path(
    get,
    path = "/api/v1/team/departments/{id}",
    tag = "部门管理",
    params(
        ("id" = i64, Path, description = "部门 ID"),
    ),
    responses(
        (status = 200, description = "部门详情", body = R<DepartmentResponse>),
        (status = 404, description = "部门不存在"),
    )
)]
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
///
/// GET /api/v1/team/departments
#[utoipa::path(
    get,
    path = "/api/v1/team/departments",
    tag = "部门管理",
    params(
        ("org_id" = Option<i64>, Query, description = "组织 ID"),
        ("parent_id" = Option<i64>, Query, description = "上级部门 ID"),
        ("keyword" = Option<String>, Query, description = "搜索关键词"),
        ("status" = Option<i16>, Query, description = "状态筛选"),
        ("page_size" = Option<u32>, Query, description = "每页条数"),
        ("cursor" = Option<u32>, Query, description = "页码"),
    ),
    responses(
        (status = 200, description = "部门列表（分页）", body = R<CursorPageBaseResp<DepartmentResponse>>),
    )
)]
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
///
/// GET /api/v1/team/departments/tree
#[utoipa::path(
    get,
    path = "/api/v1/team/departments/tree",
    tag = "部门管理",
    params(
        ("org_id" = Option<i64>, Query, description = "组织 ID（必填）"),
    ),
    responses(
        (status = 200, description = "部门树", body = R<Vec<DepartmentTreeNode>>),
    )
)]
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
///
/// PUT /api/v1/team/departments/{id}
#[utoipa::path(
    put,
    path = "/api/v1/team/departments/{id}",
    tag = "部门管理",
    params(
        ("id" = i64, Path, description = "部门 ID"),
    ),
    request_body = UpdateDepartmentRequest,
    responses(
        (status = 200, description = "更新成功", body = R<String>),
        (status = 404, description = "部门不存在"),
    )
)]
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
///
/// DELETE /api/v1/team/departments/{id}
#[utoipa::path(
    delete,
    path = "/api/v1/team/departments/{id}",
    tag = "部门管理",
    params(
        ("id" = i64, Path, description = "部门 ID"),
    ),
    responses(
        (status = 200, description = "删除成功", body = R<String>),
        (status = 404, description = "部门不存在"),
    )
)]
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
///
/// GET /api/v1/team/departments/roots
#[utoipa::path(
    get,
    path = "/api/v1/team/departments/roots",
    tag = "部门管理",
    params(
        ("org_id" = i64, Query, description = "组织 ID"),
    ),
    responses(
        (status = 200, description = "根部门列表", body = R<Vec<DepartmentResponse>>),
    )
)]
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
///
/// GET /api/v1/team/departments/{id}/children
#[utoipa::path(
    get,
    path = "/api/v1/team/departments/{id}/children",
    tag = "部门管理",
    params(
        ("id" = i64, Path, description = "父部门 ID"),
    ),
    responses(
        (status = 200, description = "子部门列表", body = R<Vec<DepartmentResponse>>),
    )
)]
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
