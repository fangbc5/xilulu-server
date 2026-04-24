//! 媒体任务服务 — 业务编排层
//!
//! Handler → **Service** → Repository 单向调用
//! 遵循分库原则：任务的创建和管理完全在本服务内自治完成。

use std::sync::Arc;

use fbc_starter::messaging::{Message, MessageProducer};
use sqlxplus::DbPool;
use tracing::{error, info, warn};

use crate::error::MediaError;

use super::model::dto::{CompletedTaskEvent, OutputItem, SubmitTaskEvent, TaskResult};
use super::model::entity::{MediaTask, MediaTaskOutput};
use super::processor::{get_processor, ProcessOutput};
use super::repository::MediaTaskRepo;
use super::s3_client::S3Client;

use sqlxplus::Crud;

/// 媒体任务服务
pub struct MediaTaskService {
    /// 数据库连接池（ms_media 库）
    db: Arc<DbPool>,
    /// S3 文件客户端
    s3: Arc<S3Client>,
    /// Kafka 生产者
    producer: Arc<dyn MessageProducer>,
}

impl MediaTaskService {
    pub fn new(db: Arc<DbPool>, s3: Arc<S3Client>, producer: Arc<dyn MessageProducer>) -> Self {
        Self { db, s3, producer }
    }

    /// 自治入口 — 自建任务 + 处理
    ///
    /// 由 Kafka handler 构建好 SubmitTaskEvent 后调用。
    /// 本方法负责：创建 media_task 记录 → 抢占 → 执行 → 回传结果给 ms-oss。
    pub async fn create_and_process(&self, event: SubmitTaskEvent) -> Result<(), MediaError> {
        let now = chrono::Utc::now().timestamp_millis();

        // 1. 自主创建任务记录（ms_media 库）
        let task = MediaTask {
            id: None,
            task_id: Some(event.task_id.clone()),
            source_bucket: Some(event.source.bucket.clone()),
            source_key: Some(event.source.key.clone()),
            task_type: Some(event.task_type.clone()),
            parameters: event.parameters.as_ref().map(|p| p.to_string()),
            status: Some("INIT".to_string()),
            priority: Some(0),
            retry_count: Some(0),
            max_retry: Some(3),
            version: Some(1),
            result_key: None,
            result_meta: None,
            error_message: None,
            callback_topic: event.callback_topic.clone(),
            created_by: Some("ms-oss".to_string()),
            created_at: Some(now),
            updated_at: Some(now),
        };

        task.insert(self.db.mysql_pool())
            .await
            .map_err(|e| MediaError::DatabaseFailed(format!("创建任务记录失败: {}", e)))?;

        info!("任务 {} 已创建，开始处理", event.task_id);

        // 2. 调用已有的处理流程
        self.process(event).await
    }

    /// 处理入口 — 任务已存在于 DB 中
    pub async fn process(&self, event: SubmitTaskEvent) -> Result<(), MediaError> {
        let task_id = &event.task_id;
        let start_time = chrono::Utc::now().timestamp_millis();

        // 1. 查询任务
        let task = MediaTaskRepo::find_by_task_id(self.db.mysql_pool(), task_id)
            .await?
            .ok_or_else(|| {
                warn!("任务 {} 在数据库中不存在，忽略", task_id);
                MediaError::LockFailed(format!("任务 {} 不存在", task_id))
            })?;

        // 2. 乐观锁抢占
        let claimed =
            MediaTaskRepo::claim_task(self.db.mysql_pool(), task_id, task.version.unwrap_or(1))
                .await?;
        if !claimed {
            info!("任务 {} 已被其他 worker 抢占，跳过", task_id);
            return Ok(());
        }
        info!("任务 {} 抢占成功，开始处理", task_id);

        // 3. 执行核心逻辑
        match self.execute(&event).await {
            Ok(outputs) => {
                // 成功：更新状态 + 保存产物 + 发布完成事件
                let primary_key = outputs
                    .first()
                    .map(|o| o.s3_key.clone())
                    .unwrap_or_default();

                let result_meta =
                    serde_json::to_string(&outputs.iter().map(|o| &o.s3_key).collect::<Vec<_>>())
                        .ok();

                MediaTaskRepo::mark_done(
                    self.db.mysql_pool(),
                    task_id,
                    &primary_key,
                    result_meta.as_deref(),
                )
                .await?;

                // 保存产物记录
                self.save_outputs(task_id, &outputs).await;

                // 发布完成事件（回传给 ms-oss）
                let elapsed = chrono::Utc::now().timestamp_millis() - start_time;
                self.publish_completed(task_id, &event, &outputs, elapsed)
                    .await;

                info!("任务 {} 处理成功，耗时 {}ms", task_id, elapsed);
            }
            Err(e) => {
                error!("任务 {} 处理失败: {:?}", task_id, e);
                let can_retry = MediaTaskRepo::mark_failed_or_retry(
                    self.db.mysql_pool(),
                    task_id,
                    &e.to_string(),
                )
                .await?;

                if !can_retry {
                    // 超过最大重试次数，发送到死信队列
                    self.publish_dlq(task_id, &event).await;
                }
            }
        }

        Ok(())
    }

