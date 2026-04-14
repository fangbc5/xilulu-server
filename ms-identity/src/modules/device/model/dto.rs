// 设备模块 DTO

use serde::{Deserialize, Serialize};

/// 注册/更新设备推送 Token 请求
#[derive(Debug, Deserialize)]
pub struct RegisterDeviceRequest {
    /// 设备指纹（与 WS 的 clientId 一致）
    pub client_id: String,
    /// APNs/FCM 推送 Token
    pub device_token: String,
    /// 平台类型：ios / android
    pub platform: String,
    /// 客户端版本号（可选）
    pub app_version: Option<String>,
}

/// 注销设备请求
#[derive(Debug, Deserialize)]
pub struct UnregisterDeviceRequest {
    /// 设备指纹
    pub client_id: String,
}

/// 设备信息响应
#[derive(Debug, Serialize)]
pub struct DeviceInfo {
    pub id: i64,
    pub uid: i64,
    pub client_id: String,
    pub platform: String,
    pub app_version: Option<String>,
    pub is_active: i16,
}

impl From<super::entity::UserDevice> for DeviceInfo {
    fn from(d: super::entity::UserDevice) -> Self {
        Self {
            id: d.id.unwrap_or(0),
            uid: d.uid.unwrap_or(0),
            client_id: d.client_id.unwrap_or_default(),
            platform: d.platform.unwrap_or_default(),
            app_version: d.app_version,
            is_active: d.is_active.unwrap_or(0),
        }
    }
}
