//! 媒体处理器 — 策略模式
//!
//! 所有处理类型实现 `MediaProcessor` trait，由工厂函数 `get_processor` 按 task_type 分发。

use crate::error::MediaError;
use async_trait::async_trait;
use std::path::{Path, PathBuf};

pub mod audio_extract;
pub mod image_resize;
pub mod image_watermark;
pub mod video_snapshot;
pub mod video_transcode;

/// 处理输出描述
#[derive(Debug, Clone)]
pub struct ProcessOutput {
    /// 本地文件路径
    pub local_path: PathBuf,
    /// 目标 S3 key
    pub s3_key: String,
    /// 文件 MIME 类型
    pub content_type: String,
    /// 产物类型标识
    pub output_type: String,
}

/// 媒体处理器接口 — 所有处理类型的统一抽象
#[async_trait]
pub trait MediaProcessor: Send + Sync {
    /// 执行处理
    ///
    /// - `input_path`:  本地输入文件路径（已下载）
    /// - `output_dir`:  本地输出目录（处理结果放这里）
    /// - `source_key`:  源文件 S3 key（用于生成衍生路径）
    /// - `params`:      任务参数 JSON
    ///
    /// 返回：衍生文件列表
    async fn process(
        &self,
        input_path: &Path,
        output_dir: &Path,
        source_key: &str,
        params: &serde_json::Value,
    ) -> Result<Vec<ProcessOutput>, MediaError>;
}

/// 根据 task_type 获取对应的处理器
pub fn get_processor(task_type: &str) -> Result<Box<dyn MediaProcessor>, MediaError> {
    match task_type {
        "VIDEO_SNAPSHOT" => Ok(Box::new(video_snapshot::VideoSnapshotProcessor)),
        "VIDEO_TRANSCODE" => Ok(Box::new(video_transcode::VideoTranscodeProcessor)),
        "IMAGE_RESIZE" => Ok(Box::new(image_resize::ImageResizeProcessor)),
        "IMAGE_WATERMARK" => Ok(Box::new(image_watermark::ImageWatermarkProcessor)),
        "AUDIO_EXTRACT" => Ok(Box::new(audio_extract::AudioExtractProcessor)),
        // TODO: VIDEO_HLS — 待独立迭代实现
        _ => Err(MediaError::UnsupportedTaskType(task_type.to_string())),
    }
}
