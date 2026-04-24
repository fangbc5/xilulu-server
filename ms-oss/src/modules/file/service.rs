//! 文件服务层 — 对标阿里云 OSS 的核心业务逻辑
//!
//! 职责：
//! - 签名服务：生成上传/下载预签名 URL、长效分享链接
//! - 对象操作：PutObject → PostObject(确认) → GetObject(302分发) → HeadObject → DeleteObject
//! - 分片上传：InitiateMultipart → CompleteMultipart → ListParts → AbortMultipart
//! - 302 分发引擎：原文件 → S3 / 图片 → imgproxy / 视频 → DB 查产物 / Style → 展开

use fbc_starter::messaging::{Message, MessageProducerType};
use sqlxplus::DbPool;
use std::sync::Arc;
use uuid::Uuid;

use super::model::dto::*;
use super::model::entity::FileMeta;
use super::repository::FileMetaRepo;
use crate::config::OssConfig;
use crate::error::OssError;
use crate::provider::OssProvider;
use crate::utils::{imgproxy, jwt, oss_process};

/// 文件服务 — HeadObject 返回的元信息
pub struct HeadObjectMeta {
    pub content_type: Option<String>,
    pub size: Option<i64>,
    pub original_name: Option<String>,
    pub scene: Option<String>,
    pub thumbnail_key: Option<String>,
}

/// 文件服务
pub struct FileService {
    db: Arc<DbPool>,
    config: OssConfig,
    provider: Arc<dyn OssProvider>,
    message_producer: Option<MessageProducerType>,
}

impl FileService {
    pub fn new(
        db: Arc<DbPool>,
        config: OssConfig,
        provider: Arc<dyn OssProvider>,
        message_producer: Option<MessageProducerType>,
    ) -> Self {
        Self {
            db,
            config,
            provider,
            message_producer,
        }
    }

    // ========================================================
    // 签名服务
    // ========================================================

    /// 统一签名 — 上传
    pub async fn signature_upload(
        &self,
        req: SignatureRequest,
        uploader_id: Option<i64>,
    ) -> Result<SignatureUploadResponse, OssError> {
        let scene = req.scene.as_deref().unwrap_or("default");
        let rule = self.config.get_scene_rule(scene);

        // 原始文件名提取扩展名
        let filename = req.filename.as_deref().unwrap_or("file.bin");
        let ext = filename.rsplit('.').next().unwrap_or("bin");

        // 校验文件类型
        self.validate_extension(ext, &rule.allowed_extensions)?;

        // 校验文件大小
        if let Some(size) = req.size {
            self.validate_size(size, rule.max_size_bytes)?;
        }

        // 确定 Bucket
        let bucket = req
            .bucket
            .as_deref()
            .or(rule.bucket.as_deref())
            .unwrap_or(&self.config.default_bucket)
            .to_string();

        // 生成或使用指定的 object key
        let object_key = if let Some(key) = &req.key {
            key.clone()
        } else {
            self.generate_object_key(scene, ext)
        };

        // 异步写入审计记录
        self.audit_insert(
            &bucket,
            &object_key,
            Some(filename.to_string()),
            req.content_type.clone(),
            req.size,
            scene,
            uploader_id,
        )
        .await;

        // 生成预签名 URL
        let presigned = self
            .provider
            .presign_put(
                &bucket,
                &object_key,
                req.content_type.as_deref(),
                self.config.presign_expires_secs,
            )
            .await
            .map_err(|e| OssError::PresignFailed(e.to_string()))?;

        Ok(SignatureUploadResponse {
            url: presigned.url,
            object_key,
            method: "PUT".to_string(),
            expires_in: presigned.expires_in,
        })
    }

    /// 统一签名 — 下载
    pub async fn signature_download(
        &self,
        req: SignatureRequest,
    ) -> Result<SignatureDownloadResponse, OssError> {
        let bucket = req.bucket.as_deref().unwrap_or(&self.config.default_bucket);
        let key = req
            .key
            .as_deref()
            .ok_or_else(|| OssError::BadRequest("下载签名缺少 key 参数".into()))?;
        let expires = req.expires_in.unwrap_or(600);

        let presigned = self
            .provider
            .presign_get(bucket, key, expires)
            .await
            .map_err(|e| OssError::PresignFailed(e.to_string()))?;

        Ok(SignatureDownloadResponse {
            url: presigned.url,
            method: "GET".to_string(),
            expires_in: presigned.expires_in,
        })
    }

