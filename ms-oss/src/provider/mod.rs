//! 多厂商 OSS Provider 适配层
//!
//! 通过 `OssProvider` trait 抽象各厂商差异，
//! 上层业务代码面向 trait 编程，切换厂商只需修改配置。

pub mod s3_compat;

use async_trait::async_trait;

/// 预签名 URL 结果
#[derive(Debug)]
pub struct PresignedUrl {
    /// 预签名 URL
    pub url: String,
    /// 过期时间（秒）
    pub expires_in: u64,
}

/// 文件元信息
#[derive(Debug)]
pub struct ObjectMeta {
    /// Bucket 名称
    pub bucket: String,
    /// 对象 Key（路径）
    pub key: String,
    /// 文件大小（字节）
    pub size: Option<i64>,
    /// Content-Type
    pub content_type: Option<String>,
}

/// OSS Provider 抽象 trait
///
/// 所有厂商适配器都需要实现此 trait。
/// 当前基于 S3 协议的厂商（RustFS/MinIO/AWS/阿里云 OSS/腾讯 COS）
/// 均可使用 `S3CompatProvider` 统一实现。
#[async_trait]
pub trait OssProvider: Send + Sync {
    /// 生成预签名上传 URL
    async fn presign_put(
        &self,
        bucket: &str,
        key: &str,
        content_type: Option<&str>,
        expires_secs: u64,
    ) -> anyhow::Result<PresignedUrl>;

    /// 生成预签名下载 URL
    async fn presign_get(
        &self,
        bucket: &str,
        key: &str,
        expires_secs: u64,
    ) -> anyhow::Result<PresignedUrl>;

    /// 删除对象
    async fn delete_object(&self, bucket: &str, key: &str) -> anyhow::Result<()>;

    /// 获取对象元信息
    async fn head_object(&self, bucket: &str, key: &str) -> anyhow::Result<ObjectMeta>;

    // ========================================================
    // 分片上传接口
    // ========================================================

    /// 初始化分片上传，返回 upload_id
    async fn create_multipart(
        &self,
        bucket: &str,
        key: &str,
        content_type: Option<&str>,
    ) -> anyhow::Result<String>;

    /// 生成单个分片的预签名上传 URL
    async fn presign_upload_part(
        &self,
        bucket: &str,
        key: &str,
        upload_id: &str,
        part_number: u32,
        expires_secs: u64,
    ) -> anyhow::Result<String>;

    /// 完成分片上传
    async fn complete_multipart(
        &self,
        bucket: &str,
        key: &str,
        upload_id: &str,
        parts: &[(u32, String)],
    ) -> anyhow::Result<()>;

    /// 取消分片上传
    async fn abort_multipart(
        &self,
        bucket: &str,
        key: &str,
        upload_id: &str,
    ) -> anyhow::Result<()>;

    /// 列出已上传的分片，返回 (part_number, etag, size)
    async fn list_parts(
        &self,
        bucket: &str,
        key: &str,
        upload_id: &str,
    ) -> anyhow::Result<Vec<(u32, String, i64)>>;
}
