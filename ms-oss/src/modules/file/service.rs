use sqlxplus::DbPool;
use std::sync::Arc;
use uuid::Uuid;

use super::model::dto::*;
use super::model::entity::FileMeta;
use super::repository::FileMetaRepo;
use crate::config::OssConfig;
use crate::provider::OssProvider;

/// 文件服务
pub struct FileService {
    db: Arc<DbPool>,
    config: OssConfig,
    provider: Arc<dyn OssProvider>,
}

impl FileService {
    pub fn new(db: Arc<DbPool>, config: OssConfig, provider: Arc<dyn OssProvider>) -> Self {
        Self {
            db,
            config,
            provider,
        }
    }

    /// 将 FileMeta 转为 FileMetaResponse
    fn to_response(
        m: &FileMeta,
        overrides: Option<(i16, Option<i64>)>,
    ) -> FileMetaResponse {
        let (status, size) = overrides.unwrap_or_else(|| (m.status.unwrap_or(0), m.size));
        FileMetaResponse {
            id: m.id.unwrap_or(0),
            file_key: m.file_key.clone().unwrap_or_default(),
            bucket: m.bucket.clone().unwrap_or_default(),
            original_name: m.original_name.clone(),
            content_type: m.content_type.clone(),
            size,
            scene: m.scene.clone().unwrap_or_default(),
            status,
            created_at: m
                .created_at
                .unwrap_or(0),
        }
    }

    /// 生成预签名上传 URL
    ///
    /// 1. 按 scene 规则校验文件格式和大小
    /// 2. 生成 object key：{scene}/{year}/{month}/{uuid}.{ext}
    /// 3. 在数据库创建 file_meta 记录
    /// 4. 调用 Provider 生成预签名 URL
    pub async fn presign_upload(
        &self,
        req: PresignUploadRequest,
        uploader_id: Option<i64>,
    ) -> anyhow::Result<PresignUploadResponse> {
        // 查找 SceneRule
        let rule = self.config.get_scene_rule(&req.scene);

        // 校验文件扩展名
        let ext = req.filename.rsplit('.').next().unwrap_or("bin");
        if !rule.allowed_extensions.is_empty()
            && !rule.allowed_extensions.iter().any(|e| e == "*")
            && !rule
                .allowed_extensions
                .iter()
                .any(|e| e.eq_ignore_ascii_case(ext))
        {
            anyhow::bail!(
                "不允许的文件类型: .{}, 允许: {:?}",
                ext,
                rule.allowed_extensions
            );
        }

        // 校验文件大小
        if let Some(size) = req.size {
            if size > rule.max_size_bytes {
                anyhow::bail!(
                    "文件大小超出限制: {} 字节, 上限: {} 字节",
                    size,
                    rule.max_size_bytes
                );
            }
        }

        // 确定 Bucket：请求指定 > SceneRule > default
        let bucket = req
            .bucket
            .as_deref()
            .or(rule.bucket.as_deref())
            .unwrap_or(&self.config.default_bucket)
            .to_string();

        // 生成 object key：{scene}/{year}/{month}/{uuid}.{ext}
        let now = chrono::Utc::now();
        let object_key = format!(
            "{}/{}/{}/{}.{}",
            req.scene,
            now.format("%Y"),
            now.format("%m"),
            Uuid::new_v4(),
            ext,
        );

        // 创建元数据记录
        let meta = FileMeta {
            id: None,
            file_key: Some(object_key.clone()),
            bucket: Some(bucket.clone()),
            original_name: Some(req.filename),
            content_type: req.content_type.clone(),
            size: req.size,
            scene: Some(req.scene),
            uploader_id,
            provider: Some(self.config.provider.clone()),
            status: Some(0),
            watermark: Some(0),
            thumbnail_key: None,
            created_at: None,
            updated_at: None,
        };
        let file_id = FileMetaRepo::insert(&self.db, &meta).await?;

        // 生成预签名 URL
        let presigned = self
            .provider
            .presign_put(
                &bucket,
                &object_key,
                req.content_type.as_deref(),
                self.config.presign_expires_secs,
            )
            .await?;

        Ok(PresignUploadResponse {
            upload_url: presigned.url,
            object_key,
            file_id,
            expires_in: presigned.expires_in,
        })
    }

    /// 生成预签名下载 URL
    pub async fn presign_download(
        &self,
        req: PresignDownloadRequest,
    ) -> anyhow::Result<PresignDownloadResponse> {
        let bucket = req.bucket.as_deref().unwrap_or(&self.config.default_bucket);

        let presigned = self
            .provider
            .presign_get(bucket, &req.object_key, self.config.presign_expires_secs)
            .await?;

        Ok(PresignDownloadResponse {
            download_url: presigned.url,
            expires_in: presigned.expires_in,
        })
    }

    /// 上传完成回调
    pub async fn upload_callback(
        &self,
        req: UploadCallbackRequest,
    ) -> anyhow::Result<FileMetaResponse> {
        let meta = FileMetaRepo::find_by_id(&self.db, req.file_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("文件记录不存在: {}", req.file_id))?;

        let bucket = meta
            .bucket
            .as_deref()
            .unwrap_or(&self.config.default_bucket);
        let key = meta
            .file_key
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("文件记录缺少 file_key"))?;

        let obj_meta = self.provider.head_object(bucket, key).await?;
        let actual_size = obj_meta.size.unwrap_or(0);

        if let Some(expected_size) = meta.size {
            if expected_size > 0 && actual_size != expected_size {
                anyhow::bail!(
                    "文件大小不一致: 预期 {} 字节，实际 {} 字节",
                    expected_size,
                    actual_size
                );
            }
        }

        FileMetaRepo::update_confirm(&self.db, req.file_id, 1, actual_size).await?;

        Ok(Self::to_response(&meta, Some((1, Some(actual_size)))))
    }

    /// 查询文件元数据
    pub async fn get_file_meta(&self, id: i64) -> anyhow::Result<Option<FileMetaResponse>> {
        let meta = FileMetaRepo::find_by_id(&self.db, id).await?;
        Ok(meta.map(|m| Self::to_response(&m, None)))
    }

    /// 删除文件
    pub async fn delete_file(&self, id: i64) -> anyhow::Result<Option<()>> {
        let meta = match FileMetaRepo::find_by_id(&self.db, id).await? {
            Some(m) => m,
            None => return Ok(None),
        };

        let bucket = meta
            .bucket
            .as_deref()
            .unwrap_or(&self.config.default_bucket);
        let key = meta.file_key.as_deref().unwrap_or("");

        if let Err(e) = self.provider.delete_object(bucket, key).await {
            tracing::warn!(
                "OSS 文件删除失败（可能已不存在）: bucket={}, key={}, err={}",
                bucket,
                key,
                e
            );
        }

        FileMetaRepo::soft_delete(&self.db, id).await?;
        Ok(Some(()))
    }
}
