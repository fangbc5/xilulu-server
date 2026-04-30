// 权限相关 DTO

use crate::modules::auth::{Application, Resource, Role};
use fbc_starter::base::CursorPageBaseReq;
use serde::{Deserialize, Serialize};

/// 角色列表查询请求（扩展游标分页请求）
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct ListRolesRequest {
    #[serde(flatten)]
    pub page: CursorPageBaseReq,
    /// 租户 ID（可选，用于过滤）
    pub tenant_id: Option<i64>,
}

/// 资源列表查询请求（扩展游标分页请求）
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct ListResourcesRequest {
    #[serde(flatten)]
    pub page: CursorPageBaseReq,
    /// 应用 ID（可选，用于过滤）
    pub application_id: Option<i64>,
    /// 租户 ID（可选，用于过滤）
    pub tenant_id: Option<i64>,
    /// 搜索关键词（资源代码、资源名称）
    pub search_key: Option<String>,
}

/// 获取用户菜单请求
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct GetUserMenusRequest {
    /// 应用 ID（必填）
    pub application_id: i64,
}

/// 获取菜单下子资源请求
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct GetMenuResourcesRequest {
    /// 应用 ID（必填）
    pub application_id: i64,
    /// 菜单资源 ID（必填，对应 resource.id，resource_type = '20'）
    pub menu_id: i64,
}

/// 应用列表查询请求（扩展游标分页请求）
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct ListApplicationsRequest {
    #[serde(flatten)]
    pub page: CursorPageBaseReq,
    /// 搜索关键词（应用标识、应用名称）
    pub search_key: Option<String>,
}

/// 创建角色请求
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateRoleRequest {
    pub code: String,
    pub name: String,
    pub tenant_id: i64,
    pub remarks: Option<String>,
    pub state: Option<bool>,
}

/// 创建角色响应
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct CreateRoleResponse {
    pub role_id: i64,
}

/// 更新角色请求
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateRoleRequest {
    pub name: Option<String>,
    pub remarks: Option<String>,
    pub state: Option<bool>,
}

/// 角色信息响应
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct RoleInfo {
    pub id: Option<i64>,
    pub code: String,
    pub name: String,
    pub tenant_id: i64,
    pub remarks: Option<String>,
    pub state: Option<bool>,
}

impl From<Role> for RoleInfo {
    fn from(role: Role) -> Self {
        Self {
            id: role.id,
            code: role.code,
            name: role.name,
            tenant_id: role.tenant_id,
            remarks: role.remarks,
            state: role.state,
        }
    }
}

/// 创建资源请求
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateResourceRequest {
    pub application_id: i64,
    pub code: String,
    pub name: String,
    pub parent_id: i64,
    pub resource_type: Option<String>,
    pub path: Option<String>,
    pub describe_: Option<String>,
}

/// 创建资源响应
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct CreateResourceResponse {
    pub resource_id: i64,
}

/// 更新资源请求
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateResourceRequest {
    pub name: Option<String>,
    pub path: Option<String>,
    pub describe_: Option<String>,
}

/// 资源信息响应
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ResourceInfo {
    pub id: Option<i64>,
    pub application_id: i64,
    pub code: String,
    pub name: String,
    pub parent_id: i64,
    pub resource_type: Option<String>,
    pub path: Option<String>,
    pub component: Option<String>,
    pub icon: Option<String>,
    pub describe_: Option<String>,
    pub state: Option<bool>,
}

/// 菜单下子资源按类型分类的响应
#[derive(Debug, Serialize, Default, utoipa::ToSchema)]
pub struct MenuResourcesByType {
    /// 菜单类型资源（一般为空，预留）
    pub menus: Vec<ResourceInfo>,
    /// 按钮类型资源（resource_type = '40'）
    pub buttons: Vec<ResourceInfo>,
    /// 字段类型资源（resource_type = '50'）
    pub fields: Vec<ResourceInfo>,
    /// 数据权限等资源（resource_type = '60'）
    pub data: Vec<ResourceInfo>,
}

impl From<Resource> for ResourceInfo {
    fn from(resource: Resource) -> Self {
        Self {
            id: resource.id,
            application_id: resource.application_id,
            code: resource.code,
            name: resource.name,
            parent_id: resource.parent_id,
            resource_type: resource.resource_type,
            path: resource.path,
            component: resource.component,
            icon: resource.icon,
            describe_: resource.describe_,
            state: resource.state,
        }
    }
}

/// 分配资源到角色请求
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct AssignResourceToRoleRequest {
    pub resource_id: i64,
}

/// 创建应用请求
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateApplicationRequest {
    pub app_key: String, // 应用标识（必填）
    pub name: String,
    #[serde(rename = "type")]
    pub r#type: Option<String>, // 应用类型：10-自建应用 20-第三方应用
    pub app_secret: Option<String>, // 应用秘钥
    pub version: Option<String>,    // 版本
    pub redirect: Option<String>,   // 重定向地址
    pub url: Option<String>,        // 应用地址
    pub introduce: Option<String>,  // 简介
    pub remark: Option<String>,     // 备注
    pub is_general: Option<bool>,   // 是否公共应用
    pub is_visible: Option<bool>,   // 是否可见
    pub sort_value: Option<i32>,    // 排序
}

/// 创建应用响应
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct CreateApplicationResponse {
    pub application_id: i64,
}

/// 更新应用请求
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateApplicationRequest {
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub r#type: Option<String>, // 应用类型
    pub version: Option<String>,   // 版本
    pub redirect: Option<String>,  // 重定向地址
    pub url: Option<String>,       // 应用地址
    pub introduce: Option<String>, // 简介
    pub remark: Option<String>,    // 备注
    pub is_general: Option<bool>,  // 是否公共应用
    pub is_visible: Option<bool>,  // 是否可见
    pub sort_value: Option<i32>,   // 排序
}

/// 应用信息响应
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ApplicationInfo {
    pub id: Option<i64>,
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub r#type: Option<String>, // 应用类型
    pub app_key: Option<String>,                            // 应用标识
    pub version: Option<String>,                            // 版本
    pub redirect: Option<String>,                           // 重定向地址
    pub url: Option<String>,                                // 应用地址
    pub introduce: Option<String>,                          // 简介
    pub remark: Option<String>,                             // 备注
    pub is_general: Option<bool>,                           // 是否公共应用
    pub is_visible: Option<bool>,                           // 是否可见
    pub sort_value: Option<i32>,                            // 排序
    pub create_time: Option<chrono::DateTime<chrono::Utc>>, // 创建时间
    pub update_time: Option<chrono::DateTime<chrono::Utc>>, // 更新时间
}

impl From<Application> for ApplicationInfo {
    fn from(app: Application) -> Self {
        Self {
            id: app.id,
            name: app.name,
            r#type: app.r#type,
            app_key: app.app_key,
            version: app.version,
            redirect: app.redirect,
            url: app.url,
            introduce: app.introduce,
            remark: app.remark,
            is_general: app.is_general,
            is_visible: app.is_visible,
            sort_value: app.sort_value,
            create_time: app.create_time,
            update_time: app.update_time,
        }
    }
}

/// 检查权限请求（待重新设计）
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CheckPermissionRequest {}

/// 检查权限响应
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct CheckPermissionResponse {
    pub allowed: bool,
}


