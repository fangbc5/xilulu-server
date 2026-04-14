use anyhow::Result;
use sqlxplus::Crud;

use super::entity::NotifyLog;

/// 通知日志 Repository
pub struct NotifyLogRepo;

impl NotifyLogRepo {
    /// 插入日志记录（返回自增 ID）
    pub async fn insert(pool: &sqlx::Pool<sqlx::MySql>, log: &mut NotifyLog) -> Result<i64> {
        let id = log.insert(pool).await?;
        Ok(id)
    }

    /// 更新状态为成功
    pub async fn mark_success(pool: &sqlx::Pool<sqlx::MySql>, id: i64) -> Result<()> {
        let log = NotifyLog {
            id: Some(id),
            status: Some(2),
            ..Default::default()
        };
        log.update(pool).await?;
        Ok(())
    }

    /// 更新状态为失败
    pub async fn mark_failed(pool: &sqlx::Pool<sqlx::MySql>, id: i64, error_msg: &str) -> Result<()> {
        let log = NotifyLog {
            id: Some(id),
            status: Some(3),
            error_msg: Some(error_msg.to_string()),
            ..Default::default()
        };
        log.update(pool).await?;
        Ok(())
    }
}
