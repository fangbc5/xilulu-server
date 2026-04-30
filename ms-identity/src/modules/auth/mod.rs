// 权限模块

pub mod handler;
pub mod model;
mod repository;
mod service;

// 重新导出 model（包括 entity 和 dto）
pub use model::*;
// 重新导出 repository
pub use repository::{
    ResourceRepo, RoleRepo, RoleResourceRelRepo,
};
// 重新导出 service
pub use service::{ApplicationService, PermissionService, ResourceService, RoleService};
// 重新导出 handler
pub use handler::*;
