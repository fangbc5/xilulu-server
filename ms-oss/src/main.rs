mod config;
mod error;
mod modules;
mod provider;
mod router;
mod state;
mod utils;
mod kafka;

use axum::middleware;
use fbc_starter::{user_context_middleware, AppResult, Server};
use sqlxplus::DbPool;
use std::sync::Arc;

use crate::config::OssConfig;
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

        // 聚合为 OssState
        let oss_state = Arc::new(state::OssState::new(
            app_state, db_pool.clone(), oss_config, provider,
        ));

        // HTTP 路由
        let http_router = router::create_router(oss_state.clone())
            .layer(middleware::from_fn(user_context_middleware))
            .layer(fbc_starter::http::create_cors_layer(builder.config()));

        // Kafka 消费者 1：MinIO 上传事件 → 自动落库
        let minio_handler: Arc<dyn fbc_starter::KafkaMessageHandler> =
            Arc::new(kafka::MinioEventConsumerHandler::new(oss_state.file_service.clone()));

        // Kafka 消费者 2：媒体处理完成事件 → 回写 thumbnail_key
        let media_completed_handler: Arc<dyn fbc_starter::KafkaMessageHandler> =
            Arc::new(kafka::MediaCompletedConsumerHandler::new(db_pool));

        builder
            .http_router(http_router)
            .with_kafka_handler(minio_handler)
            .with_kafka_handler(media_completed_handler)
    })
    .await
}
