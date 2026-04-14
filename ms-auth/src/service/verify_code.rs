// 验证码服务
// 用于生成、存储和验证验证码

use anyhow::Result;
use rand::Rng;

use crate::state::AppState;
use fbc_starter::cache::TokenService;

/// 验证码类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerifyCodeType {
    /// 手机号验证码
    Mobile,
    /// 邮箱验证码
    Email,
}

/// 验证码服务
pub struct VerifyCodeService;

impl VerifyCodeService {
    const CODE_PREFIX: &'static str = "verify_code:";
    const CODE_TTL: u64 = 300; // 5分钟过期
    const CODE_LENGTH: usize = 6; // 6位数字验证码
    const RATE_LIMIT_PREFIX: &'static str = "verify_code_rate_limit:";
    const RATE_LIMIT_TTL: u64 = 60; // 1分钟内只能发送一次

    /// 生成随机数字验证码
    fn generate_code() -> String {
        let mut rng = rand::thread_rng();
        (0..Self::CODE_LENGTH)
            .map(|_| rng.gen_range(0..10).to_string())
            .collect()
    }

    /// 检查是否超过频率限制
    /// 返回：None 表示未超过限制，Some(剩余秒数) 表示超过限制
    pub async fn check_rate_limit(app_state: &AppState, account: &str) -> Result<Option<u64>> {
        let key = format!("{}{}", Self::RATE_LIMIT_PREFIX, account);

        let mut redis_conn = app_state
            .fbc_app_state
            .redis()
            .await
            .map_err(|e| anyhow::anyhow!("Redis连接失败: {}", e))?;

        // 使用 TokenService 获取剩余过期时间（秒）
        let ttl = TokenService::get_token_ttl(&mut redis_conn, &key)
            .await
            .map_err(|e| anyhow::anyhow!("检查频率限制失败: {}", e))?;

        Ok(ttl)
    }

    /// 设置频率限制
    async fn set_rate_limit(app_state: &AppState, account: &str) -> Result<()> {
        let key = format!("{}{}", Self::RATE_LIMIT_PREFIX, account);

        let mut redis_conn = app_state
            .fbc_app_state
            .redis()
            .await
            .map_err(|e| anyhow::anyhow!("Redis连接失败: {}", e))?;

        TokenService::set_token(&mut redis_conn, &key, "1", Self::RATE_LIMIT_TTL)
            .await
            .map_err(|e| anyhow::anyhow!("设置频率限制失败: {}", e))?;

        Ok(())
    }

    /// 生成并存储验证码
    pub async fn generate_and_store(
        app_state: &AppState,
        account: &str,
        _code_type: VerifyCodeType,
    ) -> Result<String> {
        // 检查频率限制
        if let Some(remaining_seconds) = Self::check_rate_limit(app_state, account).await? {
            return Err(anyhow::anyhow!(
                "发送过于频繁，请于{}秒后再试",
                remaining_seconds
            ));
        }

        // 生成验证码
        let code = Self::generate_code();
        let key = format!("{}{}", Self::CODE_PREFIX, account);

        // 存储验证码到 Redis
        let mut redis_conn = app_state
            .fbc_app_state
            .redis()
            .await
            .map_err(|e| anyhow::anyhow!("Redis连接失败: {}", e))?;

        TokenService::set_token(&mut redis_conn, &key, &code, Self::CODE_TTL)
            .await
            .map_err(|e| anyhow::anyhow!("存储验证码失败: {}", e))?;

        // 设置频率限制
        Self::set_rate_limit(app_state, account).await?;

        Ok(code)
    }

    /// 验证验证码
    pub async fn verify(app_state: &AppState, account: &str, code: &str) -> Result<bool> {
        let key = format!("{}{}", Self::CODE_PREFIX, account);

        let mut redis_conn = app_state
            .fbc_app_state
            .redis()
            .await
            .map_err(|e| anyhow::anyhow!("Redis连接失败: {}", e))?;

        let stored_code: Option<String> = TokenService::get_token(&mut redis_conn, &key)
            .await
            .map_err(|e| anyhow::anyhow!("获取验证码失败: {}", e))?;

        match stored_code {
            Some(stored) if stored == code => {
                // 验证成功，删除验证码（一次性使用）
                TokenService::delete_token(&mut redis_conn, &key)
                    .await
                    .map_err(|e| anyhow::anyhow!("删除验证码失败: {}", e))?;
                Ok(true)
            }
            Some(_) => Ok(false), // 验证码不匹配
            None => Ok(false),    // 验证码不存在或已过期
        }
    }
}
