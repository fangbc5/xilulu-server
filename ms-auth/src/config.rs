use fbc_starter::Config as BaseConfig;
use serde::{Deserialize, Serialize};

/// Auth 服务配置
/// 扩展 fbc-starter 的基础配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    /// 基础配置（继承自 fbc-starter）
    #[serde(flatten)]
    pub base: BaseConfig,
    /// Auth 业务配置
    #[serde(default)]
    pub auth: AuthServiceConfig,
}

/// Auth 业务配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthServiceConfig {
    /// JWT 密钥
    pub jwt_secret: String,
    /// Access Token 过期时间（秒），默认 86400（24小时）
    #[serde(default = "default_access_token_timeout")]
    pub access_token_timeout: i64,
    /// Refresh Token 过期时间（秒），默认 604800（7天）
    #[serde(default = "default_refresh_token_timeout")]
    pub refresh_token_timeout: i64,
}

fn default_access_token_timeout() -> i64 {
    900 // 24小时
}

fn default_refresh_token_timeout() -> i64 {
    604800 // 7天
}

impl Default for AuthServiceConfig {
    fn default() -> Self {
        Self {
            jwt_secret: String::new(),
            access_token_timeout: default_access_token_timeout(),
            refresh_token_timeout: default_refresh_token_timeout(),
        }
    }
}

impl AuthConfig {
    /// 从环境变量加载配置
    pub fn new(base_config: BaseConfig) -> Self {
        let auth = AuthServiceConfig {
            jwt_secret: std::env::var("APP__AUTH__JWT_SECRET")
                .expect("APP__AUTH__JWT_SECRET 环境变量未设置"),
            access_token_timeout: std::env::var("APP__AUTH__ACCESS_TOKEN_TIMEOUT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or_else(default_access_token_timeout),
            refresh_token_timeout: std::env::var("APP__AUTH__REFRESH_TOKEN_TIMEOUT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or_else(default_refresh_token_timeout),
        };

        tracing::info!(
            access_token_timeout = auth.access_token_timeout,
            refresh_token_timeout = auth.refresh_token_timeout,
            "Auth 配置加载完成"
        );

        Self {
            base: base_config,
            auth,
        }
    }
}
