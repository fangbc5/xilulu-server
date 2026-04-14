// 租户模块的 Entity 定义

mod tenant;
mod tenant_application_rel;

// 重新导出
pub use tenant::Tenant;
pub use tenant_application_rel::TenantApplicationRel;
