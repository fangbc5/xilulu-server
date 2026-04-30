use fbc_starter::base::CursorPageBaseReq;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

/// 创建部门请求
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateDepartmentRequest {
    /// 所属组织ID
    pub org_id: i64,
    /// 上级部门ID
    pub parent_id: Option<i64>,
    /// 部门编码
    #[validate(length(min = 1, max = 50, message = "部门编码长度必须在1-50之间"))]
    pub code: String,
    /// 部门名称
    #[validate(length(min = 1, max = 100, message = "部门名称长度必须在1-100之间"))]
    pub name: String,
    /// 部门负责人（员工ID）
    pub leader_employee_id: Option<i64>,
    /// 排序
    pub sort_order: Option<i32>,
}

/// 更新部门请求
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdateDepartmentRequest {
    /// 部门名称
    #[validate(length(min = 1, max = 100, message = "部门名称长度必须在1-100之间"))]
    pub name: Option<String>,
    /// 部门负责人（员工ID）
    pub leader_employee_id: Option<i64>,
    /// 排序
    pub sort_order: Option<i32>,
    /// 状态
    pub status: Option<i16>,
}

/// 部门响应
#[derive(Debug, Serialize, ToSchema)]
pub struct DepartmentResponse {
    pub id: i64,
    pub tenant_id: i64,
    pub org_id: i64,
    pub parent_id: Option<i64>,
    pub code: String,
    pub name: String,
    pub full_name: Option<String>,
    pub path: Option<String>,
    pub level: Option<i32>,
    pub leader_employee_id: Option<i64>,
    pub sort_order: Option<i32>,
    pub status: Option<i16>,
    /// ⭐ 部门及所有下属部门的总员工数（包含子部门）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_employee_count: Option<i64>,
    /// 直属部门的员工数
    #[serde(skip_serializing_if = "Option::is_none")]
    pub employee_count: Option<i64>,
}

/// 部门树节点
#[derive(Debug, Serialize, ToSchema)]
pub struct DepartmentTreeNode {
    pub department: DepartmentResponse,
    #[schema(no_recursion)]
    pub children: Vec<DepartmentTreeNode>,
}

/// 部门列表查询参数
#[derive(Debug, Deserialize)]
pub struct ListDepartmentsQuery {
    #[serde(flatten)]
    pub page: CursorPageBaseReq,
    /// 组织ID
    pub org_id: Option<i64>,
    /// 上级部门ID（不传则查询所有）
    pub parent_id: Option<i64>,
    /// 是否返回树形结构
    pub tree: Option<bool>,
    /// 搜索关键词
    pub keyword: Option<String>,
    /// 状态
    pub status: Option<i16>,
}
