// 设备模块 Repository
// 自定义查询（CRUD trait 以外的方法）

use crate::modules::device::model::entity::UserDevice;
use anyhow::Result;
use sqlx::{MySql, Pool};
use sqlxplus::{Crud, QueryBuilder};

/// 设备 Repository
pub struct DeviceRepo;

impl DeviceRepo {
    /// 根据 uid + client_id 查找设备
    pub async fn find_by_uid_and_client_id(
        pool: &Pool<MySql>,
        uid: i64,
        client_id: &str,
    ) -> Result<Option<UserDevice>> {
        let builder = QueryBuilder::new("SELECT * FROM `user_device`")
            .and_eq("uid", uid)
            .and_eq("client_id", client_id);
        let device = UserDevice::find_one(pool, builder).await?;
        Ok(device)
    }

    /// 查找用户所有有效设备
    pub async fn find_active_by_uid(pool: &Pool<MySql>, uid: i64) -> Result<Vec<UserDevice>> {
        let builder = QueryBuilder::new("SELECT * FROM `user_device`")
            .and_eq("uid", uid)
            .and_eq("is_active", 1);
        let devices = UserDevice::find_all(pool, Some(builder)).await?;
        Ok(devices)
    }

    /// 批量查找多个用户的有效设备
    pub async fn find_active_by_uids(pool: &Pool<MySql>, uids: &[i64]) -> Result<Vec<UserDevice>> {
        if uids.is_empty() {
            return Ok(vec![]);
        }
        let builder = QueryBuilder::new("SELECT * FROM `user_device`")
            .and_in("uid", uids.to_vec())
            .and_eq("is_active", 1);
        let devices = UserDevice::find_all(pool, Some(builder)).await?;
        Ok(devices)
    }

    /// 将指定设备标记为无效（用于 Token 失效场景）
    pub async fn deactivate_by_token(pool: &Pool<MySql>, device_token: &str) -> Result<()> {
        sqlx::query("UPDATE `user_device` SET `is_active` = 0, `updated_at` = NOW(3) WHERE `device_token` = ?")
            .bind(device_token)
            .execute(pool)
            .await?;
        Ok(())
    }
}
