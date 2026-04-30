// 用户相关 DTO

use crate::modules::user::{TenantUserRel, User};
use serde::{Deserialize, Serialize};

/// 创建用户请求
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateUserRequest {
    pub username: String,
    pub password: String,
    pub email: Option<String>,
    pub mobile: Option<String>,
    pub nick_name: Option<String>,
}

/// 创建用户响应
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct CreateUserResponse {
    pub user_id: i64,
}

/// 更新用户请求
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateUserRequest {
    pub username: Option<String>,
    pub email: Option<String>,
    pub mobile: Option<String>,
    pub nick_name: Option<String>,
}

/// 获取活跃用户数请求
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct GetActiveUserCountRequest {
    /// 统计天数（默认30天）
    pub days: Option<u32>,
}

/// 用户列表请求
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct ListUsersRequest {
    #[serde(flatten)]
    pub page: fbc_starter::base::CursorPageBaseReq,
    /// 搜索关键词（用户名、邮箱、手机号、昵称）
    pub search_key: Option<String>,
    /// 租户 ID（可选，用于过滤）
    pub tenant_id: Option<i64>,
}

/// 修改密码请求
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct ChangePasswordRequest {
    pub old_password: String,
    pub new_password: String,
}

/// 重置密码请求
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct ResetPasswordRequest {
    pub new_password: String,
}

/// 用户信息响应
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct UserInfo {
    pub id: Option<i64>,
    pub username: Option<String>,
    pub email: Option<String>,
    pub mobile: Option<String>,
    pub nick_name: Option<String>,
    pub state: Option<i16>,
}

impl From<User> for UserInfo {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
            mobile: user.mobile,
            nick_name: user.nick_name,
            state: user.state,
        }
    }
}

/// 用户租户关系响应
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct UserTenantInfo {
    pub id: Option<i64>,
    pub user_id: i64,
    pub tenant_id: i64,
    pub is_owner: Option<i16>,
    pub status: Option<i16>,
    pub join_time: Option<chrono::DateTime<chrono::Utc>>,
}

impl From<TenantUserRel> for UserTenantInfo {
    fn from(rel: TenantUserRel) -> Self {
        Self {
            id: rel.id,
            user_id: rel.user_id,
            tenant_id: rel.tenant_id,
            is_owner: rel.is_owner,
            status: rel.status,
            join_time: rel.join_time,
        }
    }
}

/// 添加用户到租户请求
/// 注意：user_id 从路径参数中获取，请求体只需要 tenant_id
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct AddUserToTenantRequest {
    pub tenant_id: i64,
}

/// 设置默认租户请求
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct SetDefaultTenantRequest {
    pub tenant_id: i64,
}



/// 用户角色信息响应
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct UserRoleInfo {
    pub id: Option<i64>,
    pub user_id: i64,
    pub role_id: i64,
    pub role_code: String,
    pub tenant_id: i64,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl From<crate::modules::user::UserRole> for UserRoleInfo {
    fn from(role: crate::modules::user::UserRole) -> Self {
        Self {
            id: role.id,
            user_id: role.user_id,
            role_id: role.role_id,
            role_code: role.role_code,
            tenant_id: role.tenant_id,
            created_at: role.created_at,
        }
    }
}

/// 分配角色给用户请求
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct AssignRoleToUserRequest {
    pub role_id: i64,
}

/// 批量分配角色给用户请求
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct BatchAssignRolesToUserRequest {
    pub role_ids: Vec<i64>,
}

/// 移除用户角色请求（role_id 从路径参数获取，此结构体保留用于未来扩展）
#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[allow(dead_code)]
pub struct RemoveRoleFromUserRequest {
    pub role_id: i64,
}