    /// 统一签名 — 长效分享
    pub async fn signature_share(
        &self,
        req: SignatureRequest,
    ) -> Result<SignatureShareResponse, OssError> {
        let bucket = req.bucket.as_deref().unwrap_or(&self.config.default_bucket);
        let key = req
            .key
            .as_deref()
            .ok_or_else(|| OssError::BadRequest("分享签名缺少 key 参数".into()))?;
        let expires_in = req.expires_in.unwrap_or(31536000); // 默认 1 年

        let token = jwt::create_share_token(
            &self.config.share.jwt_secret,
            bucket,
            key,
            req.x_oss_process.as_deref(),
            expires_in,
        )
        .map_err(|e| OssError::InternalError(e.to_string()))?;

        Ok(SignatureShareResponse {
            url: format!("/oss/share/{}", token),
            expires_in,
        })
    }

    /// 解析长效分享链接 → 返回 302 重定向目标 URL
    pub async fn resolve_share(&self, token: &str) -> Result<String, OssError> {
        let claims = jwt::verify_share_token(&self.config.share.jwt_secret, token)
            .map_err(|e| OssError::ShareInvalid(e.to_string()))?;

        self.dispatch_get(&claims.bucket, &claims.key, claims.process)
            .await
    }

    // ========================================================
    // 对象操作
    // ========================================================

    /// PutObject — 预签名上传
    pub async fn put_object(
        &self,
        bucket: &str,
        key: &str,
        content_type: Option<&str>,
        scene: Option<&str>,
        original_name: Option<&str>,
        size: Option<i64>,
        uploader_id: Option<i64>,
    ) -> Result<PutObjectResponse, OssError> {
        let scene = scene.unwrap_or("default");
        let rule = self.config.get_scene_rule(scene);

        // 从 key 提取扩展名校验
        let ext = key.rsplit('.').next().unwrap_or("bin");
        self.validate_extension(ext, &rule.allowed_extensions)?;

        if let Some(s) = size {
            self.validate_size(s, rule.max_size_bytes)?;
        }

        // 异步写入审计记录
        self.audit_insert(
            bucket,
            key,
            original_name.map(|s| s.to_string()),
            content_type.map(|s| s.to_string()),
            size,
            scene,
            uploader_id,
        )
        .await;

        // 生成预签名 URL
        let presigned = self
            .provider
            .presign_put(bucket, key, content_type, self.config.presign_expires_secs)
            .await
            .map_err(|e| OssError::PresignFailed(e.to_string()))?;

        Ok(PutObjectResponse {
            upload_url: presigned.url,
            object_key: key.to_string(),
            expires_in: presigned.expires_in,
        })
    }

    /// PostObject（无 query）— 上传完成确认
    ///
    /// 由 Kafka MinIO 事件驱动调用。内置幂等保护：若 file_meta 已确认则跳过。
    pub async fn confirm_upload(
        &self,
        bucket: &str,
        key: &str,
    ) -> Result<ObjectInfoResponse, OssError> {
        // 幂等保护：如果 DB 中已有记录且 status=1（已确认），直接返回
        if let Ok(Some(existing)) = FileMetaRepo::find_by_bucket_key(&self.db, bucket, key).await {
            if existing.status == Some(1) {
                tracing::debug!("文件 {}/{} 已确认过，跳过重复处理", bucket, key);
                return Ok(ObjectInfoResponse {
                    bucket: bucket.to_string(),
                    key: key.to_string(),
                    content_type: existing.content_type,
                    size: existing.size,
                });
            }
        }

        // 通过 S3 head_object 验证文件确实存在
        let obj_meta = self
            .provider
            .head_object(bucket, key)
            .await
            .map_err(|e| OssError::FileNotFound(format!("文件不存在于存储中: {}", e)))?;

        // 异步更新审计记录
        self.audit_confirm(bucket, key, obj_meta.size.unwrap_or(0))
            .await;

        // 视频文件自动派发处理任务到 Kafka
        let is_video = obj_meta
            .content_type
            .as_ref()
            .map(|ct| ct.starts_with("video/"))
            .unwrap_or_else(|| {
                key.ends_with(".mp4") || key.ends_with(".mov") || key.ends_with(".avi")
            });

        if is_video {
            if let Some(producer) = &self.message_producer {
                let msg = Message::new(
                    "sys.media.task.submit",
                    "ms-oss",
                    serde_json::json!({
                        "bucket": bucket,
                        "key": key,
                        "action": "extract_video_thumbnail"
                    }),
                );
                let producer_clone = producer.clone();
                let topic = "sys.media.task.submit".to_string();
                let bucket_clone = bucket.to_string();
                let key_clone = key.to_string();
                tokio::spawn(async move {
                    if let Err(e) = producer_clone.publish(&topic, msg).await {
                        tracing::error!("视频任务派发失败 {}: {}", key_clone, e);
                    } else {
                        tracing::info!(
                            "向 Kafka [sys.media.task.submit] 提交了视频处理任务: {}/{}",
                            bucket_clone,
                            key_clone
                        );
                    }
                });
            }
        }

        Ok(ObjectInfoResponse {
            bucket: bucket.to_string(),
            key: key.to_string(),
            content_type: obj_meta.content_type,
            size: obj_meta.size,
        })
    }

