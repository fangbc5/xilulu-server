//! Kafka 消费者处理器 — 薄层
//!
//! 只负责：反序列化消息 → 调用 Service → 记录日志

use std::sync::Arc;

use async_trait::async_trait;
use fbc_starter::KafkaMessageHandler;
use tracing::error;

use crate::modules::media::model::dto::SubmitTaskEvent;
use crate::modules::media::service::MediaTaskService;

/// 任务消费处理器
pub struct TaskConsumerHandler {
    service: Arc<MediaTaskService>,
}

impl TaskConsumerHandler {
    pub fn new(service: Arc<MediaTaskService>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl KafkaMessageHandler for TaskConsumerHandler {
    fn topics(&self) -> Vec<String> {
        vec!["sys.media.task.submit".to_string()]
    }

    fn group_id(&self) -> String {
        "ms-media-processor-group".to_string()
    }

    async fn handle(&self, msg: fbc_starter::Message) {
        // 反序列化
        let event: SubmitTaskEvent = match serde_json::from_value(msg.data.clone()) {
            Ok(e) => e,
            Err(e) => {
                error!("反序列化 SubmitTaskEvent 失败: {}", e);
                return;
            }
        };

        // 调用 service 处理
        if let Err(e) = self.service.process(event).await {
            error!("任务处理异常: {:?}", e);
        }
    }
}
