//! Kafka 消费者处理器
//!
//! 监听 ms-oss 发来的简单视频事件，自主创建任务并处理。
//! 遵循分库原则：ms-oss 只发简单消息，ms-media-processor 自治管理任务生命周期。

use std::sync::Arc;

use async_trait::async_trait;
use fbc_starter::KafkaMessageHandler;
use tracing::{error, info, warn};

use crate::modules::media::model::dto::{OssMediaEvent, SourceObject, SubmitTaskEvent};
use crate::modules::media::service::MediaTaskService;

/// 任务消费处理器
pub struct TaskConsumerHandler {
    service: Arc<MediaTaskService>,
}

impl TaskConsumerHandler {
    pub fn new(service: Arc<MediaTaskService>) -> Self {
        Self { service }
    }

    /// 将 ms-oss 的 action 映射为内部 task_type
    fn map_action_to_task_type(action: &str) -> Option<&'static str> {
        match action {
            "extract_video_thumbnail" => Some("VIDEO_SNAPSHOT"),
            "transcode_video" => Some("VIDEO_TRANSCODE"),
            "resize_image" => Some("IMAGE_RESIZE"),
            "watermark_image" => Some("IMAGE_WATERMARK"),
            "extract_audio" => Some("AUDIO_EXTRACT"),
            _ => None,
        }
    }
}

#[async_trait]
impl KafkaMessageHandler for TaskConsumerHandler {
    fn topics(&self) -> Vec<String> {
        // 监听 ms-oss 发出的视频处理事件
        vec!["sys.media".to_string()]
    }

    fn group_id(&self) -> String {
        "ms-media-processor-group".to_string()
    }

    async fn handle(&self, msg: fbc_starter::Message) {
        // 解析 ms-oss 发来的简单事件：{bucket, key, action}
        let event: OssMediaEvent = match serde_json::from_value(msg.data.clone()) {
            Ok(e) => e,
            Err(e) => {
                error!("反序列化 OssMediaEvent 失败: {}, data: {}", e, msg.data);
                return;
            }
        };

        // 映射 action → task_type
        let task_type = match Self::map_action_to_task_type(&event.action) {
            Some(t) => t,
            None => {
                warn!("未知的 action: {}，忽略", event.action);
                return;
            }
        };

        info!(
            "收到 ms-oss 视频处理请求: bucket={}, key={}, action={} → task_type={}",
            event.bucket, event.key, event.action, task_type
        );

        // 自主创建任务并处理（ms-media-processor 自治，不依赖 ms-oss 建表）
        let task_event = SubmitTaskEvent {
            task_id: uuid::Uuid::new_v4().to_string(),
            task_type: task_type.to_string(),
            source: SourceObject {
                bucket: event.bucket,
                key: event.key,
            },
            parameters: None,
            priority: None,
            callback_topic: None,
        };

        if let Err(e) = self.service.create_and_process(task_event).await {
            error!("任务处理异常: {:?}", e);
        }
    }
}
