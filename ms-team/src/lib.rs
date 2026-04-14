// ms-team 库文件
// 暴露公开的 API 供集成测试和其他模块使用

pub mod config;
pub mod error;
pub mod middleware;
pub mod modules;
pub mod router;
pub mod state;

pub use error::{OrganizationError, Result};
