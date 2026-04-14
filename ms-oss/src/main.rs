mod config;
mod error;
mod modules;
mod provider;
mod router;
mod state;

use axum::middleware;
use fbc_starter::{user_context_middleware, AppResult, Server};
use sqlxplus::DbPool;
use std::sync::Arc;

use crate::config::OssConfig;
use crate::modules::file::process::{FileProcessor, ThumbnailProcessor, WatermarkProcessor};
use crate::provider::s3_compat::S3CompatProvider;

#[tokio::main]
async fn main() -> AppResult<()> {
    Server::run(|builder| {
        // 从启动器获取全局 AppState
        let app_state = builder.app_state().clone();

        // 获取 MySQL 连接池
        let mysql_pool = app_state
            .mysql
            .as_ref()
            .expect("MySQL 连接池未初始化")
            .clone();

        // 创建 sqlxplus DbPool
        let db_pool = Arc::new(DbPool::from_mysql_pool(mysql_pool).expect("创建 DbPool 失败"));

        // 加载 OSS 配置（从环境变量）
        let oss_config = OssConfig::from_env();

        // 初始化 OSS Provider（同步初始化，无需 block_on）
        let provider: Arc<dyn crate::provider::OssProvider> = Arc::new(S3CompatProvider::new(
            &oss_config.endpoint,
            &oss_config.public_endpoint,
            &oss_config.region,
            &oss_config.access_key,
            &oss_config.secret_key,
        ));

        // 按配置注册文件处理器
        let mut processors: Vec<Arc<dyn FileProcessor>> = Vec::new();
        if oss_config.watermark_enabled {
            tracing::info!("🔲 水印处理已启用");
            processors.push(Arc::new(WatermarkProcessor));
        }
        if oss_config.thumbnail_enabled {
            tracing::info!("🖼️ 缩略图生成已启用");
            processors.push(Arc::new(ThumbnailProcessor));
        }

        // 聚合为 OssState
        let oss_state = Arc::new(state::OssState::new(
            app_state, db_pool, oss_config, provider, processors,
        ));

        // HTTP 路由
        let http_router =
            router::create_router(oss_state).layer(middleware::from_fn(user_context_middleware));

        builder.http_router(http_router)
    })
    .await
}
