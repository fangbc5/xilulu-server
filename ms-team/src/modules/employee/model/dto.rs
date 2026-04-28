use fbc_starter::base::CursorPageBaseReq;
use serde::{Deserialize, Serialize};
use validator::Validate;

/// 创建员工请求
#[derive(Debug, Deserialize, Validate)]
pub struct CreateEmployeeRequest {
    /// 所属组织ID
    pub org_id: i64,
    /// 关联的用户ID（ms-identity）
    pub user_id: i64,
    /// 员工工号
    #[validate(length(min = 1, max = 50, message = "员工工号长度必须在1-50之间"))]
    pub employee_no: String,
    /// 员工姓名
    #[validate(length(min = 1, max = 100, message = "员工姓名长度必须在1-100之间"))]
    pub name: String,
    /// 员工头像
    pub avatar: Option<String>,
    /// 性别：0-未知 1-男 2-女
    pub gender: Option<i16>,
    /// 工作手机
    #[validate(length(max = 20, message = "工作手机长度不能超过20"))]
    pub mobile: Option<String>,
    /// 工作邮箱
    pub email: Option<String>,
    /// 入职日期（毫秒时间戳）
    pub hire_date: Option<i64>,
    /// 主部门ID
    pub primary_department_id: Option<i64>,
    /// 主岗位ID
    pub primary_position_id: Option<i64>,
}

/// 更新员工请求
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateEmployeeRequest {
    /// 员工姓名
    #[validate(length(min = 1, max = 100, message = "员工姓名长度必须在1-100之间"))]
    pub name: Option<String>,
    /// 员工头像
    pub avatar: Option<String>,
    /// 性别
    pub gender: Option<i16>,
    /// 工作手机
    #[validate(length(max = 20, message = "工作手机长度不能超过20"))]
    pub mobile: Option<String>,
    /// 工作邮箱
    pub email: Option<String>,
    /// 入职日期（毫秒时间戳）
    pub hire_date: Option<i64>,
    /// 离职日期（毫秒时间戳）
    pub leave_date: Option<i64>,
    /// 状态
    pub status: Option<i16>,
    /// 排序
    pub sort_order: Option<i32>,
}

/// 员工响应
#[derive(Debug, Serialize)]
pub struct EmployeeResponse {
    pub id: i64,
    pub tenant_id: i64,
    pub org_id: i64,
    pub user_id: i64,
    pub employee_no: String,
    pub name: String,
    pub avatar: Option<String>,
    pub gender: Option<i16>,
    pub mobile: Option<String>,
    pub email: Option<String>,
    pub hire_date: Option<i64>,
    pub leave_date: Option<i64>,
    pub status: Option<i16>,
    pub sort_order: Option<i32>,
    /// 主部门
    pub primary_department: Option<DepartmentBrief>,
    /// 主岗位
    pub primary_position: Option<PositionBrief>,
}

/// 部门简要信息
#[derive(Debug, Serialize)]
pub struct DepartmentBrief {
    pub id: i64,
    pub name: String,
    pub full_name: Option<String>,
}

/// 岗位简要信息
#[derive(Debug, Serialize)]
pub struct PositionBrief {
    pub id: i64,
    pub name: String,
    pub level: Option<i32>,
}

/// 员工列表查询参数
#[derive(Debug, Deserialize)]
pub struct ListEmployeesQuery {
    /// 组织ID
    pub org_id: i64,
    /// 部门ID（查询该部门下的员工）
    pub department_id: Option<i64>,
    /// 是否包含子部门员工
    pub include_children: Option<bool>,
    /// 岗位ID
    pub position_id: Option<i64>,
    /// 状态
    pub status: Option<i16>,
    /// 搜索关键词
    pub keyword: Option<String>,
    #[serde(flatten)]
    pub page: CursorPageBaseReq,
}

/// 添加员工到部门请求
#[derive(Debug, Deserialize)]
pub struct AddEmployeeToDepartmentRequest {
    /// 部门ID
    pub department_id: i64,
    /// 是否主部门
    pub is_primary: Option<bool>,
    /// 是否部门负责人
    pub is_leader: Option<bool>,
}

/// 添加员工岗位请求
#[derive(Debug, Deserialize)]
pub struct AddEmployeePositionRequest {
    /// 岗位ID
    pub position_id: i64,
    /// 是否主岗位
    pub is_primary: Option<bool>,
}

/// 员工部门关系响应
#[derive(Debug, Serialize)]
pub struct EmployeeDepartmentResponse {
    pub id: i64,
    pub employee_id: i64,
    pub department_id: i64,
    pub department_name: String,
    pub department_full_name: Option<String>,
    pub is_primary: bool,
    pub is_leader: bool,
}

/// 员工岗位关系响应
#[derive(Debug, Serialize)]
pub struct EmployeePositionResponse {
    pub id: i64,
    pub employee_id: i64,
    pub position_id: i64,
    pub position_name: String,
    pub position_level: Option<i32>,
    pub is_primary: bool,
}
