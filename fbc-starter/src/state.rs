use chrono::Local;
#[cfg(feature = "redis")]
use deadpool_redis::Pool;
#[cfg(feature = "mysql")]
use sqlx::mysql::MySqlPool;
#[cfg(feature = "postgres")]
use sqlx::postgres::PgPool;
#[cfg(feature = "sqlite")]
use sqlx::sqlite::SqlitePool;
use std::sync::Arc;

use dashmap::DashMap;
use std::any::{Any, TypeId};

/// 应用状态（包含数据库连接池、Redis 客户端、Kafka 消息生产者/消费者、用户扩展等）
#[derive(Clone)]
pub struct AppState {
    pub start_time: chrono::DateTime<Local>,
    #[cfg(feature = "mysql")]
    pub mysql: Option<Arc<MySqlPool>>,
    #[cfg(feature = "postgres")]
    pub postgres: Option<Arc<PgPool>>,
    #[cfg(feature = "sqlite")]
    pub sqlite: Option<Arc<SqlitePool>>,
    #[cfg(feature = "redis")]
    pub redis: Option<Pool>,
    #[cfg(feature = "producer")]
    pub message_producer: Option<crate::messaging::MessageProducerType>,
    #[cfg(feature = "consumer")]
    pub message_consumer: Option<crate::messaging::MessageConsumerType>,
    /// 用户扩展状态容器（类型安全的 AnyMap）
    extensions: Arc<DashMap<TypeId, Arc<dyn Any + Send + Sync>>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            start_time: Local::now(),
            #[cfg(feature = "mysql")]
            mysql: None,
            #[cfg(feature = "postgres")]
            postgres: None,
            #[cfg(feature = "sqlite")]
            sqlite: None,
            #[cfg(feature = "redis")]
            redis: None,
            #[cfg(feature = "producer")]
            message_producer: None,
            #[cfg(feature = "consumer")]
            message_consumer: None,
            extensions: Arc::new(DashMap::new()),
        }
    }

    /// 设置用户扩展状态（类型安全）
    ///
    /// # 示例
    /// ```ignore
    /// struct MyConfig { pub api_key: String }
    /// state.set_extension(MyConfig { api_key: "xxx".into() });
    /// ```
    pub fn set_extension<T: Any + Send + Sync + 'static>(&self, value: T) {
        self.extensions.insert(TypeId::of::<T>(), Arc::new(value));
    }

    /// 获取用户扩展状态（类型安全）
    ///
    /// # 示例
    /// ```ignore
    /// let config = state.get_extension::<MyConfig>();
    /// ```
    pub fn get_extension<T: Any + Send + Sync + 'static>(&self) -> Option<Arc<T>> {
        self.extensions
            .get(&TypeId::of::<T>())
            .and_then(|entry| entry.value().clone().downcast::<T>().ok())
    }

    /// 检查是否存在指定类型的扩展
    pub fn has_extension<T: Any + Send + Sync + 'static>(&self) -> bool {
        self.extensions.contains_key(&TypeId::of::<T>())
    }

    /// 带扩展的链式构建
    pub fn with_extension<T: Any + Send + Sync + 'static>(self, value: T) -> Self {
        self.set_extension(value);
        self
    }

    /// 设置 MySQL 连接池
    #[cfg(feature = "mysql")]
    pub fn with_mysql(mut self, pool: Arc<MySqlPool>) -> Self {
        self.mysql = Some(pool);
        self
    }

    /// 设置 Postgres 连接池
    #[cfg(feature = "postgres")]
    pub fn with_postgres(mut self, pool: Arc<PgPool>) -> Self {
        self.postgres = Some(pool);
        self
    }

    /// 设置 SQLite 连接池
    #[cfg(feature = "sqlite")]
    pub fn with_sqlite(mut self, pool: Arc<SqlitePool>) -> Self {
        self.sqlite = Some(pool);
        self
    }

    /// 批量设置三个数据库连接池
    #[cfg(any(feature = "mysql", feature = "postgres", feature = "sqlite"))]
    pub fn with_database_pools(mut self, pools: crate::database::DatabasePools) -> Self {
        #[cfg(feature = "mysql")]
        if let Some(pool) = pools.mysql {
            self.mysql = Some(Arc::new(pool));
        }
        #[cfg(feature = "postgres")]
        if let Some(pool) = pools.postgres {
            self.postgres = Some(Arc::new(pool));
        }
        #[cfg(feature = "sqlite")]
        if let Some(pool) = pools.sqlite {
            self.sqlite = Some(Arc::new(pool));
        }
        self
    }

    /// 设置 Redis 连接池
    #[cfg(feature = "redis")]
    pub fn with_redis(mut self, pool: Pool) -> Self {
        self.redis = Some(pool);
        self
    }

    /// 设置 Kafka 消息生产者
    #[cfg(feature = "producer")]
    pub fn with_message_producer(
        mut self,
        message_producer: crate::messaging::MessageProducerType,
    ) -> Self {
        self.message_producer = Some(message_producer);
        self
    }

    /// 设置 Kafka 消息消费者
    #[cfg(feature = "consumer")]
    pub fn with_message_consumer(
        mut self,
        message_consumer: crate::messaging::MessageConsumerType,
    ) -> Self {
        self.message_consumer = Some(message_consumer);
        self
    }

    /// 获取 MySQL 连接池
    #[cfg(feature = "mysql")]
    pub fn mysql(&self) -> crate::error::AppResult<Arc<MySqlPool>> {
        self.mysql
            .clone()
            .ok_or(crate::error::AppError::DatabaseNotInitialized)
    }

    /// 获取 Postgres 连接池
    #[cfg(feature = "postgres")]
    pub fn postgres(&self) -> crate::error::AppResult<Arc<PgPool>> {
        self.postgres
            .clone()
            .ok_or(crate::error::AppError::DatabaseNotInitialized)
    }

    /// 获取 SQLite 连接池
    #[cfg(feature = "sqlite")]
    pub fn sqlite(&self) -> crate::error::AppResult<Arc<SqlitePool>> {
        self.sqlite
            .clone()
            .ok_or(crate::error::AppError::DatabaseNotInitialized)
    }

    /// 获取 Redis 连接
    ///
    /// 每次调用都会从连接池获取一个新的连接，支持并发操作
    #[cfg(feature = "redis")]
    pub async fn redis(&self) -> crate::error::AppResult<deadpool_redis::Connection> {
        self.redis
            .as_ref()
            .ok_or(crate::error::AppError::RedisNotInitialized)?
            .get()
            .await
            .map_err(|e| {
                crate::error::AppError::Internal(anyhow::anyhow!("获取 Redis 连接失败: {}", e))
            })
    }

    /// 获取 Kafka 消息生产者
    #[cfg(feature = "producer")]
    pub fn message_producer(
        &self,
    ) -> crate::error::AppResult<&crate::messaging::MessageProducerType> {
        self.message_producer.as_ref().ok_or_else(|| {
            crate::error::AppError::Internal(anyhow::anyhow!("Kafka 消息生产者未初始化"))
        })
    }

    /// 获取 Kafka 消息消费者
    #[cfg(feature = "consumer")]
    pub fn message_consumer(
        &self,
    ) -> crate::error::AppResult<&crate::messaging::MessageConsumerType> {
        self.message_consumer.as_ref().ok_or_else(|| {
            crate::error::AppError::Internal(anyhow::anyhow!("Kafka 消息消费者未初始化"))
        })
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
