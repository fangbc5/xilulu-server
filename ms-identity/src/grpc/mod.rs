// gRPC 服务模块

pub mod identity_service;
pub mod device_service;

pub use identity_service::IdentityServiceImpl;
pub use device_service::DeviceServiceImpl;