    /// 执行核心媒体处理逻辑
    async fn execute(&self, event: &SubmitTaskEvent) -> Result<Vec<ProcessOutput>, MediaError> {
        // 1. 获取处理器（策略模式）
        let processor = get_processor(&event.task_type)?;

        // 2. 准备临时工作目录
        let work_dir =
            std::env::var("MEDIA_WORK_DIR").unwrap_or_else(|_| "/tmp/media_work".to_string());
        let task_dir = std::path::PathBuf::from(&work_dir).join(&event.task_id);
        let input_dir = task_dir.join("input");
        let output_dir = task_dir.join("output");

        tokio::fs::create_dir_all(&input_dir)
            .await
            .map_err(|e| MediaError::InternalError(format!("创建工作目录失败: {}", e)))?;

        // 3. 下载源文件
        let file_ext = event.source.key.rsplit('.').next().unwrap_or("bin");
        let input_path = input_dir.join(format!("source.{}", file_ext));

        info!(
            "下载源文件: {}/{} → {:?}",
            event.source.bucket, event.source.key, input_path
        );
        self.s3
            .download_to_file(&event.source.bucket, &event.source.key, &input_path)
            .await?;

        // 4. 执行处理
        let params = event.parameters.clone().unwrap_or_default();
        let outputs = processor
            .process(&input_path, &output_dir, &event.source.key, &params)
            .await?;

        // 5. 上传所有产物到 S3
        for output in &outputs {
            info!(
                "上传产物: {} → {}/{}",
                output.output_type, event.source.bucket, output.s3_key
            );
            self.s3
                .upload_from_file(
                    &event.source.bucket,
                    &output.s3_key,
                    &output.local_path,
                    &output.content_type,
                )
                .await?;
        }

        // 6. 清理临时文件
        if let Err(e) = tokio::fs::remove_dir_all(&task_dir).await {
            warn!("清理临时目录 {:?} 失败: {}", task_dir, e);
        }

        Ok(outputs)
    }

    /// 保存产物记录到数据库
    async fn save_outputs(&self, task_id: &str, outputs: &[ProcessOutput]) {
        for output in outputs {
            let record = MediaTaskOutput {
                task_id: Some(task_id.to_string()),
                output_key: Some(output.s3_key.clone()),
                output_type: Some(output.output_type.clone()),
                content_type: Some(output.content_type.clone()),
                ..Default::default()
            };
            if let Err(e) = record.insert(self.db.mysql_pool()).await {
                error!("保存产物记录失败: {}", e);
            }
        }
    }

    /// 发布任务完成事件（回传给 ms-oss，携带 source_bucket 供其定位 file_meta）
    async fn publish_completed(
        &self,
        task_id: &str,
        event: &SubmitTaskEvent,
        outputs: &[ProcessOutput],
        elapsed_ms: i64,
    ) {
        let completed = CompletedTaskEvent {
            task_id: task_id.to_string(),
            status: "DONE".to_string(),
            task_type: event.task_type.clone(),
            source_bucket: event.source.bucket.clone(),
            original_source: event.source.key.clone(),
            result: Some(TaskResult {
                primary_key: outputs
                    .first()
                    .map(|o| o.s3_key.clone())
                    .unwrap_or_default(),
                outputs: outputs
                    .iter()
                    .map(|o| OutputItem {
                        key: o.s3_key.clone(),
                        output_type: o.output_type.clone(),
                        content_type: o.content_type.clone(),
                        size: None,
                    })
                    .collect(),
            }),
            error_msg: None,
            processing_time_ms: Some(elapsed_ms),
        };

        // 固定发到 sys.media.task.completed，由 ms-oss 消费
        let topic = "sys.media.task.completed";

        let _ = self
            .producer
            .publish(
                topic,
                Message::new(
                    topic,
                    "ms-media-processor",
                    serde_json::to_value(&completed).unwrap(),
                ),
            )
            .await;
    }

    /// 发布到死信队列
    async fn publish_dlq(&self, task_id: &str, event: &SubmitTaskEvent) {
        warn!("任务 {} 超过最大重试次数，发送到 DLQ", task_id);
        let _ = self
            .producer
            .publish(
                "sys.media.task.dlq",
                Message::new(
                    "sys.media.task.dlq",
                    "ms-media-processor",
                    serde_json::to_value(event).unwrap(),
                ),
            )
            .await;
    }
}
