// 设备模块

mod handler;
mod model;
mod repository;
mod service;

// 重新导出 service
pub use service::DeviceService;
// 重新导出 handler
pub use handler::*;
