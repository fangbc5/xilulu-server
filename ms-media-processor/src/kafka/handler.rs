use crate::error::MediaError;
use crate::modules::task::model::dto::{CompletedTaskEvent, ResultObject, SubmitTaskEvent};
use crate::modules::task::repository::TaskRepository;
use crate::modules::worker::ffmpeg::FFmpegProcessor;
use crate::modules::worker::s3_client::S3Client;
use async_trait::async_trait;
use fbc_starter::messaging::{Message, MessageProducer};
use fbc_starter::KafkaMessageHandler;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{error, info, warn};

pub struct TaskConsumerHandler {
    task_repo: Arc<TaskRepository>,
    producer: Arc<dyn MessageProducer>,
    s3_client: Arc<S3Client>,
}

impl TaskConsumerHandler {
    pub fn new(
        task_repo: Arc<TaskRepository>,
        producer: Arc<dyn MessageProducer>,
        s3_client: Arc<S3Client>,
    ) -> Self {
        Self {
            task_repo,
            producer,
            s3_client,
        }
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
        let event: SubmitTaskEvent = match serde_json::from_value(msg.data.clone()) {
            Ok(e) => e,
            Err(e) => {
                error!("Failed to parse SubmitTaskEvent: {}", e);
                return;
            }
        };

        if let Err(e) = self.process_task(event).await {
            error!("Error processing task: {:?}", e);
        }
    }
}

impl TaskConsumerHandler {
    async fn process_task(&self, event: SubmitTaskEvent) -> Result<(), MediaError> {
        let task_id = &event.task_id;

        // 1. 获取任务详情
        let task = match self.task_repo.get_task(task_id).await? {
            Some(t) => t,
            None => {
                warn!("Task {} not found in DB, ignoring.", task_id);
                return Ok(());
            }
        };

        // 2. 尝试乐观锁抢占
        let claimed = self
            .task_repo
            .claim_task(task_id, task.version.unwrap_or(1))
            .await?;
        if !claimed {
            info!(
                "Task {} already claimed or processed by another worker.",
                task_id
            );
            return Ok(());
        }

        info!(
            "Task {} claimed successfully. Starting processing...",
            task_id
        );

        // 3. 执行核心逻辑（包裹在 catch 逻辑中处理错误和 DLQ）
        match self.execute_media_logic(&event).await {
            Ok(result_key) => {
                // 处理成功，标记已完成
                self.task_repo.mark_done(task_id, &result_key).await?;

                // 发布成功事件
                let completed = CompletedTaskEvent {
                    task_id: task_id.clone(),
                    status: "DONE".to_string(),
                    original_source: event.source.key.clone(),
                    result: Some(ResultObject {
                        bucket: event.source.bucket.clone(),
                        key: result_key,
                        size: None, // 可选
                    }),
                    error_msg: None,
                };

                let _ = self
                    .producer
                    .publish(
                        "sys.media.task.completed",
                        Message::new(
                            "sys.media.task.completed",
                            "ms-media-processor",
                            serde_json::to_value(completed).unwrap(),
                        ),
                    )
                    .await;

                info!("Task {} completed successfully.", task_id);
            }
            Err(e) => {
                error!("Task {} failed to execute: {:?}", task_id, e);
                // 标记失败或增加重试
                let can_retry = self
                    .task_repo
                    .mark_failed_or_retry(task_id, &e.to_string())
                    .await?;
                if !can_retry {
                    // 发送到死信队列
                    let _ = self
                        .producer
                        .publish(
                            "sys.media.task.dlq",
                            Message::new(
                                "sys.media.task.dlq",
                                "ms-media-processor",
                                serde_json::to_value(&event).unwrap(),
                            ),
                        )
                        .await;
                }
            }
        }

        Ok(())
    }

    async fn execute_media_logic(&self, event: &SubmitTaskEvent) -> Result<String, MediaError> {
        let tmp_input = PathBuf::from(format!("/tmp/input_{}.mp4", event.task_id));
        let tmp_output = PathBuf::from(format!("/tmp/output_{}.jpg", event.task_id));

        // 1. 从 S3 拉取大文件
        info!(
            "Downloading video from S3: {}/{}",
            event.source.bucket, event.source.key
        );
        self.s3_client
            .download_to_file(&event.source.bucket, &event.source.key, &tmp_input)
            .await?;

        // 2. FFmpeg 抽帧
        let offset_ms = event
            .parameters
            .as_ref()
            .and_then(|p| p.get("time_offset_ms"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        info!("Extracting frame using FFmpeg at ms: {}", offset_ms);
        FFmpegProcessor::extract_thumbnail(&tmp_input, &tmp_output, offset_ms).await?;

        // 3. 上传到 S3 衍生目录
        let result_key = format!(
            "_derivative/{}_thumb.jpg",
            event.source.key.replace("/", "_")
        );
        info!(
            "Uploading extracted frame to S3: {}/{}",
            event.source.bucket, result_key
        );

        self.s3_client
            .upload_from_file(&event.source.bucket, &result_key, &tmp_output, "image/jpeg")
            .await?;

        // 4. 清理临时文件
        FFmpegProcessor::cleanup(&tmp_input).await;
        FFmpegProcessor::cleanup(&tmp_output).await;

        Ok(result_key)
    }
}
