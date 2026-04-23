//! 媒体处理完成事件消费者
//!
//! 监听 ms-media-processor 发来的 `sys.media.task.completed` 事件，
//! 将视频截帧产物的 key 回写到 `file_meta.thumbnail_key`。
//! 遵循分库原则：ms-oss 只管自己的 file_meta 表。

use std::sync::Arc;
use async_trait::async_trait;
use serde::Deserialize;
use tracing::{info, warn, error};
use fbc_starter::KafkaMessageHandler;

use crate::modules::file::repository::FileMetaRepo;
use sqlxplus::DbPool;

/// 媒体处理完成事件（来自 ms-media-processor）
#[derive(Debug, Deserialize)]
struct MediaCompletedEvent {
    /// 任务状态
    status: String,
    /// 源文件 Bucket
    source_bucket: String,
    /// 源文件 Key
    original_source: String,
    /// 处理结果
    result: Option<MediaTaskResult>,
}

#[derive(Debug, Deserialize)]
struct MediaTaskResult {
    /// 主产物路径（即缩略图 key）
    primary_key: String,
}

/// 媒体完成事件消费处理器
pub struct MediaCompletedConsumerHandler {
    db: Arc<DbPool>,
}

impl MediaCompletedConsumerHandler {
    pub fn new(db: Arc<DbPool>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl KafkaMessageHandler for MediaCompletedConsumerHandler {
    fn topics(&self) -> Vec<String> {
        vec!["sys.media.task.completed".to_string()]
    }

    fn group_id(&self) -> String {
        "ms-oss-media-completed-group".to_string()
    }

    async fn handle(&self, msg: fbc_starter::Message) {
        let event: MediaCompletedEvent = match serde_json::from_value(msg.data.clone()) {
            Ok(e) => e,
            Err(e) => {
                error!("反序列化 MediaCompletedEvent 失败: {}", e);
                return;
            }
        };

        if event.status != "DONE" {
            warn!(
                "收到非成功的媒体处理完成事件: status={}, bucket={}, key={}",
                event.status, event.source_bucket, event.original_source
            );
            return;
        }

        let thumbnail_key = match &event.result {
            Some(r) if !r.primary_key.is_empty() => &r.primary_key,
            _ => {
                warn!("媒体完成事件中缺少 primary_key，跳过");
                return;
            }
        };

        info!(
            "收到媒体处理完成事件: bucket={}, key={}, thumbnail_key={}",
            event.source_bucket, event.original_source, thumbnail_key
        );

        // 更新 file_meta.thumbnail_key
        if let Err(e) = FileMetaRepo::update_thumbnail_key(
            &self.db,
            &event.source_bucket,
            &event.original_source,
            thumbnail_key,
        )
        .await
        {
            error!(
                "更新 thumbnail_key 失败: bucket={}, key={}, error: {}",
                event.source_bucket, event.original_source, e
            );
        } else {
            info!(
                "已回写 thumbnail_key: {}/{} → {}",
                event.source_bucket, event.original_source, thumbnail_key
            );
        }
    }
}
