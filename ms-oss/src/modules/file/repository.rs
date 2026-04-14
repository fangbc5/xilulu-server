use sqlx;
use sqlxplus::DbPool;
use std::sync::Arc;

use super::model::entity::FileMeta;

/// 文件元数据 Repository
pub struct FileMetaRepo;

impl FileMetaRepo {
    /// 插入文件元数据记录（返回自增 ID）
    pub async fn insert(db: &Arc<DbPool>, meta: &FileMeta) -> anyhow::Result<i64> {
        let result: sqlx::mysql::MySqlQueryResult = sqlx::query(
            r#"INSERT INTO file_meta (file_key, bucket, original_name, content_type, size, scene, uploader_id, provider, status)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
        )
        .bind(&meta.file_key)
        .bind(&meta.bucket)
        .bind(&meta.original_name)
        .bind(&meta.content_type)
        .bind(&meta.size)
        .bind(&meta.scene)
        .bind(&meta.uploader_id)
        .bind(&meta.provider)
        .bind(&meta.status)
        .execute(db.mysql_pool())
        .await?;

        Ok(result.last_insert_id() as i64)
    }

    /// 根据 ID 查询
    pub async fn find_by_id(db: &Arc<DbPool>, id: i64) -> anyhow::Result<Option<FileMeta>> {
        let meta = sqlx::query_as::<_, FileMeta>("SELECT * FROM file_meta WHERE id = ?")
            .bind(id)
            .fetch_optional(db.mysql_pool())
            .await?;
        Ok(meta)
    }

    /// 更新状态
    pub async fn update_status(db: &Arc<DbPool>, id: i64, status: i8) -> anyhow::Result<()> {
        sqlx::query("UPDATE file_meta SET status = ? WHERE id = ?")
            .bind(status)
            .bind(id)
            .execute(db.mysql_pool())
            .await?;
        Ok(())
    }

    /// 更新文件大小（上传回调时）
    pub async fn update_size(db: &Arc<DbPool>, id: i64, size: i64) -> anyhow::Result<()> {
        sqlx::query("UPDATE file_meta SET size = ? WHERE id = ?")
            .bind(size)
            .bind(id)
            .execute(db.mysql_pool())
            .await?;
        Ok(())
    }

    /// 确认上传完成：更新状态和实际文件大小
    pub async fn update_confirm(db: &Arc<DbPool>, id: i64, status: i8, size: i64) -> anyhow::Result<()> {
        sqlx::query("UPDATE file_meta SET status = ?, size = ? WHERE id = ?")
            .bind(status)
            .bind(size)
            .bind(id)
            .execute(db.mysql_pool())
            .await?;
        Ok(())
    }

    /// 根据 file_key 查询
    pub async fn find_by_key(db: &Arc<DbPool>, file_key: &str) -> anyhow::Result<Option<FileMeta>> {
        let meta = sqlx::query_as::<_, FileMeta>("SELECT * FROM file_meta WHERE file_key = ?")
            .bind(file_key)
            .fetch_optional(db.mysql_pool())
            .await?;
        Ok(meta)
    }

    /// 标记删除
    pub async fn soft_delete(db: &Arc<DbPool>, id: i64) -> anyhow::Result<()> {
        sqlx::query("UPDATE file_meta SET status = 2 WHERE id = ?")
            .bind(id)
            .execute(db.mysql_pool())
            .await?;
        Ok(())
    }
}
