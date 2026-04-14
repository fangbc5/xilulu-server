use deadpool_redis::redis::AsyncCommands;
use deadpool_redis::{Config, Pool, Runtime};

/// 初始化 Redis 连接池
///
/// 使用 deadpool-redis 创建连接池，支持并发访问
pub async fn init_redis(
    url: &str,
    password: Option<&str>,
    pool_size: usize,
) -> crate::error::AppResult<Pool> {
    // 构建最终的 Redis URL
    // 如果提供了单独的密码，且 URL 中不包含密码，则构建带密码的 URL
    let final_url = if let Some(pwd) = password {
        // 检查 URL 中是否已包含密码（通过 @ 符号判断）
        if !url.contains('@') {
            // URL 格式：redis://host:port，需要添加密码
            // 构建格式：redis://:password@host:port
            if let Some(prefix_end) = url.find("://") {
                let prefix = &url[..prefix_end + 3];
                let rest = &url[prefix_end + 3..];
                format!("{}:{}@{}", prefix, pwd, rest)
            } else {
                // 如果 URL 格式不正确，直接使用原 URL
                url.to_string()
            }
        } else {
            // URL 中已包含密码，直接使用（忽略单独的密码字段）
            tracing::warn!("Redis URL 中已包含密码，忽略单独的密码配置");
            url.to_string()
        }
    } else {
        url.to_string()
    };

    // 创建连接池配置
    let mut cfg = Config::from_url(final_url);
    if pool_size > 0 {
        cfg.pool = Some(deadpool_redis::PoolConfig {
            max_size: pool_size,
            ..Default::default()
        });
    }

    // 创建连接池
    let pool = cfg
        .create_pool(Some(Runtime::Tokio1))
        .map_err(|e| crate::error::AppError::Internal(anyhow::anyhow!(e)))?;

    // 测试连接
    let mut conn = pool.get().await?;
    // 使用与 deadpool_redis 绑定的 redis 版本，避免 trait 版本不匹配
    let _: String = conn.ping().await?;
    tracing::info!("Redis缓存初始化成功");
    Ok(pool)
}
