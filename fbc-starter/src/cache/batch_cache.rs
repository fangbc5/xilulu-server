/// 批量缓存操作 Trait
///
/// 定义批量缓存操作的通用接口，支持单个和批量获取、删除操作
///
/// 使用关联类型 `Key` 和 `Value` 来定义缓存键和值的类型。
///
/// # 示例
/// ```rust,no_run
/// use fbc_starter::cache::BatchCache;
/// use fbc_starter::AppResult;
/// use std::collections::HashMap;
///
/// struct MyCache;
///
/// #[async_trait::async_trait]
/// impl BatchCache for MyCache {
///     type Key = i64;
///     type Value = String;
///
///     async fn get(&self, req: Self::Key) -> AppResult<Option<Self::Value>> {
///         // 实现单个获取逻辑
///         Ok(None)
///     }
///
///     async fn get_batch(&self, req: &[Self::Key]) -> AppResult<HashMap<Self::Key, Self::Value>> {
///         // 实现批量获取逻辑
///         Ok(HashMap::new())
///     }
///
///     async fn delete(&self, req: Self::Key) -> AppResult<()> {
///         // 实现单个删除逻辑
///         Ok(())
///     }
///
///     async fn delete_batch(&self, req: &[Self::Key]) -> AppResult<()> {
///         // 实现批量删除逻辑
///         Ok(())
///     }
/// }
/// ```
use async_trait::async_trait;
use std::collections::HashMap;
use std::hash::Hash;

#[cfg(feature = "redis")]
use deadpool_redis::redis::{self, AsyncCommands};
#[cfg(feature = "redis")]
use serde::{Deserialize, Serialize};

use crate::error::AppResult;

/// 批量缓存操作 Trait
///
/// 提供单个和批量缓存操作的统一接口
///
/// # 关联类型
/// - `Key`: 缓存键类型，需要实现 `Send + Sync + Clone + Hash + Eq`
/// - `Value`: 缓存值类型，需要实现 `Send + Sync + Clone` 和序列化（当使用 Redis 默认实现时）
/// - `RedisClient`: （可选）Redis 客户端类型，用于提供基于 Redis 的默认实现
#[async_trait]
pub trait BatchCache {
    /// 缓存键类型
    type Key: Send + Sync + Clone + Hash + Eq;
    /// 缓存值类型
    type Value: Send + Sync + Clone;

    /// 获取单个缓存
    ///
    /// # 参数
    /// - `req`: 缓存键
    ///
    /// # 返回
    /// - `Ok(Some(value))`: 找到缓存值
    /// - `Ok(None)`: 缓存不存在
    /// - `Err(e)`: 操作失败
    async fn get(&self, req: Self::Key) -> AppResult<Option<Self::Value>>;

    /// 批量获取缓存
    ///
    /// # 参数
    /// - `req`: 缓存键列表
    ///
    /// # 返回
    /// - `Ok(map)`: 返回键值对映射，只包含存在的缓存项
    /// - `Err(e)`: 操作失败
    async fn get_batch(&self, req: &[Self::Key]) -> AppResult<HashMap<Self::Key, Self::Value>>;

    /// 删除单个缓存
    ///
    /// # 参数
    /// - `req`: 缓存键
    ///
    /// # 返回
    /// - `Ok(())`: 删除成功（无论缓存是否存在）
    /// - `Err(e)`: 操作失败
    async fn delete(&self, req: Self::Key) -> AppResult<()>;

    /// 批量删除缓存
    ///
    /// # 参数
    /// - `req`: 缓存键列表
    ///
    /// # 返回
    /// - `Ok(())`: 删除成功（无论缓存是否存在）
    /// - `Err(e)`: 操作失败
    async fn delete_batch(&self, req: &[Self::Key]) -> AppResult<()>;

