use fbc_starter::Config as BaseConfig;
use serde::{Deserialize, Serialize};

/// 身份服务配置
/// 扩展 fbc-starter 的配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityConfig {
    /// 基础配置（继承自 fbc-starter）
    #[serde(flatten)]
    pub base: BaseConfig,
    /// 身份服务配置
    pub identity: IdentityServiceConfig,
}

/// 身份服务配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityServiceConfig {
    /// JWT Token 配置
    #[serde(default = "default_jwt_config")]
    pub jwt: JwtConfig,
    /// Session 配置
    #[serde(default = "default_session_config")]
    pub session: SessionConfig,
    /// 密码加密配置
    #[serde(default = "default_password_config")]
    pub password: PasswordConfig,
}

fn default_jwt_config() -> JwtConfig {
    JwtConfig {
        secret: "your-secret-key-change-in-production".to_string(),
        access_token_expire: default_access_token_expire(),
        refresh_token_expire: default_refresh_token_expire(),
    }
}

fn default_session_config() -> SessionConfig {
    SessionConfig {
        expire: default_session_expire(),
    }
}

fn default_password_config() -> PasswordConfig {
    PasswordConfig {
        min_length: default_min_length(),
    }
}

/// JWT Token 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtConfig {
    /// Token 密钥
    pub secret: String,
    /// Access Token 过期时间（秒）
    #[serde(default = "default_access_token_expire")]
    pub access_token_expire: u64,
    /// Refresh Token 过期时间（秒）
    #[serde(default = "default_refresh_token_expire")]
    pub refresh_token_expire: u64,
}

fn default_access_token_expire() -> u64 {
    900 // 15分钟
}

fn default_refresh_token_expire() -> u64 {
    604800 // 7天
}

/// Session 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    /// Session 过期时间（秒）
    #[serde(default = "default_session_expire")]
    pub expire: u64,
}

fn default_session_expire() -> u64 {
    86400 // 24小时
}

/// 密码加密配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordConfig {
    /// 密码最小长度
    #[serde(default = "default_min_length")]
    pub min_length: usize,
}

fn default_min_length() -> usize {
    8
}

impl IdentityConfig {
    /// 从环境变量加载配置
    pub fn new(base_config: BaseConfig) -> Result<Self, config::ConfigError> {
        // 直接读取环境变量，如果不存在则使用默认值
        let identity_config = IdentityServiceConfig {
            jwt: JwtConfig {
                secret: std::env::var("APP__JWT__SECRET")
                    .unwrap_or_else(|_| "your-secret-key-change-in-production".to_string()),
                access_token_expire: std::env::var("APP__IDENTITY__JWT__ACCESS_TOKEN_EXPIRE")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or_else(default_access_token_expire),
                refresh_token_expire: std::env::var("APP__IDENTITY__JWT__REFRESH_TOKEN_EXPIRE")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or_else(default_refresh_token_expire),
            },
            session: SessionConfig {
                expire: std::env::var("APP__IDENTITY__SESSION__EXPIRE")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or_else(default_session_expire),
            },
            password: PasswordConfig {
                min_length: std::env::var("APP__IDENTITY__PASSWORD__MIN_LENGTH")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or_else(default_min_length),
            },
        };

        Ok(Self {
            base: base_config,
            identity: identity_config,
        })
    }
}
