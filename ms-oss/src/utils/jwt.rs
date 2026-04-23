//! JWT 长效分享链接签发与验证
//!
//! 通过 JWT 突破 S3 预签名 7 天上限，实现任意有效期的文件分享。
//! JWT Payload 中封装 bucket/key/x_oss_process，URL 中不暴露存储路径。

use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

/// JWT Claims
#[derive(Debug, Serialize, Deserialize)]
pub struct ShareClaims {
    /// Bucket 名称
    pub bucket: String,
    /// 对象 Key
    pub key: String,
    /// 可选的处理参数（绑定后不可篡改）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub process: Option<String>,
    /// 过期时间（Unix 时间戳）
    pub exp: usize,
}

/// 签发分享 JWT
///
/// # Arguments
/// * `secret` - JWT 签名密钥
/// * `bucket` - Bucket 名称
/// * `key` - 对象 Key
/// * `process` - 可选的 x-oss-process 参数
/// * `expires_in_secs` - 有效期（秒）
pub fn create_share_token(
    secret: &str,
    bucket: &str,
    key: &str,
    process: Option<&str>,
    expires_in_secs: u64,
) -> anyhow::Result<String> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let claims = ShareClaims {
        bucket: bucket.to_string(),
        key: key.to_string(),
        process: process.map(|s| s.to_string()),
        exp: (now + expires_in_secs) as usize,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| anyhow::anyhow!("JWT 签发失败: {}", e))?;

    Ok(token)
}

/// 验证分享 JWT 并提取 Claims
///
/// # Arguments
/// * `secret` - JWT 签名密钥
/// * `token` - JWT 字符串
///
/// # Returns
/// 验证通过返回 Claims，失败返回错误
pub fn verify_share_token(secret: &str, token: &str) -> anyhow::Result<ShareClaims> {
    let token_data = decode::<ShareClaims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|e| anyhow::anyhow!("JWT 验证失败: {}", e))?;

    Ok(token_data.claims)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_verify() {
        let secret = "test-secret";
        let token = create_share_token(
            secret,
            "public",
            "avatar/test.jpg",
            Some("image/resize,m_fill,w_128,h_128"),
            3600,
        )
        .unwrap();

        let claims = verify_share_token(secret, &token).unwrap();
        assert_eq!(claims.bucket, "public");
        assert_eq!(claims.key, "avatar/test.jpg");
        assert_eq!(
            claims.process,
            Some("image/resize,m_fill,w_128,h_128".to_string())
        );
    }

    #[test]
    fn test_verify_expired() {
        let secret = "test-secret";
        // 签发一个已过期的 token（0 秒有效期）
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let claims = ShareClaims {
            bucket: "public".into(),
            key: "test.jpg".into(),
            process: None,
            exp: (now - 100) as usize, // 已过期
        };

        let token = jsonwebtoken::encode(
            &jsonwebtoken::Header::default(),
            &claims,
            &jsonwebtoken::EncodingKey::from_secret(secret.as_bytes()),
        )
        .unwrap();

        let result = verify_share_token(secret, &token);
        assert!(result.is_err());
    }
}
