use sqlxplus::{Crud, DbPool};
use std::sync::Arc;

use super::model::entity::FileMeta;

/// 文件元数据 Repository
pub struct FileMetaRepo;

impl FileMetaRepo {
    /// 幂等写入：若 (bucket, file_key) 已存在则返回已有 ID
    pub async fn insert(db: &Arc<DbPool>, meta: &FileMeta) -> anyhow::Result<i64> {
        if let (Some(bucket), Some(key)) = (&meta.bucket, &meta.file_key) {
            if let Ok(Some(existing)) = Self::find_by_bucket_key(db, bucket, key).await {
                if let Some(id) = existing.id {
                    return Ok(id);
                }
            }
        }
        let result = meta.insert(db.mysql_pool()).await?;
        Ok(result)
    }

    pub async fn find_by_id(db: &Arc<DbPool>, id: i64) -> anyhow::Result<Option<FileMeta>> {
        let meta = FileMeta::find_by_id(db.mysql_pool(), id).await?;
        Ok(meta)
    }

    pub async fn update_status(db: &Arc<DbPool>, id: i64, status: i16) -> anyhow::Result<()> {
        let meta = FileMeta {
            id: Some(id),
            status: Some(status),
            ..Default::default()
        };
        meta.update(db.mysql_pool()).await?;
        Ok(())
    }

    /// 更新文件大小（上传回调时）
    pub async fn update_size(db: &Arc<DbPool>, id: i64, size: i64) -> anyhow::Result<()> {
        let meta = FileMeta {
            id: Some(id),
            size: Some(size),
            ..Default::default()
        };
        meta.update(db.mysql_pool()).await?;
        Ok(())
    }

    /// 确认上传完成：更新状态和实际文件大小
    pub async fn update_confirm(db: &Arc<DbPool>, id: i64, status: i16, size: i64) -> anyhow::Result<()> {
        let meta = FileMeta {
            id: Some(id),
            status: Some(status),
            size: Some(size),
            ..Default::default()
        };
        meta.update(db.mysql_pool()).await?;
        Ok(())
    }

    pub async fn find_by_key(db: &Arc<DbPool>, file_key: &str) -> anyhow::Result<Option<FileMeta>> {
        let builder = sqlxplus::QueryBuilder::new("SELECT * FROM file_meta")
            .and_eq("file_key", file_key.to_string());
        let list = FileMeta::find_all(db.mysql_pool(), Some(builder)).await?;
        Ok(list.into_iter().next())
    }

    /// 按 bucket + file_key 查找元数据
    pub async fn find_by_bucket_key(db: &Arc<DbPool>, bucket: &str, key: &str) -> anyhow::Result<Option<FileMeta>> {
        let builder = sqlxplus::QueryBuilder::new("SELECT * FROM file_meta")
            .and_eq("bucket", bucket.to_string())
            .and_eq("file_key", key.to_string());
        let list = FileMeta::find_all(db.mysql_pool(), Some(builder)).await?;
        Ok(list.into_iter().next())
    }

    /// 标记删除
    pub async fn soft_delete(db: &Arc<DbPool>, id: i64) -> anyhow::Result<()> {
        let meta = FileMeta {
            id: Some(id),
            status: Some(2),
            ..Default::default()
        };
        meta.update(db.mysql_pool()).await?;
        Ok(())
    }

    /// 回写视频缩略图 Key（由 ms-media-processor 完成事件触发）
    pub async fn update_thumbnail_key(
        db: &Arc<DbPool>,
        bucket: &str,
        key: &str,
        thumbnail_key: &str,
    ) -> anyhow::Result<()> {
        if let Some(meta) = Self::find_by_bucket_key(db, bucket, key).await? {
            if let Some(id) = meta.id {
                let update = FileMeta {
                    id: Some(id),
                    thumbnail_key: Some(thumbnail_key.to_string()),
                    ..Default::default()
                };
                update.update(db.mysql_pool()).await?;
            }
        }
        Ok(())
    }
}
