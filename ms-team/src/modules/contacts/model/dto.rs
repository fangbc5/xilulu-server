use serde::{Deserialize, Serialize};

// ============================ 通讯录入口 ============================

/// 通讯录入口响应
#[derive(Debug, Serialize)]
pub struct ContactsEntryResponse {
    /// 组织信息
    pub organization: OrganizationBrief,
    /// 根部门列表
    pub departments: Vec<DepartmentSummary>,
    /// 组织总可见人数
    pub total_member_count: i64,
}

/// 组织简要信息
#[derive(Debug, Serialize)]
pub struct OrganizationBrief {
    pub id: i64,
    pub name: String,
    pub logo: Option<String>,
}

/// 部门摘要（用于入口和展开的子部门列表）
#[derive(Debug, Serialize)]
pub struct DepartmentSummary {
    pub id: i64,
    pub name: String,
    pub has_children: bool,
    pub member_count: i64,
    pub leader: Option<LeaderBrief>,
}

/// 负责人简要信息
#[derive(Debug, Serialize)]
pub struct LeaderBrief {
    pub id: i64,
    pub name: String,
    pub avatar: Option<String>,
}

// ============================ 部门展开 ============================

/// 部门展开响应
#[derive(Debug, Serialize)]
pub struct ContactsDepartmentResponse {
    /// 当前部门信息
    pub department: DepartmentInfo,
    /// 子部门列表
    pub children: Vec<DepartmentSummary>,
    /// 成员预览（默认 20 人，负责人置顶）
    pub members: Vec<MemberPreview>,
    /// 该部门直属可见成员总数
    pub direct_member_count: i64,
    /// 是否还有更多成员
    pub has_more_members: bool,
}

/// 部门信息
#[derive(Debug, Serialize)]
pub struct DepartmentInfo {
    pub id: i64,
    pub name: String,
    pub full_name: Option<String>,
}

/// 成员预览（用于部门展开和搜索结果）
#[derive(Debug, Serialize)]
pub struct MemberPreview {
    pub id: i64,
    pub name: String,
    pub avatar: Option<String>,
    pub department_title: Option<String>,
    pub mobile: Option<String>,
    pub email: Option<String>,
    pub is_leader: bool,
}

// ============================ 联系人详情 ============================

/// 联系人详情响应
#[derive(Debug, Serialize)]
pub struct ContactsEmployeeDetailResponse {
    pub id: i64,
    pub name: String,
    pub avatar: Option<String>,
    pub employee_no: Option<String>,
    pub gender: Option<i16>,
    pub mobile: Option<String>,
    pub email: Option<String>,
    pub status: Option<i16>,
    pub hire_date: Option<i64>,
    /// 所属部门列表
    pub departments: Vec<EmployeeDeptInfo>,
    /// 岗位列表
    pub positions: Vec<EmployeePosInfo>,
}

/// 员工部门信息
#[derive(Debug, Serialize)]
pub struct EmployeeDeptInfo {
    pub id: i64,
    pub name: String,
    pub full_name: Option<String>,
    pub is_primary: bool,
    pub is_leader: bool,
}

/// 员工岗位信息
#[derive(Debug, Serialize)]
pub struct EmployeePosInfo {
    pub id: i64,
    pub name: String,
    pub level: Option<i32>,
    pub is_primary: bool,
}

// ============================ 全局搜索 ============================

/// 搜索响应
#[derive(Debug, Serialize)]
pub struct ContactsSearchResponse {
    /// 搜索结果列表
    pub items: Vec<MemberPreview>,
    /// 估算命中总数
    pub estimated_total: u64,
    /// 是否还有下一页
    pub has_next: bool,
    /// 是否降级到 MySQL 搜索
    pub degraded: bool,
}

// ============================ 部门成员分页 ============================

/// 部门成员分页响应
#[derive(Debug, Serialize)]
pub struct ContactsMemberPageResponse {
    /// 成员列表
    pub items: Vec<MemberPreview>,
    /// 总数
    pub total: i64,
    /// 是否还有下一页
    pub has_next: bool,
}

// ============================ 请求参数 ============================

/// 通讯录入口请求参数
#[derive(Debug, Deserialize)]
pub struct ContactsEntryQuery {
    /// 组织 ID
    pub org_id: i64,
}

/// 搜索请求参数
#[derive(Debug, Deserialize)]
pub struct ContactsSearchQuery {
    /// 组织 ID
    pub org_id: i64,
    /// 搜索关键词
    pub keyword: String,
    /// 页码，默认 1
    pub page: Option<i64>,
    /// 每页条数，默认 20，最大 50
    pub page_size: Option<i64>,
}

/// 部门成员分页请求参数
#[derive(Debug, Deserialize)]
pub struct ContactsMembersQuery {
    /// 是否包含子部门成员，默认 false
    pub include_children: Option<bool>,
    /// 页码，默认 1
    pub page: Option<i64>,
    /// 每页条数，默认 20
    pub page_size: Option<i64>,
}

/// 索引重建请求参数
#[derive(Debug, Deserialize)]
pub struct RebuildIndexQuery {
    /// 限定重建范围的组织 ID，不传则重建全部
    pub org_id: Option<i64>,
}
