use fbc_starter::base::CursorPageBaseReq;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

/// 创建岗位请求
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreatePositionRequest {
    /// 所属组织ID
    pub org_id: i64,
    /// 岗位编码
    #[validate(length(min = 1, max = 50, message = "岗位编码长度必须在1-50之间"))]
    pub code: String,
    /// 岗位名称
    #[validate(length(min = 1, max = 100, message = "岗位名称长度必须在1-100之间"))]
    pub name: String,
    /// 岗位类别
    pub category: Option<String>,
    /// 岗位级别
    pub level: Option<i32>,
    /// 岗位职责描述
    pub description: Option<String>,
    /// 任职要求
    pub requirements: Option<String>,
    /// 排序
    pub sort_order: Option<i32>,
}

/// 更新岗位请求
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdatePositionRequest {
    /// 岗位名称
    #[validate(length(min = 1, max = 100, message = "岗位名称长度必须在1-100之间"))]
    pub name: Option<String>,
    /// 岗位类别
    pub category: Option<String>,
    /// 岗位级别
    pub level: Option<i32>,
    /// 岗位职责描述
    pub description: Option<String>,
    /// 任职要求
    pub requirements: Option<String>,
    /// 排序
    pub sort_order: Option<i32>,
    /// 状态
    pub status: Option<i16>,
}

/// 岗位响应
#[derive(Debug, Serialize, ToSchema)]
pub struct PositionResponse {
    pub id: i64,
    pub tenant_id: i64,
    pub org_id: i64,
    pub code: String,
    pub name: String,
    pub category: Option<String>,
    pub level: Option<i32>,
    pub description: Option<String>,
    pub requirements: Option<String>,
    pub sort_order: Option<i32>,
    pub status: Option<i16>,
}

/// 岗位列表查询参数
#[derive(Debug, Deserialize)]
pub struct ListPositionsQuery {
    #[serde(flatten)]
    pub page: CursorPageBaseReq,
    /// 组织ID
    pub org_id: Option<i64>,
    /// 岗位类别
    pub category: Option<String>,
    /// 搜索关键词
    pub keyword: Option<String>,
    /// 状态
    pub status: Option<i16>,
}
