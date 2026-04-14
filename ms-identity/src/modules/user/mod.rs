// 用户模块

mod handler;
mod model;
mod repository;
mod service;

// 重新导出 model（供外部使用，包括 entity 和 dto）
pub use model::*;
// 重新导出 repository（供外部使用，service.rs 也通过这个重新导出使用）
pub use repository::{UserRepo, UserRoleRepo, UserTenantRelRepo};
// 重新导出 service
pub use service::{UserRoleService, UserService, UserTenantService};
// 重新导出 handler（去掉登录/刷新/验证码相关 HTTP 接口）
pub use handler::*;
