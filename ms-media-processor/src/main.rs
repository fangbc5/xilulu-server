use crate::config::MediaProcessorConfig;
use crate::kafka::handler::TaskConsumerHandler;
use crate::modules::task::repository::TaskRepository;
use crate::modules::worker::s3_client::S3Client;
use crate::state::ProcessState;
use fbc_starter::{AppResult, Server};
use sqlxplus::DbPool;
use std::sync::Arc;

pub mod config;
pub mod error;
pub mod kafka;
pub mod modules;
pub mod state;

#[tokio::main]
async fn main() -> AppResult<()> {
    // 1. 加载配置
    let config = MediaProcessorConfig::from_env()?;

    Server::run(move |builder| {
        let fbc_app_state = builder.app_state().clone();

        // 2. 数据库池初始化
        let mysql_pool = fbc_app_state
            .mysql
            .as_ref()
            .expect("MySQL not configured")
            .clone();
        let db_pool =
            Arc::new(DbPool::from_mysql_pool(mysql_pool).expect("Failed to create DbPool"));

        // 3. 构建依赖仓库与客户端
        let task_repo = Arc::new(TaskRepository::new(db_pool.clone()));
        let s3_client = Arc::new(S3Client::new(config.media.clone()));

        // 如果要发送消息到 Kafka，则使用 fbc_starter 提供的 Producer
        let producer = fbc_app_state
            .message_producer
            .clone()
            .expect("Kafka Producer not configured");

        // 4. 配置 Kafka 处理器
        let media_handler: Arc<dyn fbc_starter::KafkaMessageHandler> = Arc::new(
            TaskConsumerHandler::new(task_repo.clone(), producer, s3_client),
        );

        // 5. 将 State 存入备用，用于未来的 HTTP API （如需要）
        let _process_state = Arc::new(ProcessState {
            fbc_app_state: fbc_app_state,
            db: db_pool,
            task_repo,
        });

        // 注册到 server 构造器
        builder.with_kafka_handlers(vec![media_handler])
    })
    .await
}
