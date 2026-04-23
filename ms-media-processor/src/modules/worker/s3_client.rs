use crate::config::MediaConfig;
use crate::error::MediaError;
use s3::creds::Credentials;
use s3::{Bucket, Region};
use std::path::Path;
use tokio::fs::{File, self};
use tokio::io::{AsyncWriteExt, AsyncReadExt};

pub struct S3Client {
    config: MediaConfig,
}

impl S3Client {
    pub fn new(config: MediaConfig) -> Self {
        Self { config }
    }

    fn get_bucket(&self, bucket_name: &str) -> Result<Box<Bucket>, MediaError> {
        let credentials = Credentials::new(
            Some(&self.config.access_key),
            Some(&self.config.secret_key),
            None,
            None,
            None,
        ).map_err(|e| MediaError::InternalError(format!("S3 Creds config error: {}", e)))?;

        let region = Region::Custom {
            region: self.config.region.clone(),
            endpoint: self.config.endpoint.clone(),
        };

        // rust-s3 中的 Bucket 创建，设置路径样式以兼容 MinIO
        let mut bucket = Bucket::new(bucket_name, region, credentials)
            .map_err(|e| MediaError::InternalError(format!("S3 Bucket init error: {}", e)))?;
            
        bucket.set_path_style();

        // 避免发送 expect header，对老版本 nginx 或 minio 的兼容
        Ok(bucket)
    }

    /// 从 S3 拉取文件到本地
    pub async fn download_to_file(&self, bucket_name: &str, key: &str, local_path: &Path) -> Result<(), MediaError> {
        let bucket = self.get_bucket(bucket_name)?;
        
        let response_data = bucket.get_object(key).await.map_err(|e| {
            MediaError::S3Failed(format!("Failed to download {}/{}: {}", bucket_name, key, e))
        })?;
        
        let bytes = response_data.bytes();
        
        // 保证临时文件的父级目录存在
        if let Some(parent) = local_path.parent() {
            fs::create_dir_all(parent).await.map_err(|e| MediaError::InternalError(e.to_string()))?;
        }

        let mut file = File::create(local_path).await.map_err(|e| {
            MediaError::InternalError(format!("Failed to create local file {:?}: {}", local_path, e))
        })?;
        
        file.write_all(bytes).await.map_err(|e| {
            MediaError::InternalError(format!("Failed to write to local file {:?}: {}", local_path, e))
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
            MediaError::InternalError(format!("Failed to open local file {:?}: {}", local_path, e))
        })?;
        
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).await.map_err(|e| {
            MediaError::InternalError(format!("Failed to read local file {:?}: {}", local_path, e))
        })?;

        bucket.put_object_with_content_type(key, &buffer, content_type).await.map_err(|e| {
            MediaError::S3Failed(format!("Failed to upload to {}/{}: {}", bucket_name, key, e))
        })?;

        Ok(())
    }
}
