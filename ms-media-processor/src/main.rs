use fbc_starter::{AppResult, Server};
use sqlxplus::DbPool;
use std::sync::Arc;

mod config;
mod error;
mod kafka;
mod modules;

use config::MediaConfig;
use kafka::TaskConsumerHandler;
use modules::media::s3_client::S3Client;
use modules::media::service::MediaTaskService;

#[tokio::main]
async fn main() -> AppResult<()> {
    Server::run(move |builder| {
        // 加载业务配置（S3/MinIO），此时 fbc_starter 已完成 .env 文件的加载
        let media_config = MediaConfig::from_env();

        let fbc_app_state = builder.app_state().clone();

        // 数据库初始化
        let mysql_pool = fbc_app_state
            .mysql
            .as_ref()
            .expect("MySQL 连接池未初始化")
            .clone();
        let db_pool = Arc::new(
            DbPool::from_mysql_pool(mysql_pool).expect("创建 DbPool 失败"),
        );

        // 构建依赖
        let s3_client = Arc::new(S3Client::new(media_config));
        let producer = fbc_app_state
            .message_producer
            .clone()
            .expect("Kafka Producer 未初始化");

        // 创建 Service
        let service = Arc::new(MediaTaskService::new(
            db_pool,
            s3_client,
            producer,
        ));

        // 注册 Kafka 处理器
        let handler: Arc<dyn fbc_starter::KafkaMessageHandler> =
            Arc::new(TaskConsumerHandler::new(service));

        builder.with_kafka_handler(handler)
    })
    .await
}
