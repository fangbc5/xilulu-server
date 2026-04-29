use super::model::dto::{
    AddEmployeePositionRequest, AddEmployeeToDepartmentRequest, CreateEmployeeRequest,
    EmployeeResponse, ListEmployeesQuery, UpdateEmployeeRequest,
};
use super::model::entity::Employee;
use crate::error::OrganizationError;
use crate::middleware::CurrentUser;
use crate::state::AppState;
use axum::extract::{Path, Query, State};
use axum::Json;
use fbc_starter::base::CursorPageBaseResp;
use fbc_starter::R;
use std::sync::Arc;
use validator::Validate;

/// 转换实体为响应
fn to_response(emp: Employee) -> EmployeeResponse {
    EmployeeResponse {
        id: emp.id.unwrap_or(0),
        tenant_id: emp.tenant_id,
        org_id: emp.org_id,
        user_id: emp.user_id,
        employee_no: emp.employee_no,
        name: emp.name,
        avatar: emp.avatar,
        gender: emp.gender,
        mobile: emp.mobile,
        email: emp.email,
        hire_date: emp.hire_date,
        leave_date: emp.leave_date,
        status: emp.status,
        sort_order: emp.sort_order,
        primary_department: None, // TODO: 填充部门信息
        primary_position: None,   // TODO: 填充岗位信息
    }
}

/// 创建员工
///
/// POST /api/v1/team/employees
#[utoipa::path(
    post,
    path = "/api/v1/team/employees",
    tag = "员工管理",
    request_body = CreateEmployeeRequest,
    responses(
        (status = 200, description = "创建成功，返回员工 ID", body = R<i64>),
        (status = 400, description = "参数校验失败"),
    )
)]
pub async fn create_employee(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    Json(req): Json<CreateEmployeeRequest>,
) -> Result<Json<R<i64>>, OrganizationError> {
    req.validate()
        .map_err(|e| OrganizationError::ParamFormatError(e.to_string()))?;

    let id = state
        .employee_service
        .create(current_user.tenant_id, req, Some(current_user.user_id))
        .await?;

    Ok(Json(R::ok_with_data(id)))
}

/// 获取员工详情
///
/// GET /api/v1/team/employees/{id}
#[utoipa::path(
    get,
    path = "/api/v1/team/employees/{id}",
    tag = "员工管理",
    params(
        ("id" = i64, Path, description = "员工 ID"),
    ),
    responses(
        (status = 200, description = "员工详情", body = R<EmployeeResponse>),
        (status = 404, description = "员工不存在"),
    )
)]
pub async fn get_employee(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<Json<R<EmployeeResponse>>, OrganizationError> {
    let emp = state
        .employee_service
        .get_by_id(id)
        .await?;

    Ok(Json(R::ok_with_data(to_response(emp))))
}

/// 获取员工列表
///
/// GET /api/v1/team/employees
#[utoipa::path(
    get,
    path = "/api/v1/team/employees",
    tag = "员工管理",
    params(
        ("org_id" = i64, Query, description = "组织 ID（必填）"),
        ("department_id" = Option<i64>, Query, description = "部门 ID 筛选"),
        ("include_children" = Option<bool>, Query, description = "是否包含子部门员工"),
        ("position_id" = Option<i64>, Query, description = "岗位 ID 筛选"),
        ("status" = Option<i16>, Query, description = "状态筛选"),
        ("keyword" = Option<String>, Query, description = "搜索关键词"),
        ("page_size" = Option<u32>, Query, description = "每页条数"),
        ("cursor" = Option<u32>, Query, description = "页码"),
    ),
    responses(
        (status = 200, description = "员工列表（分页）", body = R<CursorPageBaseResp<EmployeeResponse>>),
    )
)]
pub async fn list_employees(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    Query(query): Query<ListEmployeesQuery>,
) -> Result<Json<R<CursorPageBaseResp<EmployeeResponse>>>, OrganizationError> {
    let page = query.page.cursor.unwrap_or(1);
    let page_size = query.page.page_size;

    let (emps, total) = state
        .employee_service
        .find_page(current_user.tenant_id, query)
        .await?;

    let responses: Vec<EmployeeResponse> = emps.into_iter().map(to_response).collect();
    let has_next = (page as i64 * page_size as i64) < total;
    let next_cursor = if has_next { Some(page + 1) } else { None };

    Ok(Json(R::ok_with_data(CursorPageBaseResp::init(
        next_cursor,
        has_next,
        responses,
        total,
    ))))
}

