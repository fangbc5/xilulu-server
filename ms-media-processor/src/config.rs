use serde::{Deserialize, Serialize};

/// S3/MinIO 配置
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MediaConfig {
    /// S3 端点
    pub endpoint: String,
    /// S3 区域
    pub region: String,
    /// Access Key
    pub access_key: String,
    /// Secret Key
    pub secret_key: String,
    /// 公网访问端点（可选，用于生成公开 URL）
    pub public_endpoint: String,
}

impl MediaConfig {
    /// 从环境变量加载
    pub fn from_env() -> Self {
        Self {
            endpoint: std::env::var("OSS__ENDPOINT")
                .unwrap_or_else(|_| "http://127.0.0.1:9000".to_string()),
            public_endpoint: std::env::var("OSS__PUBLIC_ENDPOINT")
                .unwrap_or_else(|_| {
                    std::env::var("OSS__ENDPOINT")
                        .unwrap_or_else(|_| "http://127.0.0.1:9000".to_string())
                }),
            region: std::env::var("OSS__REGION")
                .unwrap_or_else(|_| "us-east-1".to_string()),
            access_key: std::env::var("OSS__ACCESS_KEY")
                .unwrap_or_else(|_| "minioadmin".to_string()),
            secret_key: std::env::var("OSS__SECRET_KEY")
                .unwrap_or_else(|_| "minioadmin".to_string()),
        }
    }
}
