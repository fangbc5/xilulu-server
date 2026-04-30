// 设备模块

pub mod handler;
pub mod model;
mod repository;
mod service;

// 重新导出 service
pub use service::DeviceService;
// 重新导出 handler
pub use handler::*;