    /// 批量刷新缓存（清除指定 map 中所有键对应的缓存）
    ///
    /// # 参数
    /// - `req`: 包含要清除的缓存键的 HashMap（值会被忽略，只使用键）
    ///
    /// # 返回
    /// - `Ok(())`: 清除成功（无论缓存是否存在）
    /// - `Err(e)`: 操作失败
    ///
    /// # 示例
    /// ```rust,no_run
    /// use fbc_starter::cache::BatchCache;
    /// use fbc_starter::AppResult;
    /// use std::collections::HashMap;
    ///
    /// # async fn example<T: BatchCache<Key = i64, Value = String>>(cache: &T) -> AppResult<()> {
    /// let mut map = HashMap::new();
    /// map.insert(1, "value1".to_string());
    /// map.insert(2, "value2".to_string());
    /// map.insert(3, "value3".to_string());
    ///
    /// // 清除 map 中所有键对应的缓存
    /// cache.refresh(&map).await?;
    /// # Ok(())
    /// # }
    /// ```
    async fn refresh(&self, req: &HashMap<Self::Key, Self::Value>) -> AppResult<()> {
        // 默认实现：提取所有键并批量删除
        let keys: Vec<Self::Key> = req.keys().cloned().collect();
        self.delete_batch(&keys).await
    }
}

