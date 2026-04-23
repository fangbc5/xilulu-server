//! MinIO S3 Event Consumer
//!
//! 监听 MinIO 原生推送的 S3 Webhook 事件（例如 put_object）。
//! 收到事件后提取 bucket 和 key，调用 FileService 的 confirm_upload，
//! 以异步代替以前的前端同步确认。

use std::sync::Arc;
use async_trait::async_trait;
use tracing::{info, debug, error};
use fbc_starter::KafkaMessageHandler;

use crate::modules::file::service::FileService;

/// MinIO 事件消费处理器
pub struct MinioEventConsumerHandler {
    file_service: Arc<FileService>,
}

impl MinioEventConsumerHandler {
    pub fn new(file_service: Arc<FileService>) -> Self {
        Self { file_service }
    }
}

#[async_trait]
impl KafkaMessageHandler for MinioEventConsumerHandler {
    fn topics(&self) -> Vec<String> {
        // 监听 user 在 docker mc event 里配置的 topic
        vec!["minio-events".to_string()]
    }

    fn group_id(&self) -> String {
        "ms-oss-minio-group".to_string()
    }

    async fn handle(&self, msg: fbc_starter::Message) {
        // 打印完整的原始 Kafka 消息内容以便调试和观察（生产环境建议 debug 级别）
        debug!("【Kafka收到消息】Raw MinIO Event (topic: {}): {}", msg.topic, msg.data);

        // 解析 MinIO 发来的 S3 Event JSON
        // 格式通常为:
        // {
        //   "EventName": "s3:ObjectCreated:Put",
        //   "Records": [
        //      {
        //         "eventName": "s3:ObjectCreated:Put",
        //         "s3": {
        //            "bucket": { "name": "public" },
        //            "object": { "key": "avatar/2026/04/..." }
        //         }
        //      }
        //   ]
        // }

        let records = match msg.data.get("Records").and_then(|r| r.as_array()) {
            Some(v) => v,
            None => {
                // 有些时候 MinIO 发送测试/诊断事件或结构不同，忽略
                return;
            }
        };

        for record in records {
            let event_name = record
                .get("eventName")
                .and_then(|e| e.as_str())
                .unwrap_or("");

            // 只处理创建相关的事件，包括单文件和分片合并完成
            if event_name.starts_with("s3:ObjectCreated:") {
                let s3 = match record.get("s3") {
                    Some(v) => v,
                    None => continue,
                };
                
                let bucket = s3
                    .get("bucket")
                    .and_then(|b| b.get("name"))
                    .and_then(|n| n.as_str())
                    .unwrap_or("");

                let encoded_key = s3
                    .get("object")
                    .and_then(|o| o.get("key"))
                    .and_then(|k| k.as_str())
                    .unwrap_or("");

                if bucket.is_empty() || encoded_key.is_empty() {
                    continue;
                }

                // S3 的 key 通常是 URLEncoded 的，我们要 decode 才能进行正常存取
                let decode_result = urlencoding::decode(encoded_key);
                let key = match decode_result {
                    Ok(k) => k.into_owned(),
                    Err(_) => encoded_key.to_string(), // fallback
                };

                info!("收到 MinIO 原生 Webhook 消息: bucket={}, key={}", bucket, key);

                // 直接调用原来的确认方法（这里会自动生成元数据记录，并发起下游业务 kafka 消息）
                if let Err(e) = self.file_service.confirm_upload(bucket, &key).await {
                    error!("Kafka MinIO Event 处理文件落地确认失败: bucket={}, key={}, error: {:?}", bucket, key, e);
                } else {
                    info!("自动落库确认完毕: {}/{}", bucket, key);
                }
            }
        }
    }
}
