#[cfg(feature = "mysql")]
use sqlx::mysql::{MySqlPool, MySqlPoolOptions};
#[cfg(feature = "postgres")]
use sqlx::postgres::{PgPool, PgPoolOptions};
#[cfg(feature = "sqlite")]
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};

use crate::{AppError, AppResult};

/// 三种数据库连接池
#[derive(Clone, Debug, Default)]
pub struct DatabasePools {
    #[cfg(feature = "mysql")]
    pub mysql: Option<MySqlPool>,
    #[cfg(feature = "postgres")]
    pub postgres: Option<PgPool>,
    #[cfg(feature = "sqlite")]
    pub sqlite: Option<SqlitePool>,
}

/// 根据配置初始化数据库连接池（可同时支持 MySQL/Postgres/SQLite）
pub async fn init_database(config: &crate::config::DatabaseConfig) -> AppResult<DatabasePools> {
    let mut pools = DatabasePools::default();

    #[cfg(feature = "mysql")]
    {
        let url = &config.url;
        let pool = MySqlPoolOptions::new()
            .max_connections(config.max_connections)
            .min_connections(config.min_connections)
            .connect(url)
            .await
            .map_err(AppError::Database)?;
        tracing::info!("数据库 MySQL 初始化成功");
        pools.mysql = Some(pool);
    }

    #[cfg(feature = "postgres")]
    {
        let url = &config.url;
        let pool = PgPoolOptions::new()
            .max_connections(config.max_connections)
            .min_connections(config.min_connections)
            .connect(url)
            .await
            .map_err(AppError::Database)?;
        tracing::info!("数据库 Postgres 初始化成功");
        pools.postgres = Some(pool);
    }

    #[cfg(feature = "sqlite")]
    {
        let url = &config.url;
        let pool = SqlitePoolOptions::new()
            .max_connections(config.max_connections)
            .min_connections(config.min_connections)
            .connect(url)
            .await
            .map_err(AppError::Database)?;
        tracing::info!("数据库 SQLite 初始化成功");
        pools.sqlite = Some(pool);
    }

    Ok(pools)
}
