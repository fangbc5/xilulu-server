/// JWT Token 验证服务（sa-token 兼容 Claims 结构）
///
/// Claims 结构与 sa-token `JwtClaims` 完全兼容：
/// - `sub` — 用户 ID（login_id）
/// - `exp` / `iat` / `jti` — 标准 JWT 字段
/// - `extra` — 业务扩展字段（tenant_id, username, token_type 等）
///
/// **Token 生成**：由 ms-auth 使用 sa-token `JwtManager` 完成
/// **Token 验证**：所有微服务使用此模块解码（仅需 `jsonwebtoken` crate）
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// JWT Claims（与 sa-token JwtClaims 兼容）
///
/// 网关和所有微服务共用此结构解码 JWT。
/// ms-auth 通过 sa-token 的 `JwtClaims.extra` 注入 `tenant_id`、`username` 等。
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    /// 用户 ID（sa-token 的 login_id）
    #[serde(rename = "sub")]
    pub login_id: String,

    /// 过期时间（Unix 时间戳，秒）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exp: Option<i64>,

    /// 签发时间（Unix 时间戳，秒）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iat: Option<i64>,

    /// JWT ID（唯一标识符）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jti: Option<String>,

    /// 签发者
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iss: Option<String>,

    /// 受众
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aud: Option<String>,

    /// 登录类型
    #[serde(skip_serializing_if = "Option::is_none")]
    pub login_type: Option<String>,

    /// 扩展字段（tenant_id, username, token_type 等业务字段）
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub extra: HashMap<String, serde_json::Value>,
}

impl Claims {
    /// 获取用户 ID（i64）
    pub fn user_id(&self) -> Option<i64> {
        self.login_id.parse::<i64>().ok()
    }

    /// 获取租户 ID（从 extra 中读取）
    pub fn tenant_id(&self) -> Option<i64> {
        self.extra
            .get("tenant_id")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<i64>().ok())
    }

    /// 获取用户名（从 extra 中读取）
    pub fn username(&self) -> String {
        self.extra
            .get("username")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string()
    }

    /// 获取 token 类型（access / refresh，从 extra 中读取）
    pub fn token_type(&self) -> String {
        self.extra
            .get("token_type")
            .and_then(|v| v.as_str())
            .unwrap_or("access")
            .to_string()
    }
}

/// JWT 验证服务（仅解码，不生成 — 生成由 ms-auth 的 sa-token 负责）
pub struct JwtService {
    decoding_key: DecodingKey,
}

impl JwtService {
    /// 从环境变量 APP__JWT__SECRET 创建 JWT 验证服务
    pub fn from_env() -> Result<Self, String> {
        let secret = std::env::var("APP__JWT__SECRET")
            .map_err(|_| "环境变量 APP__JWT__SECRET 未设置".to_string())?;

        Ok(Self {
            decoding_key: DecodingKey::from_secret(secret.as_bytes()),
        })
    }

    /// 从密钥字符串创建 JWT 验证服务
    pub fn new(secret: &str) -> Self {
        Self {
            decoding_key: DecodingKey::from_secret(secret.as_bytes()),
        }
    }

    /// 验证并解析 Token
    pub fn verify_token(&self, token: &str) -> Result<Claims, String> {
        let token = token.trim();
        // 去除 Bearer 前缀
        let token = if token.starts_with("Bearer ") {
            &token[7..]
        } else {
            token
        };

        let mut validation = Validation::default();
        validation.leeway = 300; // 5 分钟容差

        let token_data = decode::<Claims>(token, &self.decoding_key, &validation)
            .map_err(|e| format!("JWT 验证失败: {}", e))?;

        Ok(token_data.claims)
    }
}
