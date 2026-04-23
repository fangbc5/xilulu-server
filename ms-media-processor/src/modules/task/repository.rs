use super::model::entity::MediaTask;
use crate::error::MediaError;
use sqlx::Row;
use sqlxplus::DbPool;
use std::sync::Arc;
use tracing::error;

pub struct TaskRepository {
    pool: Arc<DbPool>,
}

impl TaskRepository {
    pub fn new(pool: Arc<DbPool>) -> Self {
        Self { pool }
    }

    /// 乐观锁抢占任务
    pub async fn claim_task(
        &self,
        task_id: &str,
        current_version: i32,
    ) -> Result<bool, MediaError> {
        let rs = sqlx::query(
            r#"UPDATE media_tasks 
               SET status = 'PROCESSING', version = version + 1, updated_at = ? 
               WHERE id = ? AND status = 'INIT' AND version = ?"#,
        )
        .bind(chrono::Utc::now().timestamp_millis())
        .bind(task_id)
        .bind(current_version)
        .execute(self.pool.mysql_pool())
        .await
        .map_err(|e| MediaError::DatabaseFailed(e.to_string()))?;

        Ok(rs.rows_affected() > 0)
    }

    /// 获取任务详情
    pub async fn get_task(&self, task_id: &str) -> Result<Option<MediaTask>, MediaError> {
        let task = sqlx::query_as::<_, MediaTask>("SELECT * FROM media_tasks WHERE id = ?")
            .bind(task_id)
            .fetch_optional(self.pool.mysql_pool())
            .await
            .map_err(|e| MediaError::DatabaseFailed(e.to_string()))?;
        Ok(task)
    }

    /// 标记任务完成
    pub async fn mark_done(&self, task_id: &str, result_key: &str) -> Result<(), MediaError> {
        sqlx::query(
            r#"UPDATE media_tasks 
               SET status = 'DONE', result_key = ?, updated_at = ? 
               WHERE id = ?"#,
        )
        .bind(result_key)
        .bind(chrono::Utc::now().timestamp_millis())
        .bind(task_id)
        .execute(self.pool.mysql_pool())
        .await
        .map_err(|e| {
            error!("Fail to mark task {} done: {}", task_id, e);
            MediaError::DatabaseFailed(e.to_string())
        })?;
        Ok(())
    }

    /// 标记任务失败并处理重试
    pub async fn mark_failed_or_retry(
        &self,
        task_id: &str,
        err_msg: &str,
    ) -> Result<bool, MediaError> {
        // 先查出当前 retry count
        let row = sqlx::query("SELECT retry_count FROM media_tasks WHERE id = ?")
            .bind(task_id)
            .fetch_one(self.pool.mysql_pool())
            .await
            .map_err(|e| MediaError::DatabaseFailed(e.to_string()))?;

        let retry_count: i32 = row.get("retry_count");

        if retry_count < 3 {
            sqlx::query(
                r#"UPDATE media_tasks 
                   SET status = 'INIT', retry_count = retry_count + 1, error_message = ?, updated_at = ? 
                   WHERE id = ?"#
            )
            .bind(err_msg)
            .bind(chrono::Utc::now().timestamp_millis())
            .bind(task_id)
            .execute(self.pool.mysql_pool())
            .await
            .map_err(|e| MediaError::DatabaseFailed(e.to_string()))?;
            Ok(true) // 可重试
        } else {
            sqlx::query(
                r#"UPDATE media_tasks 
                   SET status = 'FAILED', error_message = ?, updated_at = ? 
                   WHERE id = ?"#,
            )
            .bind(err_msg)
            .bind(chrono::Utc::now().timestamp_millis())
            .bind(task_id)
            .execute(self.pool.mysql_pool())
            .await
            .map_err(|e| MediaError::DatabaseFailed(e.to_string()))?;
            Ok(false) // 发送 DLQ
        }
    }
}