/// 更新员工
///
/// PUT /api/v1/team/employees/{id}
#[utoipa::path(
    put,
    path = "/api/v1/team/employees/{id}",
    tag = "员工管理",
    params(
        ("id" = i64, Path, description = "员工 ID"),
    ),
    request_body = UpdateEmployeeRequest,
    responses(
        (status = 200, description = "更新成功", body = R<String>),
        (status = 404, description = "员工不存在"),
    )
)]
pub async fn update_employee(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    Path(id): Path<i64>,
    Json(req): Json<UpdateEmployeeRequest>,
) -> Result<Json<R<()>>, OrganizationError> {
    req.validate()
        .map_err(|e| OrganizationError::ParamFormatError(e.to_string()))?;

    state
        .employee_service
        .update(id, req, Some(current_user.user_id))
        .await?;

    Ok(Json(R::ok()))
}

/// 删除员工
///
/// DELETE /api/v1/team/employees/{id}
#[utoipa::path(
    delete,
    path = "/api/v1/team/employees/{id}",
    tag = "员工管理",
    params(
        ("id" = i64, Path, description = "员工 ID"),
    ),
    responses(
        (status = 200, description = "删除成功", body = R<String>),
        (status = 404, description = "员工不存在"),
    )
)]
pub async fn delete_employee(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<Json<R<()>>, OrganizationError> {
    state
        .employee_service
        .delete(id)
        .await?;

    Ok(Json(R::ok()))
}

/// 添加员工到部门
///
/// POST /api/v1/team/employees/{id}/departments
#[utoipa::path(
    post,
    path = "/api/v1/team/employees/{id}/departments",
    tag = "员工关系",
    params(
        ("id" = i64, Path, description = "员工 ID"),
    ),
    request_body = AddEmployeeToDepartmentRequest,
    responses(
        (status = 200, description = "添加成功，返回关系 ID", body = R<i64>),
        (status = 404, description = "员工或部门不存在"),
        (status = 409, description = "关系已存在"),
    )
)]
pub async fn add_employee_to_department(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    Path(employee_id): Path<i64>,
    Json(req): Json<AddEmployeeToDepartmentRequest>,
) -> Result<Json<R<i64>>, OrganizationError> {
    let id = state
        .employee_department_service
        .add_to_department(
            current_user.tenant_id,
            employee_id,
            req.department_id,
            req.is_primary.unwrap_or(false),
            req.is_leader.unwrap_or(false),
            Some(current_user.user_id),
        )
        .await?;

    Ok(Json(R::ok_with_data(id)))
}

/// 从部门移除员工
///
/// DELETE /api/v1/team/employees/{employee_id}/departments/{department_id}
#[utoipa::path(
    delete,
    path = "/api/v1/team/employees/{employee_id}/departments/{department_id}",
    tag = "员工关系",
    params(
        ("employee_id" = i64, Path, description = "员工 ID"),
        ("department_id" = i64, Path, description = "部门 ID"),
    ),
    responses(
        (status = 200, description = "移除成功", body = R<String>),
        (status = 404, description = "关系不存在"),
    )
)]
pub async fn remove_employee_from_department(
    State(state): State<Arc<AppState>>,
    Path((employee_id, department_id)): Path<(i64, i64)>,
) -> Result<Json<R<()>>, OrganizationError> {
    state
        .employee_department_service
        .remove_from_department(employee_id, department_id)
        .await?;

    Ok(Json(R::ok()))
}

/// 添加员工岗位
///
/// POST /api/v1/team/employees/{id}/positions
#[utoipa::path(
    post,
    path = "/api/v1/team/employees/{id}/positions",
    tag = "员工关系",
    params(
        ("id" = i64, Path, description = "员工 ID"),
    ),
    request_body = AddEmployeePositionRequest,
    responses(
        (status = 200, description = "添加成功，返回关系 ID", body = R<i64>),
        (status = 404, description = "员工或岗位不存在"),
        (status = 409, description = "关系已存在"),
    )
)]
pub async fn add_employee_position(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    Path(employee_id): Path<i64>,
    Json(req): Json<AddEmployeePositionRequest>,
) -> Result<Json<R<i64>>, OrganizationError> {
    let id = state
        .employee_position_service
        .add_position(
            current_user.tenant_id,
            employee_id,
            req.position_id,
            req.is_primary.unwrap_or(false),
            Some(current_user.user_id),
        )
        .await?;

    Ok(Json(R::ok_with_data(id)))
}

/// 移除员工岗位
///
/// DELETE /api/v1/team/employees/{employee_id}/positions/{position_id}
#[utoipa::path(
    delete,
    path = "/api/v1/team/employees/{employee_id}/positions/{position_id}",
    tag = "员工关系",
    params(
        ("employee_id" = i64, Path, description = "员工 ID"),
        ("position_id" = i64, Path, description = "岗位 ID"),
    ),
    responses(
        (status = 200, description = "移除成功", body = R<String>),
        (status = 404, description = "关系不存在"),
    )
)]
pub async fn remove_employee_position(
    State(state): State<Arc<AppState>>,
    Path((employee_id, position_id)): Path<(i64, i64)>,
) -> Result<Json<R<()>>, OrganizationError> {
    state
        .employee_position_service
        .remove_position(employee_id, position_id)
        .await?;

    Ok(Json(R::ok()))
}
