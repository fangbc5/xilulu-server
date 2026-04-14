// Token相关操作

use crate::error::{AppError, AppResult};
use deadpool_redis::redis::AsyncCommands;

/// Token服务
pub struct TokenService;

impl TokenService {
    /// 存储token到Redis
    ///
    /// # 参数
    /// - `redis_conn`: Redis连接
    /// - `key`: token的key
    /// - `value`: token的值
    /// - `expire_seconds`: 过期时间（秒）
    pub async fn set_token(
        redis_conn: &mut deadpool_redis::Connection,
        key: &str,
        value: &str,
        expire_seconds: u64,
    ) -> AppResult<()> {
        let _: () = redis_conn
            .set_ex(key, value, expire_seconds)
            .await
            .map_err(AppError::from)?;
        Ok(())
    }

    /// 获取token从Redis
    ///
    /// # 参数
    /// - `redis_conn`: Redis连接
    /// - `key`: token的key
    ///
    /// # 返回
    /// - `Option<String>`: token的值，如果不存在则返回None
    pub async fn get_token(
        redis_conn: &mut deadpool_redis::Connection,
        key: &str,
    ) -> AppResult<Option<String>> {
        let value: Option<String> = redis_conn.get(key).await.map_err(AppError::from)?;
        Ok(value)
    }

    /// 删除token从Redis
    ///
    /// # 参数
    /// - `redis_conn`: Redis连接
    /// - `key`: token的key
    pub async fn delete_token(
        redis_conn: &mut deadpool_redis::Connection,
        key: &str,
    ) -> AppResult<()> {
        let _: () = redis_conn.del(key).await.map_err(AppError::from)?;
        Ok(())
    }

    /// 检查token是否存在
    ///
    /// # 参数
    /// - `redis_conn`: Redis连接
    /// - `key`: token的key
    ///
    /// # 返回
    /// - `bool`: token是否存在
    pub async fn exists_token(
        redis_conn: &mut deadpool_redis::Connection,
        key: &str,
    ) -> AppResult<bool> {
        let exists: bool = redis_conn.exists(key).await.map_err(AppError::from)?;
        Ok(exists)
    }

    /// 获取token剩余过期时间（秒）
    ///
    /// # 参数
    /// - `redis_conn`: Redis连接
    /// - `key`: token的key
    ///
    /// # 返回
    /// - `Ok(Some(ttl))`: 剩余过期时间（秒）
    /// - `Ok(None)`: key 不存在或不过期
    pub async fn get_token_ttl(
        redis_conn: &mut deadpool_redis::Connection,
        key: &str,
    ) -> AppResult<Option<u64>> {
        let ttl: i64 = redis_conn.ttl(key).await.map_err(AppError::from)?;
        if ttl < 0 {
            // -2: key不存在；-1: key存在但无过期时间
            Ok(None)
        } else {
            Ok(Some(ttl as u64))
        }
    }
}
