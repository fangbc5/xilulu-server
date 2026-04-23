//! S3/MinIO 文件操作客户端

use crate::config::MediaConfig;
use crate::error::MediaError;
use s3::creds::Credentials;
use s3::{Bucket, Region};
use std::path::Path;
use tokio::fs::{self, File};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// S3 客户端 — 封装 MinIO/S3 文件下载和上传
pub struct S3Client {
    config: MediaConfig,
}

impl S3Client {
    pub fn new(config: MediaConfig) -> Self {
        Self { config }
    }

    /// 创建 S3 Bucket 实例
    fn get_bucket(&self, bucket_name: &str) -> Result<Box<Bucket>, MediaError> {
        let credentials = Credentials::new(
            Some(&self.config.access_key),
            Some(&self.config.secret_key),
            None, None, None,
        )
        .map_err(|e| MediaError::InternalError(format!("S3 凭证配置错误: {}", e)))?;

        let region = Region::Custom {
            region: self.config.region.clone(),
            endpoint: self.config.endpoint.clone(),
        };

        let mut bucket = Bucket::new(bucket_name, region, credentials)
            .map_err(|e| MediaError::InternalError(format!("S3 Bucket 初始化错误: {}", e)))?;

        bucket.set_path_style(); // MinIO 兼容
        Ok(bucket)
    }

    /// 从 S3 下载文件到本地
    pub async fn download_to_file(
        &self,
        bucket_name: &str,
        key: &str,
        local_path: &Path,
    ) -> Result<(), MediaError> {
        let bucket = self.get_bucket(bucket_name)?;

        let response = bucket.get_object(key).await.map_err(|e| {
            MediaError::S3Failed(format!("下载 {}/{} 失败: {}", bucket_name, key, e))
        })?;

        // 确保父目录存在
        if let Some(parent) = local_path.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|e| MediaError::InternalError(e.to_string()))?;
        }

        let mut file = File::create(local_path).await.map_err(|e| {
            MediaError::InternalError(format!("创建本地文件 {:?} 失败: {}", local_path, e))
        })?;

        file.write_all(response.bytes()).await.map_err(|e| {
            MediaError::InternalError(format!("写入本地文件 {:?} 失败: {}", local_path, e))
        })?;

        Ok(())
    }

    /// 将本地文件上传到 S3
    pub async fn upload_from_file(
        &self,
        bucket_name: &str,
        key: &str,
        local_path: &Path,
        content_type: &str,
    ) -> Result<(), MediaError> {
        let bucket = self.get_bucket(bucket_name)?;

        let mut file = File::open(local_path).await.map_err(|e| {
            MediaError::InternalError(format!("打开本地文件 {:?} 失败: {}", local_path, e))
        })?;

        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).await.map_err(|e| {
            MediaError::InternalError(format!("读取本地文件 {:?} 失败: {}", local_path, e))
        })?;

        bucket
            .put_object_with_content_type(key, &buffer, content_type)
            .await
            .map_err(|e| {
                MediaError::S3Failed(format!("上传 {}/{} 失败: {}", bucket_name, key, e))
            })?;

        Ok(())
    }
}
