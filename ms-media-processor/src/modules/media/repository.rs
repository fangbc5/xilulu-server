//! 媒体任务数据层
//!
//! 只实现 sqlxplus CRUD 不存在的方法（乐观锁抢占、状态更新等）

use crate::error::MediaError;
use sqlx::Row;

/// 媒体任务仓库
pub struct MediaTaskRepo;

impl MediaTaskRepo {
    /// 乐观锁抢占任务：INIT → PROCESSING
    pub async fn claim_task(
        pool: &sqlx::MySqlPool,
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
        .execute(pool)
        .await
        .map_err(|e| MediaError::DatabaseFailed(e.to_string()))?;

        Ok(rs.rows_affected() > 0)
    }

    /// 标记任务完成
    pub async fn mark_done(
        pool: &sqlx::MySqlPool,
        task_id: &str,
        result_key: &str,
        result_meta: Option<&str>,
    ) -> Result<(), MediaError> {
        sqlx::query(
            r#"UPDATE media_tasks 
               SET status = 'DONE', result_key = ?, result_meta = ?, updated_at = ? 
               WHERE id = ?"#,
        )
        .bind(result_key)
        .bind(result_meta)
        .bind(chrono::Utc::now().timestamp_millis())
        .bind(task_id)
        .execute(pool)
        .await
        .map_err(|e| MediaError::DatabaseFailed(e.to_string()))?;

        Ok(())
    }

    /// 标记失败或重试（单条 SQL，避免 TOCTOU 竞态）
    ///
    /// 返回 true = 已重置为 INIT（可重试），false = 已标记 FAILED（发 DLQ）
    pub async fn mark_failed_or_retry(
        pool: &sqlx::MySqlPool,
        task_id: &str,
        err_msg: &str,
    ) -> Result<bool, MediaError> {
        // 单条 SQL 原子更新，避免先 SELECT 再 UPDATE 的竞态
        let rs = sqlx::query(
            r#"UPDATE media_tasks 
               SET status = CASE WHEN retry_count < max_retry THEN 'INIT' ELSE 'FAILED' END,
                   retry_count = retry_count + 1,
                   error_message = ?,
                   updated_at = ?
               WHERE id = ? AND status = 'PROCESSING'"#,
        )
        .bind(err_msg)
        .bind(chrono::Utc::now().timestamp_millis())
        .bind(task_id)
        .execute(pool)
        .await
        .map_err(|e| MediaError::DatabaseFailed(e.to_string()))?;

        if rs.rows_affected() == 0 {
            return Ok(false);
        }

        // 查一下最终状态
        let row = sqlx::query("SELECT status FROM media_tasks WHERE id = ?")
            .bind(task_id)
            .fetch_one(pool)
            .await
            .map_err(|e| MediaError::DatabaseFailed(e.to_string()))?;

        let status: String = row.get("status");
        Ok(status == "INIT") // true=可重试, false=已FAILED
    }
}
