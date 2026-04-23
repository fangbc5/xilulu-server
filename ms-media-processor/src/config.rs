use fbc_starter::Config as BaseConfig;
use serde::{Deserialize, Serialize};

/// 媒体处理服务配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaProcessorConfig {
    #[serde(flatten)]
    pub base: BaseConfig,
    
    #[serde(default)]
    pub media: MediaConfig,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MediaConfig {
    pub endpoint: String,
    pub region: String,
    pub access_key: String,
    pub secret_key: String,
    pub public_endpoint: String,
}

impl MediaProcessorConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        let base = BaseConfig::from_env()?;
        let media = MediaConfig {
            endpoint: std::env::var("OSS__ENDPOINT")
                .unwrap_or_else(|_| "http://127.0.0.1:9000".to_string()),
            public_endpoint: std::env::var("OSS__PUBLIC_ENDPOINT")
                .unwrap_or_else(|_| std::env::var("OSS__ENDPOINT")
                    .unwrap_or_else(|_| "http://127.0.0.1:9000".to_string())),
            region: std::env::var("OSS__REGION")
                .unwrap_or_else(|_| "us-east-1".to_string()),
            access_key: std::env::var("OSS__ACCESS_KEY")
                .unwrap_or_else(|_| "minioadmin".to_string()),
            secret_key: std::env::var("OSS__SECRET_KEY")
                .unwrap_or_else(|_| "minioadmin".to_string()),
        };

        Ok(Self { base, media })
    }
}


