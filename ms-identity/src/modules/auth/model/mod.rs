// 权限模块的数据模型
// 包含 entity（数据库实体）、dto（数据传输对象）等

pub mod dto;
mod entity;

// 重新导出 entity
pub use entity::*;
// 重新导出 dto
pub use dto::*;