/// 基于 Redis 的批量缓存操作扩展 Trait
///
/// 为实现了 `BatchCache` 且提供了 Redis 客户端的类型提供基于 Redis 的默认实现。
/// 实现此 trait 后，可以使用 `get_from_redis`、`set_to_redis` 等辅助方法。
///
/// # 使用示例
/// ```rust,no_run
/// use fbc_starter::cache::{BatchCache, RedisBatchCache};
/// use std::sync::Arc;
/// use deadpool_redis::Pool;
///
/// struct MyRedisCache {
///     redis_pool: Arc<Pool>,
/// }
///
/// #[async_trait::async_trait]
/// impl BatchCache for MyRedisCache {
///     type Key = i64;
///     type Value = String;
///     // ... 实现其他必需的方法
/// }
///
/// #[async_trait::async_trait]
/// impl RedisBatchCache for MyRedisCache {
///     async fn get_redis_connection(&self) -> AppResult<deadpool_redis::Connection> {
///         use fbc_starter::error::AppError;
///         Ok(self.redis_pool.get().await.map_err(|e| AppError::Internal(anyhow::anyhow!("获取 Redis 连接失败: {}", e)))?)
///     }
///     
///     fn build_cache_key(&self, key: &Self::Key) -> String {
///         format!("cache:{}", key)
///     }
///     
///     fn cache_expire(&self) -> Option<u64> {
///         Some(300) // 5 分钟
///     }
/// }
/// ```
#[cfg(feature = "redis")]
#[async_trait]
pub trait RedisBatchCache: BatchCache
where
    Self::Value: Serialize + for<'de> Deserialize<'de>,
{
    /// 获取 Redis 连接
    async fn get_redis_connection(&self) -> AppResult<deadpool_redis::Connection>;

    /// 构建缓存键
    ///
    /// 将 `Key` 类型转换为字符串键
    fn build_cache_key(&self, key: &Self::Key) -> String;

    /// 获取缓存过期时间（秒）
    ///
    /// 返回 `Some(seconds)` 如果设置了过期时间，否则返回 `None`（永不过期）
    fn cache_expire(&self) -> Option<u64> {
        None
    }

    /// 基于 Redis 的默认实现：获取单个缓存
    async fn get_from_redis(&self, req: Self::Key) -> AppResult<Option<Self::Value>> {
        use crate::error::AppError;
        let key = self.build_cache_key(&req);
        let mut conn = self.get_redis_connection().await?;

        // 尝试从 Redis 获取
        let value: Option<String> = conn.get(&key).await.map_err(AppError::Redis)?;
        if let Some(json_str) = value {
            let value: Self::Value = serde_json::from_str(&json_str)
                .map_err(|e| AppError::Internal(anyhow::anyhow!("JSON 反序列化失败: {}", e)))?;
            return Ok(Some(value));
        }

        Ok(None)
    }

    /// 基于 Redis 的默认实现：批量获取缓存
    async fn get_batch_from_redis(
        &self,
        req: &[Self::Key],
    ) -> AppResult<HashMap<Self::Key, Self::Value>> {
        use crate::error::AppError;
        if req.is_empty() {
            return Ok(HashMap::new());
        }

        let mut conn = self.get_redis_connection().await?;
        let mut result = HashMap::new();

        // 构建所有键
        let keys: Vec<String> = req.iter().map(|k| self.build_cache_key(k)).collect();

        // 批量从 Redis 获取
        let values: Vec<Option<String>> = conn.mget(&keys).await.map_err(AppError::Redis)?;

        for (key, value_opt) in req.iter().zip(values.iter()) {
            if let Some(json_str) = value_opt {
                if let Ok(value) = serde_json::from_str::<Self::Value>(json_str) {
                    result.insert(key.clone(), value);
                }
            }
        }

        Ok(result)
    }

    /// 基于 Redis 的默认实现：删除单个缓存
    async fn delete_from_redis(&self, req: Self::Key) -> AppResult<()> {
        use crate::error::AppError;
        let key = self.build_cache_key(&req);
        let mut conn = self.get_redis_connection().await?;
        let _: () = conn.del(&key).await.map_err(AppError::Redis)?;
        Ok(())
    }

    /// 基于 Redis 的默认实现：批量删除缓存
    async fn delete_batch_from_redis(&self, req: &[Self::Key]) -> AppResult<()> {
        use crate::error::AppError;
        if req.is_empty() {
            return Ok(());
        }

        let mut conn = self.get_redis_connection().await?;
        let keys: Vec<String> = req.iter().map(|k| self.build_cache_key(k)).collect();

        if !keys.is_empty() {
            let _: () = conn.del(&keys).await.map_err(AppError::Redis)?;
        }

        Ok(())
    }

    /// 写入缓存到 Redis
    ///
    /// 注意：对于批量写入，建议使用 `set_batch_to_redis` 方法以获得更好的性能
    async fn set_to_redis(&self, key: Self::Key, value: &Self::Value) -> AppResult<()> {
        use crate::error::AppError;
        let cache_key = self.build_cache_key(&key);
        let json_str = serde_json::to_string(value)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("JSON 序列化失败: {}", e)))?;
        let mut conn = self.get_redis_connection().await?;

        if let Some(expire) = self.cache_expire() {
            let _: () = conn
                .set_ex(&cache_key, &json_str, expire)
                .await
                .map_err(AppError::Redis)?;
        } else {
            let _: () = conn
                .set(&cache_key, &json_str)
                .await
                .map_err(AppError::Redis)?;
        }

        Ok(())
    }

    /// 批量写入缓存到 Redis（性能更高）
    ///
    /// 使用 Redis 的 pipeline 或 MSET 命令进行批量写入，性能比单个写入高很多
    ///
    /// # 参数
    /// - `items`: 键值对映射
    ///
    /// # 返回
    /// - `Ok(())`: 写入成功
    /// - `Err(e)`: 操作失败
    ///
    /// # 示例
    /// ```rust,no_run
    /// use fbc_starter::cache::RedisBatchCache;
    /// use fbc_starter::AppResult;
    /// use std::collections::HashMap;
    ///
    /// # async fn example<T: RedisBatchCache<Key = i64, Value = String>>(cache: &T) -> AppResult<()> {
    /// let mut items = HashMap::new();
    /// items.insert(1, "value1".to_string());
    /// items.insert(2, "value2".to_string());
    /// items.insert(3, "value3".to_string());
    ///
    /// // 批量写入缓存
    /// cache.set_batch_to_redis(&items).await?;
    /// # Ok(())
    /// # }
    /// ```
    async fn set_batch_to_redis(&self, items: &HashMap<Self::Key, Self::Value>) -> AppResult<()> {
        use crate::error::AppError;
        if items.is_empty() {
            return Ok(());
        }

        let mut conn = self.get_redis_connection().await?;
        let expire = self.cache_expire();

        // 使用 pipeline 批量执行命令，性能更高
        let mut pipe = redis::pipe();
        pipe.atomic();

        for (key, value) in items {
            let cache_key = self.build_cache_key(key);
            let json_str = serde_json::to_string(value)
                .map_err(|e| AppError::Internal(anyhow::anyhow!("JSON 序列化失败: {}", e)))?;

            if let Some(expire_secs) = expire {
                pipe.set_ex(&cache_key, &json_str, expire_secs);
            } else {
                pipe.set(&cache_key, &json_str);
            }
        }

        pipe.query_async::<()>(&mut conn)
            .await
            .map_err(AppError::Redis)?;
        Ok(())
    }
}

