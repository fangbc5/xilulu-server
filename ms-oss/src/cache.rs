//! 文件元数据 Redis 缓存层
//!
//! 使用 `fbc-starter` 框架的 `SimpleCacheKeyBuilder` 构建 key，
//! `deadpool-redis` 异步连接池执行读写。
//!
//! Redis 仅作为加速层，所有操作 graceful 降级：
//! 失败只打日志，不阻断主流程。MySQL 始终是 source of truth。

use deadpool_redis::Pool;
use deadpool_redis::redis::AsyncCommands;
use fbc_starter::cache::{CacheKeyBuilder, SimpleCacheKeyBuilder, FILE};
use std::time::Duration;

use crate::modules::file::model::entity::FileMeta;

/// 缓存 TTL：10 分钟
const CACHE_TTL_SECS: u64 = 600;

/// 文件元数据缓存
///
/// 封装 Redis 读写操作，按 `(bucket, file_key)` 作为唯一键。
/// key 格式：`file:file.meta:obj:{bucket}:{file_key}`
#[derive(Clone)]
pub struct FileMetaCache {
    pool: Option<Pool>,
}

impl FileMetaCache {
    /// 创建缓存实例
    ///
    /// `pool` 为 None 时所有操作静默跳过（未配置 Redis 的场景）
    pub fn new(pool: Option<Pool>) -> Self {
        Self { pool }
    }

    /// 构建缓存 key
    fn build_key(bucket: &str, key: &str) -> String {
        let builder = SimpleCacheKeyBuilder::new("file.meta")
            .with_modular(FILE)
            .with_expire(Duration::from_secs(CACHE_TTL_SECS));
        let cache_key = builder.key(&[&bucket, &key]);
        cache_key.key
    }

    /// 从缓存获取文件元数据
    ///
    /// 返回 `None` 表示缓存未命中或 Redis 不可用
    pub async fn get(&self, bucket: &str, key: &str) -> Option<FileMeta> {
        let pool = self.pool.as_ref()?;
        let cache_key = Self::build_key(bucket, key);

        match pool.get().await {
            Ok(mut conn) => {
                let result: Result<Option<String>, _> = conn.get(&cache_key).await;
                match result {
                    Ok(Some(json)) => match serde_json::from_str::<FileMeta>(&json) {
                        Ok(meta) => {
                            tracing::debug!("缓存命中: {}", cache_key);
                            Some(meta)
                        }
                        Err(e) => {
                            tracing::warn!("缓存反序列化失败: {}, err: {}", cache_key, e);
                            // 删除损坏的缓存
                            let _ = conn.del::<_, i64>(&cache_key).await;
                            None
                        }
                    },
                    Ok(None) => None,
                    Err(e) => {
                        tracing::warn!("Redis GET 失败: {}, err: {}", cache_key, e);
                        None
                    }
                }
            }
            Err(e) => {
                tracing::warn!("获取 Redis 连接失败: {}", e);
                None
            }
        }
    }

    /// 写入缓存
    ///
    /// 失败只打日志，不阻断主流程
    pub async fn set(&self, bucket: &str, key: &str, meta: &FileMeta) {
        let pool = match self.pool.as_ref() {
            Some(p) => p,
            None => return,
        };
        let cache_key = Self::build_key(bucket, key);

        let json = match serde_json::to_string(meta) {
            Ok(j) => j,
            Err(e) => {
                tracing::warn!("缓存序列化失败: {}", e);
                return;
            }
        };

        match pool.get().await {
            Ok(mut conn) => {
                let result: Result<(), _> = conn
                    .set_ex(&cache_key, &json, CACHE_TTL_SECS)
                    .await;
                if let Err(e) = result {
                    tracing::warn!("Redis SET 失败: {}, err: {}", cache_key, e);
                }
            }
            Err(e) => {
                tracing::warn!("获取 Redis 连接失败: {}", e);
            }
        }
    }

    /// 删除缓存（失效）
    ///
    /// 在数据变更（确认/删除/更新）后调用
    pub async fn invalidate(&self, bucket: &str, key: &str) {
        let pool = match self.pool.as_ref() {
            Some(p) => p,
            None => return,
        };
        let cache_key = Self::build_key(bucket, key);

        match pool.get().await {
            Ok(mut conn) => {
                let result: Result<i64, _> = conn.del(&cache_key).await;
                if let Err(e) = result {
                    tracing::warn!("Redis DEL 失败: {}, err: {}", cache_key, e);
                }
            }
            Err(e) => {
                tracing::warn!("获取 Redis 连接失败: {}", e);
            }
        }
    }
}
