// 租户模块的数据模型
// 包含 entity（数据库实体）、dto（数据传输对象）等

mod dto;
mod entity;
mod enums;

// 重新导出 entity
pub use entity::{Tenant, TenantApplicationRel};
// 重新导出 dto
pub use dto::*;
// 重新导出 enums
pub use enums::SystemTenant;