    /// GetObject — 核心 302 分发引擎
    ///
    /// 根据 x-oss-process 参数决定重定向目标：
    /// - None → 原文件下载（302 → S3 presigned URL）
    /// - image/* → imgproxy 实时处理（302 → imgproxy 签名 URL）
    /// - video/* → 视频截帧产物（302 → thumbnail_key 的 presigned URL）
    /// - style/* → 展开后递归处理
    pub fn dispatch_get<'a>(
        &'a self,
        bucket: &'a str,
        key: &'a str,
        x_oss_process: Option<String>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<String, OssError>> + Send + 'a>>
    {
        Box::pin(async move {
            match x_oss_process.as_deref() {
                None => {
                    // 场景 A：原文件下载
                    let presigned = self
                        .provider
                        .presign_get(bucket, key, 600)
                        .await
                        .map_err(|e| OssError::PresignFailed(e.to_string()))?;
                    Ok(presigned.url)
                }
                Some(raw) => {
                    let params = oss_process::parse(raw)
                        .map_err(|e| OssError::ProcessParseError(e.to_string()))?;

                    match params.process_type {
                        oss_process::ProcessType::Image => {
                            // 场景 B：图片实时处理 → imgproxy
                            let url = imgproxy::build_url(
                                &self.config.imgproxy,
                                &params.imgproxy_processing,
                                bucket,
                                key,
                                params.output_format.as_deref(),
                            )
                            .map_err(|e| OssError::InternalError(e.to_string()))?;
                            Ok(url)
                        }
                        oss_process::ProcessType::Video => {
                            // 场景 C：视频截帧产物
                            self.dispatch_video_product(bucket, key, &params).await
                        }
                        oss_process::ProcessType::Style(style_name) => {
                            // 场景 D：Style 预设 → 展开后递归
                            let expanded = self
                                .config
                                .get_style(&style_name)
                                .ok_or_else(|| {
                                    OssError::StyleNotFound(format!("样式不存在: {}", style_name))
                                })?
                                .to_string();
                            self.dispatch_get(bucket, key, Some(expanded)).await
                        }
                    }
                }
            }
        })
    }

    /// 视频截帧产物分发
    async fn dispatch_video_product(
        &self,
        bucket: &str,
        key: &str,
        params: &oss_process::ProcessParams,
    ) -> Result<String, OssError> {
        // 查 DB 获取 thumbnail_key
        let meta = FileMetaRepo::find_by_bucket_key(&self.db, bucket, key).await;
        let meta = match meta {
            Ok(Some(m)) => m,
            _ => {
                return Err(OssError::FileNotFound(format!(
                    "视频元信息不存在: {}/{}",
                    bucket, key
                )));
            }
        };

        let thumbnail_key = meta
            .thumbnail_key
            .ok_or_else(|| OssError::FileNotFound("视频截帧尚未完成或不存在".into()))?;

        // 如果有额外的 width/height 参数，通过 imgproxy 二次 resize
        if params.video_width.is_some() || params.video_height.is_some() {
            let w = params.video_width.unwrap_or(0);
            let h = params.video_height.unwrap_or(0);
            let processing = format!("rs:fit:{}:{}", w, h);
            let url = imgproxy::build_url(
                &self.config.imgproxy,
                &processing,
                bucket,
                &thumbnail_key,
                None,
            )
            .map_err(|e| OssError::InternalError(e.to_string()))?;
            Ok(url)
        } else {
            // 直接返回截帧图的 presigned URL
            let presigned = self
                .provider
                .presign_get(bucket, &thumbnail_key, 600)
                .await
                .map_err(|e| OssError::PresignFailed(e.to_string()))?;
            Ok(presigned.url)
        }
    }

