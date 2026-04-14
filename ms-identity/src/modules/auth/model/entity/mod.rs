// 权限模块的 Entity 定义

mod application;
mod resource;
mod role;
mod role_resource_rel;

// 重新导出
pub use application::Application;
pub use resource::Resource;
pub use role::Role;
pub use role_resource_rel::RoleResourceRel;