/// 基于本地内存缓存的批量缓存操作扩展 Trait（类似 Caffeine）
///
/// 为实现了 `BatchCache` 且提供了本地缓存客户端的类型提供基于本地内存缓存的默认实现。
/// 使用 `moka` 库作为底层实现，提供高性能的本地缓存功能。
///
/// # 使用示例
/// ```rust,no_run
/// use fbc_starter::cache::{BatchCache, LocalBatchCache};
/// use fbc_starter::AppResult;
/// use std::sync::Arc;
/// use std::collections::HashMap;
/// use moka::future::Cache;
/// use std::time::Duration;
///
/// struct MyLocalCache {
///     cache: Arc<Cache<String, String>>,
/// }
///
/// impl MyLocalCache {
///     fn new() -> Self {
///         // 创建缓存，设置最大容量和过期时间
///         let cache = Cache::builder()
///             .max_capacity(10_000)
///             .time_to_live(Duration::from_secs(300)) // 5 分钟过期
///             .build();
///         
///         Self {
///             cache: Arc::new(cache),
///         }
///     }
/// }
///
/// #[async_trait::async_trait]
/// impl BatchCache for MyLocalCache {
///     type Key = i64;
///     type Value = String;
///
///     async fn get(&self, req: Self::Key) -> AppResult<Option<Self::Value>> {
///         // 使用本地缓存的默认实现
///         self.get_from_local(req).await
///     }
///
///     async fn get_batch(&self, req: &[Self::Key]) -> AppResult<HashMap<Self::Key, Self::Value>> {
///         // 使用本地缓存的默认实现
///         self.get_batch_from_local(req).await
///     }
///
///     async fn delete(&self, req: Self::Key) -> AppResult<()> {
///         // 使用本地缓存的默认实现
///         self.delete_from_local(req).await
///     }
///
///     async fn delete_batch(&self, req: &[Self::Key]) -> AppResult<()> {
///         // 使用本地缓存的默认实现
///         self.delete_batch_from_local(req).await
///     }
/// }
///
/// #[async_trait::async_trait]
/// impl LocalBatchCache for MyLocalCache {
///     fn get_local_cache(&self) -> Arc<Cache<String, Self::Value>> {
///         self.cache.clone()
///     }
///     
///     fn build_cache_key(&self, key: &Self::Key) -> String {
///         format!("cache:{}", key)
///     }
/// }
/// ```
#[cfg(feature = "local_cache")]
#[async_trait]
pub trait LocalBatchCache: BatchCache
where
    Self::Value: Send + Sync + Clone + 'static,
    Self::Key: Send + Sync + Clone + Hash + Eq + 'static,
{
    /// 获取本地缓存实例
    ///
    /// 返回 `moka::future::Cache` 的 `Arc` 包装，用于实际的缓存操作
    fn get_local_cache(&self) -> std::sync::Arc<moka::future::Cache<String, Self::Value>>;

    /// 构建缓存键
    ///
    /// 将 `Key` 类型转换为字符串键
    fn build_cache_key(&self, key: &Self::Key) -> String;

    /// 基于本地缓存的默认实现：获取单个缓存
    async fn get_from_local(&self, req: Self::Key) -> AppResult<Option<Self::Value>> {
        let cache = self.get_local_cache();
        let key = self.build_cache_key(&req);
        let value = cache.get(&key).await;
        Ok(value)
    }

    /// 基于本地缓存的默认实现：批量获取缓存
    async fn get_batch_from_local(
        &self,
        req: &[Self::Key],
    ) -> AppResult<HashMap<Self::Key, Self::Value>> {
        if req.is_empty() {
            return Ok(HashMap::new());
        }

        let cache = self.get_local_cache();
        let mut result = HashMap::new();

        // 批量从本地缓存获取
        for key in req {
            let cache_key = self.build_cache_key(key);
            if let Some(value) = cache.get(&cache_key).await {
                result.insert(key.clone(), value);
            }
        }

        Ok(result)
    }

    /// 基于本地缓存的默认实现：删除单个缓存
    async fn delete_from_local(&self, req: Self::Key) -> AppResult<()> {
        let cache = self.get_local_cache();
        let key = self.build_cache_key(&req);
        cache.invalidate(&key).await;
        Ok(())
    }

    /// 基于本地缓存的默认实现：批量删除缓存
    async fn delete_batch_from_local(&self, req: &[Self::Key]) -> AppResult<()> {
        if req.is_empty() {
            return Ok(());
        }

        let cache = self.get_local_cache();
        let keys: Vec<String> = req.iter().map(|k| self.build_cache_key(k)).collect();

        // 批量失效缓存
        for key in keys {
            cache.invalidate(&key).await;
        }

        Ok(())
    }

    /// 写入缓存到本地内存
    ///
    /// 注意：对于批量写入，建议使用 `set_batch_to_local` 方法以获得更好的性能
    async fn set_to_local(&self, key: Self::Key, value: &Self::Value) -> AppResult<()> {
        let cache = self.get_local_cache();
        let cache_key = self.build_cache_key(&key);
        cache.insert(cache_key, value.clone()).await;
        Ok(())
    }

    /// 批量写入缓存到本地内存（性能更高）
    ///
    /// 批量插入缓存项，性能比单个插入高
    ///
    /// # 参数
    /// - `items`: 键值对映射
    ///
    /// # 返回
    /// - `Ok(())`: 写入成功
    /// - `Err(e)`: 操作失败
    ///
    /// # 示例
    /// ```rust,no_run
    /// use fbc_starter::cache::LocalBatchCache;
    /// use fbc_starter::AppResult;
    /// use std::collections::HashMap;
    ///
    /// # async fn example<T: LocalBatchCache<Key = i64, Value = String>>(cache: &T) -> AppResult<()> {
    /// let mut items = HashMap::new();
    /// items.insert(1, "value1".to_string());
    /// items.insert(2, "value2".to_string());
    /// items.insert(3, "value3".to_string());
    ///
    /// // 批量写入缓存
    /// cache.set_batch_to_local(&items).await?;
    /// # Ok(())
    /// # }
    /// ```
    async fn set_batch_to_local(&self, items: &HashMap<Self::Key, Self::Value>) -> AppResult<()> {
        if items.is_empty() {
            return Ok(());
        }

        let cache = self.get_local_cache();

        // 批量插入缓存
        for (key, value) in items {
            let cache_key = self.build_cache_key(key);
            cache.insert(cache_key, value.clone()).await;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    /// 测试用的简单缓存实现
    struct TestCache {
        data: HashMap<i64, String>,
    }

    #[async_trait]
    impl BatchCache for TestCache {
        type Key = i64;
        type Value = String;

        async fn get(&self, req: Self::Key) -> AppResult<Option<Self::Value>> {
            Ok(self.data.get(&req).cloned())
        }

        async fn get_batch(&self, req: &[Self::Key]) -> AppResult<HashMap<Self::Key, Self::Value>> {
            let mut result = HashMap::new();
            for key in req {
                if let Some(value) = self.data.get(key) {
                    result.insert(*key, value.clone());
                }
            }
            Ok(result)
        }

        async fn delete(&self, _req: Self::Key) -> AppResult<()> {
            // 测试实现，不做实际删除
            Ok(())
        }

        async fn delete_batch(&self, _req: &[Self::Key]) -> AppResult<()> {
            // 测试实现，不做实际删除
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_get() {
        let mut data = HashMap::new();
        data.insert(1, "value1".to_string());
        data.insert(2, "value2".to_string());

        let cache = TestCache { data };

        let result = cache.get(1).await.unwrap();
        assert_eq!(result, Some("value1".to_string()));

        let result = cache.get(3).await.unwrap();
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn test_get_batch() {
        let mut data = HashMap::new();
        data.insert(1, "value1".to_string());
        data.insert(2, "value2".to_string());
        data.insert(3, "value3".to_string());

        let cache = TestCache { data };

        let result = cache.get_batch(&[1, 2, 4]).await.unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result.get(&1), Some(&"value1".to_string()));
        assert_eq!(result.get(&2), Some(&"value2".to_string()));
        assert_eq!(result.get(&4), None);
    }

    #[tokio::test]
    async fn test_delete() {
        let data = HashMap::new();
        let cache = TestCache { data };

        let result = cache.delete(1).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_delete_batch() {
        let data = HashMap::new();
        let cache = TestCache { data };

        let result = cache.delete_batch(&[1, 2, 3]).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_refresh() {
        let mut data = HashMap::new();
        data.insert(1, "value1".to_string());
        data.insert(2, "value2".to_string());
        data.insert(3, "value3".to_string());

        let cache = TestCache { data };

        let mut refresh_map = HashMap::new();
        refresh_map.insert(1, "ignored".to_string());
        refresh_map.insert(2, "ignored".to_string());

        let result = cache.refresh(&refresh_map).await;
        assert!(result.is_ok());
    }
}
