//! imgproxy HMAC-SHA256 签名 URL 生成器
//!
//! imgproxy 要求所有请求 URL 都经过 HMAC-SHA256 签名，
//! 防止外部用户直接构造 URL 盗用计算资源。
//!
//! 签名算法：
//! 1. 将 hex 格式的 key/salt 解码为字节
//! 2. path = "/{processing}/{source}" (不含 base_url)
//! 3. digest = HMAC-SHA256(key, salt + path_bytes)
//! 4. signature = base64url_nopad(digest)
//! 5. 最终 URL = base_url + "/" + signature + path

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use hmac::{Hmac, Mac};
use sha2::Sha256;

use crate::config::ImgproxyConfig;

type HmacSha256 = Hmac<Sha256>;

/// 对 imgproxy 路径进行 HMAC-SHA256 签名
///
/// # Arguments
/// * `key_hex` - hex 编码的 HMAC key
/// * `salt_hex` - hex 编码的 HMAC salt
/// * `path` - 待签名的路径（以 / 开头，如 `/rs:fill:128:128/plain/s3://bucket/key@webp`）
///
/// # Returns
/// base64url（无 padding）编码的签名字符串
pub fn sign_path(key_hex: &str, salt_hex: &str, path: &str) -> anyhow::Result<String> {
    let key_bytes = hex::decode(key_hex)
        .map_err(|e| anyhow::anyhow!("imgproxy key hex 解码失败: {}", e))?;
    let salt_bytes = hex::decode(salt_hex)
        .map_err(|e| anyhow::anyhow!("imgproxy salt hex 解码失败: {}", e))?;

    let mut mac = HmacSha256::new_from_slice(&key_bytes)
        .map_err(|e| anyhow::anyhow!("HMAC 初始化失败: {}", e))?;

    // imgproxy 签名算法：HMAC(key, salt + path_bytes)
    mac.update(&salt_bytes);
    mac.update(path.as_bytes());

    let result = mac.finalize();
    let digest = result.into_bytes();

    Ok(URL_SAFE_NO_PAD.encode(digest))
}

/// 构建完整的 imgproxy 签名 URL
///
/// # Arguments
/// * `config` - imgproxy 配置
/// * `processing` - 处理指令（如 `rs:fill:128:128/q:85`）
/// * `bucket` - S3 bucket 名
/// * `key` - 对象 key
/// * `format` - 输出格式后缀（如 `webp`），为空则不追加
///
/// # Returns
/// 完整的带签名的 imgproxy URL
pub fn build_url(
    config: &ImgproxyConfig,
    processing: &str,
    bucket: &str,
    key: &str,
    format: Option<&str>,
) -> anyhow::Result<String> {
    // 构建 source URL
    let source = format!("s3://{}/{}", bucket, key);

    // 构建 path：/{processing}/plain/{source}[@format]
    // 当 processing 为空时（纯格式转换），使用 imgproxy 的 raw:1 透传原图
    let effective_processing = if processing.is_empty() { "raw:1" } else { processing };
    let path = if let Some(fmt) = format {
        format!("/{}/plain/{}@{}", effective_processing, source, fmt)
    } else {
        format!("/{}/plain/{}", effective_processing, source)
    };

    if config.is_enabled() {
        // 有签名配置：生成带签名的 URL
        let signature = sign_path(&config.key, &config.salt, &path)?;
        Ok(format!("{}/{}{}", config.base_url, signature, path))
    } else {
        // 无签名配置（开发模式）：使用 insecure 前缀
        Ok(format!("{}/insecure{}", config.base_url, path))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_path() {
        // 使用 docker-compose-imgproxy.yml 中的 key/salt
        let key = "7a8f3b14e9c60d852a41f69c7b3e10d8f5c246b9a81f3d7e5b4c90a2c58efb13";
        let salt = "3b9d6c2a8f4e15079a408bec1b305d2e7f849c31b26a5cd9e10fa78c6b4e3902";
        let path = "/rs:fill:128:128/plain/s3://public/avatar/test.jpg@webp";

        let result = sign_path(key, salt, path);
        assert!(result.is_ok());
        let sig = result.unwrap();
        assert!(!sig.is_empty());
        // 签名应该是 base64url 格式
        assert!(!sig.contains('+'));
        assert!(!sig.contains('/'));
    }

    #[test]
    fn test_build_url() {
        let config = ImgproxyConfig {
            key: "7a8f3b14e9c60d852a41f69c7b3e10d8f5c246b9a81f3d7e5b4c90a2c58efb13".into(),
            salt: "3b9d6c2a8f4e15079a408bec1b305d2e7f849c31b26a5cd9e10fa78c6b4e3902".into(),
            base_url: "http://localhost:8085".into(),
        };

        let url = build_url(&config, "rs:fill:128:128", "public", "avatar/test.jpg", Some("webp"));
        assert!(url.is_ok());
        let url = url.unwrap();
        assert!(url.starts_with("http://localhost:8085/"));
        assert!(url.contains("/rs:fill:128:128/plain/s3://public/avatar/test.jpg@webp"));
    }

    #[test]
    fn test_build_url_insecure() {
        let config = ImgproxyConfig::default(); // key/salt 为空

        let url = build_url(&config, "rs:fill:64:64", "public", "avatar/test.jpg", None);
        assert!(url.is_ok());
        let url = url.unwrap();
        assert!(url.contains("/insecure/"));
    }
}
