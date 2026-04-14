use crate::config::NotifyConfig;
use crate::kafka::{NotificationHandler, NotificationHandlerContext, OfflinePushHandler};
use crate::modules::notify_log::NotifyLogService;
use fbc_starter::{AppResult, Server};
use sqlxplus::DbPool;
use std::sync::Arc;

mod adapters;
mod config;
mod error;
mod handlers;
mod kafka;
mod models;
mod modules;
mod router;
pub mod client;

#[tokio::main]
async fn main() -> AppResult<()> {
    // 加载配置
    let config = NotifyConfig::from_env()?;

    let push_service = crate::modules::push::service::PushService::new(&config.notify)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to init PushService: {}", e))?;

    Server::run(move |builder| {
        let app_state = builder.app_state().clone();

        // 创建处理器上下文
        let mut context = NotificationHandlerContext::new(&config);

        // 尝试初始化 DB（如果配置了 MySQL）
        if let Some(mysql_pool) = app_state.mysql.as_ref() {
            match DbPool::from_mysql_pool(mysql_pool.clone()) {
                Ok(db_pool) => {
                    let db = Arc::new(db_pool);
                    let log_service = Arc::new(NotifyLogService::new(db));
                    context = context.with_log_service(log_service);
                    tracing::info!("📝 通知日志服务已启用");
                }
                Err(e) => {
                    tracing::warn!("通知日志服务初始化失败（降级为不记录）: {}", e);
                }
            }
        } else {
            tracing::warn!("MySQL 未配置，通知日志服务未启用");
        }

        let context = Arc::new(context);

        // HTTP 路由
        let http_router = router::create_router(context.clone());

        // Kafka 处理器
        let notification_handler: Arc<dyn fbc_starter::KafkaMessageHandler> =
            Arc::new(NotificationHandler::new(context));

        // 离线推送处理器
        let offline_push_handler: Arc<dyn fbc_starter::KafkaMessageHandler> =
            Arc::new(OfflinePushHandler::new(app_state, push_service.clone()));

        let handlers = vec![notification_handler, offline_push_handler];

        builder
            .with_kafka_handlers(handlers)
            .http_router(http_router)
    })
    .await
}
