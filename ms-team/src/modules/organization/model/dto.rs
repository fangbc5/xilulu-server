use fbc_starter::base::CursorPageBaseReq;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

/// 创建组织请求
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateOrganizationRequest {
    /// 上级组织ID
    pub parent_id: Option<i64>,
    /// 组织编码
    #[validate(length(min = 2, max = 50, message = "组织编码长度必须在2-50之间"))]
    pub code: String,
    /// 组织名称
    #[validate(length(min = 2, max = 100, message = "组织名称长度必须在2-100之间"))]
    pub name: String,
    /// 简称
    #[validate(length(max = 50, message = "简称长度不能超过50"))]
    pub short_name: Option<String>,
    /// 组织类型：1-集团 2-公司 3-分公司 4-子公司
    #[serde(rename = "type")]
    pub r#type: Option<i16>,
    /// 组织Logo
    pub logo: Option<String>,
    /// 描述
    #[validate(length(max = 500, message = "描述长度不能超过500"))]
    pub description: Option<String>,
    /// 排序
    pub sort_order: Option<i32>,
    /// 联系人姓名（顶级组织创建租户时使用）
    pub contact_name: Option<String>,
    /// 联系人手机号（顶级组织创建租户时使用）
    pub contact_mobile: Option<String>,
}

/// 更新组织请求
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdateOrganizationRequest {
    /// 组织名称
    #[validate(length(min = 2, max = 100, message = "组织名称长度必须在2-100之间"))]
    pub name: Option<String>,
    /// 简称
    #[validate(length(max = 50, message = "简称长度不能超过50"))]
    pub short_name: Option<String>,
    /// 组织类型
    #[serde(rename = "type")]
    pub r#type: Option<i16>,
    /// 组织Logo
    pub logo: Option<String>,
    /// 描述
    #[validate(length(max = 500, message = "描述长度不能超过500"))]
    pub description: Option<String>,
    /// 排序
    pub sort_order: Option<i32>,
    /// 状态
    pub status: Option<i16>,
}

/// 组织响应
#[derive(Debug, Serialize, ToSchema)]
pub struct OrganizationResponse {
    pub id: i64,
    pub tenant_id: i64,
    pub parent_id: Option<i64>,
    pub code: String,
    pub name: String,
    pub short_name: Option<String>,
    #[serde(rename = "type")]
    pub r#type: Option<i16>,
    pub logo: Option<String>,
    pub description: Option<String>,
    pub sort_order: Option<i32>,
    pub status: Option<i16>,
}

/// 组织列表查询参数
#[derive(Debug, Deserialize)]
pub struct ListOrganizationsQuery {
    #[serde(flatten)]
    pub page: CursorPageBaseReq,
    /// 搜索关键词
    pub keyword: Option<String>,
    /// 状态
    pub status: Option<i16>,
}

/// 组织树节点
#[derive(Debug, Serialize, ToSchema)]
pub struct OrganizationTreeNode {
    #[serde(flatten)]
    pub organization: OrganizationResponse,
    #[schema(no_recursion)]
    pub children: Vec<OrganizationTreeNode>,
}
