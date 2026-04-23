//! S3 兼容协议的统一 Provider 实现
//!
//! 适用于：RustFS、MinIO、AWS S3、阿里云 OSS（S3 兼容模式）、腾讯 COS（S3 兼容模式）
//!
//! 使用 `rust-s3` crate 实现，替代 `aws-sdk-s3`。

use async_trait::async_trait;
use s3::bucket::Bucket;
use s3::creds::Credentials;
use s3::serde_types::Part as S3Part;
use s3::Region;

use super::{ObjectMeta, OssProvider, PresignedUrl};

/// S3 兼容 Provider
///
/// 所有支持 S3 协议的对象存储都通过此实现适配。
/// 内部存储连接参数，每次操作按 bucket 名动态构建 `Bucket` 实例
///（`Bucket::new` 是同步纯内存操作，开销极低）。
pub struct S3CompatProvider {
    region: Region,
    /// 用于生成预签名 URL 的 region（使用客户端可访问的公网 endpoint）
    presign_region: Region,
    credentials: Credentials,
}

impl S3CompatProvider {
    /// 创建 S3 兼容 Provider（同步方法，可在 Server::run 闭包中直接调用）
    ///
    /// # Arguments
    /// * `endpoint` - 服务端点（例如 http://127.0.0.1:9000）
    /// * `region` - 区域（RustFS/MinIO 随意填）
    /// * `access_key` - 访问密钥
    /// * `secret_key` - 密钥
    pub fn new(
        endpoint: &str,
        public_endpoint: &str,
        region: &str,
        access_key: &str,
        secret_key: &str,
    ) -> Self {
        let region_val = Region::Custom {
            region: region.to_string(),
            endpoint: endpoint.to_string(),
        };

        let presign_region = Region::Custom {
            region: region.to_string(),
            endpoint: public_endpoint.to_string(),
        };

        let credentials = Credentials::new(
            Some(access_key),
            Some(secret_key),
            None,
            None,
            None,
        )
        .expect("Failed to create S3 credentials");

        Self {
            region: region_val,
            presign_region,
            credentials,
        }
    }

    /// 按 bucket 名构建 `Bucket` 实例（path-style，兼容 RustFS/MinIO）
    fn bucket(&self, bucket_name: &str) -> anyhow::Result<Box<Bucket>> {
        let mut bucket = Bucket::new(
            bucket_name,
            self.region.clone(),
            self.credentials.clone(),
        )?;
        bucket.set_path_style();
        Ok(bucket)
    }

    /// 按 bucket 名构建用于预签名的 `Bucket` 实例（使用公网 endpoint）
    fn presign_bucket(&self, bucket_name: &str) -> anyhow::Result<Box<Bucket>> {
        let mut bucket = Bucket::new(
            bucket_name,
            self.presign_region.clone(),
            self.credentials.clone(),
        )?;
        bucket.set_path_style();
        Ok(bucket)
    }
}

#[async_trait]
impl OssProvider for S3CompatProvider {
    async fn presign_put(
        &self,
        bucket: &str,
        key: &str,
        content_type: Option<&str>,
        expires_secs: u64,
    ) -> anyhow::Result<PresignedUrl> {
        let b = self.presign_bucket(bucket)?;

        // content_type 通过 custom_headers 传递
        let custom_headers = if let Some(ct) = content_type {
            let mut headers = http::HeaderMap::new();
            headers.insert(
                http::header::CONTENT_TYPE,
                http::HeaderValue::from_str(ct)?,
            );
            Some(headers)
        } else {
            None
        };

        let url = b
            .presign_put(key, expires_secs as u32, custom_headers, None)
            .await?;

        Ok(PresignedUrl {
            url,
            expires_in: expires_secs,
        })
    }

    async fn presign_get(
        &self,
        bucket: &str,
        key: &str,
        expires_secs: u64,
    ) -> anyhow::Result<PresignedUrl> {
        let b = self.presign_bucket(bucket)?;

        let url = b.presign_get(key, expires_secs as u32, None).await?;

        Ok(PresignedUrl {
            url,
            expires_in: expires_secs,
        })
    }

    async fn delete_object(&self, bucket: &str, key: &str) -> anyhow::Result<()> {
        let b = self.bucket(bucket)?;
        b.delete_object(key).await?;
        Ok(())
    }

    async fn head_object(&self, bucket: &str, key: &str) -> anyhow::Result<ObjectMeta> {
        let b = self.bucket(bucket)?;
        let (head, _status) = b.head_object(key).await?;

        Ok(ObjectMeta {
            bucket: bucket.to_string(),
            key: key.to_string(),
            size: head.content_length,
            content_type: head.content_type,
        })
    }

    // ========================================================
    // 分片上传实现
    // ========================================================

    async fn create_multipart(
        &self,
        bucket: &str,
        key: &str,
        content_type: Option<&str>,
    ) -> anyhow::Result<String> {
        let b = self.bucket(bucket)?;

        let ct = content_type.unwrap_or("application/octet-stream");
        let response = b.initiate_multipart_upload(key, ct).await?;
        Ok(response.upload_id)
    }

    async fn presign_upload_part(
        &self,
        bucket: &str,
        key: &str,
        upload_id: &str,
        part_number: u32,
        expires_secs: u64,
    ) -> anyhow::Result<String> {
        let b = self.presign_bucket(bucket)?;

        // 构建分片上传的预签名 URL
        // rust-s3 没有直接的 presign_upload_part 方法，
        // 我们通过 presign_put 并附加 query 参数来实现
        let path = format!(
            "{}?partNumber={}&uploadId={}",
            key, part_number, upload_id
        );
        let url = b
            .presign_put(&path, expires_secs as u32, None, None)
            .await?;

        Ok(url)
    }

    async fn complete_multipart(
        &self,
        bucket: &str,
        key: &str,
        upload_id: &str,
        parts: &[(u32, String)],
    ) -> anyhow::Result<()> {
        let b = self.bucket(bucket)?;

        // 构建 Part 列表（rust-s3 使用 serde_types::Part）
        let part_list: Vec<S3Part> = parts
            .iter()
            .map(|(num, etag)| S3Part {
                part_number: *num,
                etag: etag.clone(),
            })
            .collect();

        b.complete_multipart_upload(key, upload_id, part_list)
            .await?;

        Ok(())
    }

    async fn abort_multipart(
        &self,
        bucket: &str,
        key: &str,
        upload_id: &str,
    ) -> anyhow::Result<()> {
        let b = self.bucket(bucket)?;
        // rust-s3 0.37 方法名为 abort_upload
        b.abort_upload(key, upload_id).await?;
        Ok(())
    }

    async fn list_parts(
        &self,
        bucket: &str,
        key: &str,
        upload_id: &str,
    ) -> anyhow::Result<Vec<(u32, String, i64)>> {
        // rust-s3 0.37 没有直接的 list_parts API
        // 可以通过 list_multiparts_uploads 找到对应的 upload，但无法列出分片
        // 这里暂时返回空列表，待后续通过 raw HTTP 请求实现
        // 实际生产中客户端应自行跟踪已上传分片
        let _b = self.bucket(bucket)?;
        let _ = (key, upload_id); // 避免 unused 警告
        tracing::warn!("list_parts 暂未实现（rust-s3 0.37 无此直接 API），建议客户端本地跟踪分片状态");
        Ok(Vec::new())
    }
}
