// 设备模块 Service
// 负责推送设备注册/注销的业务逻辑

use crate::error::IdentityError;
use crate::modules::device::model::dto::RegisterDeviceRequest;
use crate::modules::device::model::entity::UserDevice;
use crate::modules::device::repository::DeviceRepo;
use anyhow::Result;
use chrono::Utc;
use sqlxplus::{Crud, DbPool};
use std::sync::Arc;
use tracing::info;

/// 设备 Service
pub struct DeviceService {
    db_pool: Arc<DbPool>,
}

impl DeviceService {
    /// 创建新的 DeviceService
    pub fn new(db_pool: Arc<DbPool>) -> Self {
        Self { db_pool }
    }

    /// 注册/更新设备推送 Token（UPSERT 语义）
    pub async fn register_device(&self, uid: i64, req: &RegisterDeviceRequest) -> Result<i64> {
        // 验证平台类型
        if req.platform != "ios" && req.platform != "android" {
            return Err(IdentityError::BusinessError(
                "平台类型仅支持 ios 或 android".to_string(),
            )
            .into());
        }

        // 查找是否已有同一 uid + client_id 的记录
        let existing = DeviceRepo::find_by_uid_and_client_id(
            self.db_pool.mysql_pool(),
            uid,
            &req.client_id,
        )
        .await?;

        if let Some(device) = existing {
            // 更新现有记录
            let device_id = device.id.unwrap_or(0);
            let updated = UserDevice {
                id: device.id,
                device_token: Some(req.device_token.clone()),
                platform: Some(req.platform.clone()),
                app_version: req.app_version.clone(),
                is_active: Some(1),
                updated_at: Some(Utc::now()),
                ..Default::default()
            };
            updated.update(self.db_pool.mysql_pool()).await
                .map_err(|e| IdentityError::DatabaseError(e.to_string()))?;
            info!("设备 Token 已更新: uid={}, client_id={}", uid, req.client_id);
            Ok(device_id)
        } else {
            // 插入新记录
            let device = UserDevice {
                id: None,
                uid: Some(uid),
                client_id: Some(req.client_id.clone()),
                device_token: Some(req.device_token.clone()),
                platform: Some(req.platform.clone()),
                app_version: req.app_version.clone(),
                is_active: Some(1),
                created_at: Some(Utc::now()),
                updated_at: Some(Utc::now()),
            };
            let device_id = device.insert(self.db_pool.mysql_pool()).await
                .map_err(|e| IdentityError::DatabaseError(e.to_string()))?;
            info!("设备 Token 已注册: uid={}, client_id={}, id={}", uid, req.client_id, device_id);
            Ok(device_id)
        }
    }

    /// 注销设备（标记为无效）
    pub async fn unregister_device(&self, uid: i64, client_id: &str) -> Result<()> {
        let existing = DeviceRepo::find_by_uid_and_client_id(
            self.db_pool.mysql_pool(),
            uid,
            client_id,
        )
        .await?;

        if let Some(device) = existing {
            let updated = UserDevice {
                id: device.id,
                is_active: Some(0),
                updated_at: Some(Utc::now()),
                ..Default::default()
            };
            updated.update(self.db_pool.mysql_pool()).await
                .map_err(|e| IdentityError::DatabaseError(e.to_string()))?;
            info!("设备已注销: uid={}, client_id={}", uid, client_id);
        }

        Ok(())
    }

    /// 获取用户所有有效设备
    pub async fn get_active_devices(&self, uid: i64) -> Result<Vec<UserDevice>> {
        DeviceRepo::find_active_by_uid(self.db_pool.mysql_pool(), uid).await
    }

    /// 批量获取多个用户的有效设备
    pub async fn get_active_devices_by_uids(&self, uids: &[i64]) -> Result<Vec<UserDevice>> {
        DeviceRepo::find_active_by_uids(self.db_pool.mysql_pool(), uids).await
    }

    /// 标记 Token 为失效（推送返回无效 Token 时调用）
    pub async fn deactivate_token(&self, device_token: &str) -> Result<()> {
        DeviceRepo::deactivate_by_token(self.db_pool.mysql_pool(), device_token).await?;
        info!("设备 Token 已标记失效: token={}...", &device_token[..device_token.len().min(20)]);
        Ok(())
    }
}