    /// HeadObject — 获取文件元数据
    pub async fn head_object(&self, bucket: &str, key: &str) -> Result<HeadObjectMeta, OssError> {
        // 先查 DB 审计记录
        if let Ok(Some(meta)) = FileMetaRepo::find_by_bucket_key(&self.db, bucket, key).await {
            return Ok(HeadObjectMeta {
                content_type: meta.content_type,
                size: meta.size,
                original_name: meta.original_name,
                scene: meta.scene,
                thumbnail_key: meta.thumbnail_key,
            });
        }

        // DB 无记录，尝试直接查 S3
        let obj_meta = self
            .provider
            .head_object(bucket, key)
            .await
            .map_err(|e| OssError::FileNotFound(format!("文件不存在: {}", e)))?;

        Ok(HeadObjectMeta {
            content_type: obj_meta.content_type,
            size: obj_meta.size,
            original_name: None,
            scene: None,
            thumbnail_key: None,
        })
    }

    /// DeleteObject — 删除文件
    pub async fn delete_object(&self, bucket: &str, key: &str) -> Result<(), OssError> {
        // 查 DB 获取关联的 thumbnail_key
        if let Ok(Some(meta)) = FileMetaRepo::find_by_bucket_key(&self.db, bucket, key).await {
            // 删除缩略图
            if let Some(thumb_key) = &meta.thumbnail_key {
                let _ = self.provider.delete_object(bucket, thumb_key).await;
            }
            // 标记删除
            if let Some(id) = meta.id {
                let _ = FileMetaRepo::soft_delete(&self.db, id).await;
            }
        }

        // 删除原文件（无论 DB 是否有记录）
        let _ = self.provider.delete_object(bucket, key).await;
        Ok(())
    }

    // ========================================================
    // 分片上传
    // ========================================================

    /// InitiateMultipartUpload — 初始化分片上传
    pub async fn initiate_multipart(
        &self,
        bucket: &str,
        key: &str,
        scene: &str,
        content_type: Option<&str>,
        total_size: Option<i64>,
        part_size: Option<i64>,
    ) -> Result<MultipartInitResponse, OssError> {
        let rule = self.config.get_scene_rule(scene);

        // 校验总大小
        if let Some(total) = total_size {
            self.validate_size(total, rule.max_size_bytes)?;
        }

        // 调用 S3 CreateMultipartUpload
        let upload_id = self
            .provider
            .create_multipart(bucket, key, content_type)
            .await
            .map_err(|e| OssError::MultipartError(e.to_string()))?;

        // 计算分片
        let part_sz = part_size.unwrap_or(5 * 1024 * 1024); // 默认 5MB
        let total = total_size.unwrap_or(part_sz); // 若未提供总大小，假设1个分片
        let part_count = ((total as f64) / (part_sz as f64)).ceil() as usize;

        // 批量生成各分片的 presigned URL
        let mut part_urls = Vec::with_capacity(part_count);
        for i in 1..=(part_count as u32) {
            let url = self
                .provider
                .presign_upload_part(bucket, key, &upload_id, i, 7200)
                .await
                .map_err(|e| OssError::MultipartError(e.to_string()))?;
            part_urls.push(PartUrlInfo {
                part_number: i,
                upload_url: url,
            });
        }

        // 异步写入审计记录
        self.audit_insert(
            bucket,
            key,
            None,
            content_type.map(|s| s.to_string()),
            total_size,
            scene,
            None,
        )
        .await;

        Ok(MultipartInitResponse {
            upload_id,
            object_key: key.to_string(),
            part_count,
            part_urls,
            expires_in: 7200,
        })
    }

    /// CompleteMultipartUpload — 完成分片上传
    pub async fn complete_multipart(
        &self,
        bucket: &str,
        key: &str,
        upload_id: &str,
        parts: Vec<PartInfo>,
    ) -> Result<ObjectInfoResponse, OssError> {
        // 转换为 provider 需要的格式
        let part_list: Vec<(u32, String)> =
            parts.into_iter().map(|p| (p.part_number, p.etag)).collect();

        self.provider
            .complete_multipart(bucket, key, upload_id, &part_list)
            .await
            .map_err(|e| OssError::MultipartError(e.to_string()))?;

        // 获取合并后的文件元信息
        let obj_meta = self
            .provider
            .head_object(bucket, key)
            .await
            .map_err(|e| OssError::InternalError(e.to_string()))?;

        // 异步更新审计记录
        // 注意：视频任务派发统一由 Kafka MinIO 事件消费者（confirm_upload）处理，
        // MinIO 合并分片后会自动发出 s3:ObjectCreated:CompleteMultipartUpload 事件，
        // 因此此处不再重复派发，避免同一视频被双重处理。
        self.audit_confirm(bucket, key, obj_meta.size.unwrap_or(0))
            .await;

        Ok(ObjectInfoResponse {
            bucket: bucket.to_string(),
            key: key.to_string(),
            content_type: obj_meta.content_type,
            size: obj_meta.size,
        })
    }

