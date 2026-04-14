// 图片验证码服务
// 用于生成图片验证码并存储到 Redis，默认 5 分钟有效

use anyhow::Result;
use captcha::{filters::Noise, Captcha};
use deadpool_redis::redis::AsyncCommands;
use uuid::Uuid;

use crate::state::AppState;

/// 图片验证码服务
pub struct ImageCaptchaService;

impl ImageCaptchaService {
    const CAPTCHA_PREFIX: &'static str = "image_captcha:";
    const CAPTCHA_TTL: u64 = 300; // 5 分钟
    const RATE_LIMIT_PREFIX: &'static str = "image_captcha_rate_limit:";
    const RATE_LIMIT_TTL: u64 = 1; // 1秒内只能请求一次

    /// 检查是否超过频率限制
    /// 返回：None 表示未超过限制，Some(剩余秒数) 表示超过限制
    async fn check_rate_limit(app_state: &AppState, ip: &str) -> Result<Option<u64>> {
        let key = format!("{}{}", Self::RATE_LIMIT_PREFIX, ip);

        let mut redis_conn = app_state
            .fbc_app_state
            .redis()
            .await
            .map_err(|e| anyhow::anyhow!("Redis连接失败: {}", e))?;

        // 使用 TTL 获取剩余过期时间（秒）
        let ttl: i64 = redis_conn
            .ttl(&key)
            .await
            .map_err(|e| anyhow::anyhow!("检查频率限制失败: {}", e))?;

        if ttl > 0 {
            // key 存在且未过期，返回剩余秒数
            Ok(Some(ttl as u64))
        } else {
            // key 不存在或已过期，未超过限制
            Ok(None)
        }
    }

    /// 设置频率限制
    async fn set_rate_limit(app_state: &AppState, ip: &str) -> Result<()> {
        let key = format!("{}{}", Self::RATE_LIMIT_PREFIX, ip);

        let mut redis_conn = app_state
            .fbc_app_state
            .redis()
            .await
            .map_err(|e| anyhow::anyhow!("Redis连接失败: {}", e))?;

        redis_conn
            .set_ex::<_, _, ()>(&key, "1", Self::RATE_LIMIT_TTL)
            .await
            .map_err(|e| anyhow::anyhow!("设置频率限制失败: {}", e))?;

        Ok(())
    }

    /// 使用 captcha crate 生成图片验证码并存储，返回 (captcha_id, image_base64_png)
    ///
    /// # 参数
    /// - `app_state`: 应用状态
    /// - `client_ip`: 客户端 IP 地址，用于频率限制
    pub async fn generate(app_state: &AppState, client_ip: &str) -> Result<(String, String)> {
        // 0. 检查频率限制
        if let Some(remaining_seconds) = Self::check_rate_limit(app_state, client_ip).await? {
            return Err(anyhow::anyhow!(
                "请求过于频繁，请于{}秒后再试",
                remaining_seconds
            ));
        }
        // 1. 生成图片验证码（PNG）和对应文本
        // 为了保证 async Future 是 Send，把可能包含 !Send 的 Captcha
        // 限制在一个内部作用域里，避免跨越 .await。
        let (code, image_base64) = {
            let mut captcha = Captcha::new();
            captcha.add_chars(5);
            captcha.apply_filter(Noise::new(0.1));
            captcha.view(160, 60);

            let code = captcha.chars_as_string().to_uppercase();
            let image_base64 = captcha
                .as_base64()
                .ok_or_else(|| anyhow::anyhow!("生成图片验证码失败: as_base64 返回 None"))?;

            (code, image_base64)
        };

        // 2. 生成 captcha_id 并存储到 Redis，5 分钟有效
        let captcha_id = Uuid::new_v4().to_string();
        let key = format!("{}{}", Self::CAPTCHA_PREFIX, captcha_id);

        let mut redis_conn = app_state
            .fbc_app_state
            .redis()
            .await
            .map_err(|e| anyhow::anyhow!("Redis连接失败: {}", e))?;

        redis_conn
            .set_ex::<_, _, ()>(&key, &code, Self::CAPTCHA_TTL)
            .await
            .map_err(|e| anyhow::anyhow!("存储图片验证码失败: {}", e))?;

        // 3. 设置频率限制
        Self::set_rate_limit(app_state, client_ip).await?;

        Ok((captcha_id, image_base64))
    }

    /// 校验图片验证码
    /// 成功后会删除 Redis 中的验证码（一次性使用）
    pub async fn verify(app_state: &AppState, captcha_id: &str, captcha: &str) -> Result<bool> {
        let key = format!("{}{}", Self::CAPTCHA_PREFIX, captcha_id);

        let mut redis_conn = app_state
            .fbc_app_state
            .redis()
            .await
            .map_err(|e| anyhow::anyhow!("Redis连接失败: {}", e))?;

        let stored_code: Option<String> = redis_conn
            .get(&key)
            .await
            .map_err(|e| anyhow::anyhow!("获取图片验证码失败: {}", e))?;

        match stored_code {
            Some(stored) if stored.eq_ignore_ascii_case(captcha) => {
                // 验证成功，删除验证码
                redis_conn
                    .del::<_, ()>(&key)
                    .await
                    .map_err(|e| anyhow::anyhow!("删除图片验证码失败: {}", e))?;
                Ok(true)
            }
            Some(_) => Ok(false),
            None => Ok(false),
        }
    }
}
