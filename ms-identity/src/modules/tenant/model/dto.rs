// 租户相关 DTO

use crate::modules::tenant::Tenant;
use serde::{Deserialize, Serialize};

/// 创建租户请求
#[derive(Debug, Deserialize)]
pub struct CreateTenantRequest {
    pub name: String,
    pub contact_name: String,
    pub contact_mobile: Option<String>,
    pub package_id: i64,
    pub expire_time: chrono::DateTime<chrono::Utc>,
    pub account_count: i32,
    pub website: Option<String>,
}

/// 创建租户响应
#[derive(Debug, Serialize)]
pub struct CreateTenantResponse {
    pub tenant_id: i64,
}

/// 更新租户请求
#[derive(Debug, Deserialize)]
pub struct UpdateTenantRequest {
    pub name: Option<String>,
    pub contact_name: Option<String>,
    pub contact_mobile: Option<String>,
    pub status: Option<i16>,
    pub website: Option<String>,
}

/// 租户信息响应
#[derive(Debug, Serialize)]
pub struct TenantInfo {
    pub id: Option<i64>,
    pub name: String,
    pub contact_name: String,
    pub contact_mobile: Option<String>,
    pub status: Option<i16>,
    pub website: Option<String>,
    pub package_id: i64,
    pub account_count: i32,
    pub expire_time: chrono::DateTime<chrono::Utc>,
}

impl From<Tenant> for TenantInfo {
    fn from(tenant: Tenant) -> Self {
        Self {
            id: tenant.id,
            name: tenant.name,
            contact_name: tenant.contact_name,
            contact_mobile: tenant.contact_mobile,
            status: tenant.status,
            website: tenant.website,
            package_id: tenant.package_id,
            account_count: tenant.account_count,
            expire_time: tenant.expire_time,
        }
    }
}



/// 添加应用到租户请求
#[derive(Debug, Deserialize)]
pub struct AddApplicationToTenantRequest {
    pub application_id: i64,
}

/// 租户列表请求
#[derive(Debug, Deserialize)]
pub struct ListTenantsRequest {
    #[serde(flatten)]
    pub page: fbc_starter::base::CursorPageBaseReq,
    /// 搜索关键词（租户名称、联系人、联系电话）
    pub search_key: Option<String>,
}