    /// ListParts — 查询已上传分片
    pub async fn list_parts(
        &self,
        bucket: &str,
        key: &str,
        upload_id: &str,
    ) -> Result<ListPartsResponse, OssError> {
        let parts = self
            .provider
            .list_parts(bucket, key, upload_id)
            .await
            .map_err(|e| OssError::MultipartError(e.to_string()))?;

        let next = parts.last().map(|p| p.0 + 1).unwrap_or(1);
        let part_details = parts
            .into_iter()
            .map(|(num, etag, size)| PartDetail {
                part_number: num,
                etag,
                size,
            })
            .collect();

        Ok(ListPartsResponse {
            upload_id: upload_id.to_string(),
            parts: part_details,
            next_part_number: next,
        })
    }

    /// AbortMultipartUpload — 取消分片上传
    pub async fn abort_multipart(
        &self,
        bucket: &str,
        key: &str,
        upload_id: &str,
    ) -> Result<(), OssError> {
        self.provider
            .abort_multipart(bucket, key, upload_id)
            .await
            .map_err(|e| OssError::MultipartError(e.to_string()))?;
        Ok(())
    }

    // ========================================================
    // 内部工具方法
    // ========================================================

    /// 生成 object key：{scene}/{year}/{month}/{day}/{uuid}.{ext}
    fn generate_object_key(&self, scene: &str, ext: &str) -> String {
        let now = chrono::Utc::now();
        format!(
            "{}/{}/{}/{}/{}.{}",
            scene,
            now.format("%Y"),
            now.format("%m"),
            now.format("%d"),
            Uuid::new_v4(),
            ext,
        )
    }

    /// 校验文件扩展名
    fn validate_extension(&self, ext: &str, allowed: &[String]) -> Result<(), OssError> {
        if allowed.is_empty() || allowed.iter().any(|e| e == "*") {
            return Ok(());
        }
        if !allowed.iter().any(|e| e.eq_ignore_ascii_case(ext)) {
            return Err(OssError::FileTypeNotAllowed(format!(
                "不允许的文件类型: .{}, 允许: {:?}",
                ext, allowed
            )));
        }
        Ok(())
    }

    /// 校验文件大小
    fn validate_size(&self, size: i64, max: i64) -> Result<(), OssError> {
        if size > max {
            return Err(OssError::FileTooLarge(format!(
                "文件大小超出限制: {} 字节, 上限: {} 字节",
                size, max
            )));
        }
        Ok(())
    }

    /// 异步写入审计记录（不阻塞主流程）
    async fn audit_insert(
        &self,
        bucket: &str,
        key: &str,
        original_name: Option<String>,
        content_type: Option<String>,
        size: Option<i64>,
        scene: &str,
        uploader_id: Option<i64>,
    ) {
        let db = self.db.clone();
        let meta = FileMeta {
            id: None,
            file_key: Some(key.to_string()),
            bucket: Some(bucket.to_string()),
            original_name,
            content_type,
            size,
            scene: Some(scene.to_string()),
            uploader_id,
            provider: Some(self.config.provider.clone()),
            status: Some(0),
            watermark: Some(0),
            thumbnail_key: None,
            created_at: None,
            updated_at: None,
        };
        tokio::spawn(async move {
            if let Err(e) = FileMetaRepo::insert(&db, &meta).await {
                tracing::warn!("审计记录写入失败: {}", e);
            }
        });
    }

    /// 异步更新审计记录（上传确认）
    async fn audit_confirm(&self, bucket: &str, key: &str, actual_size: i64) {
        let db = self.db.clone();
        let bucket = bucket.to_string();
        let key = key.to_string();
        tokio::spawn(async move {
            if let Ok(Some(meta)) = FileMetaRepo::find_by_bucket_key(&db, &bucket, &key).await {
                if let Some(id) = meta.id {
                    let _ = FileMetaRepo::update_confirm(&db, id, 1, actual_size).await;
                }
            }
        });
    }
}
