//! 视频截图处理器

use crate::error::MediaError;
use async_trait::async_trait;
use std::path::Path;
use std::process::Command;
use tracing::{error, info};

use super::{MediaProcessor, ProcessOutput};

/// 视频截图处理器 — 使用 FFmpeg 抽取指定时间点帧
pub struct VideoSnapshotProcessor;

#[async_trait]
impl MediaProcessor for VideoSnapshotProcessor {
    async fn process(
        &self,
        input_path: &Path,
        output_dir: &Path,
        source_key: &str,
        params: &serde_json::Value,
    ) -> Result<Vec<ProcessOutput>, MediaError> {
        let offset_ms = params.get("time_offset_ms").and_then(|v| v.as_u64()).unwrap_or(0);
        let format = params.get("format").and_then(|v| v.as_str()).unwrap_or("jpg");
        let quality = params.get("quality").and_then(|v| v.as_u64()).unwrap_or(2);

        let time_str = format!("{}.{:03}", offset_ms / 1000, offset_ms % 1000);
        let output_path = output_dir.join(format!("thumbnail.{}", format));

        info!("FFmpeg 截图: input={:?}, time={}, output={:?}", input_path, time_str, output_path);

        // 创建输出目录
        tokio::fs::create_dir_all(output_dir)
            .await
            .map_err(|e| MediaError::InternalError(format!("创建输出目录失败: {}", e)))?;

        // FFmpeg 抽帧（阻塞操作放在 spawn_blocking 中）
        let input = input_path.to_path_buf();
        let output = output_path.clone();
        let q_val = quality.to_string();

        let result = tokio::task::spawn_blocking(move || {
            Command::new("ffmpeg")
                .arg("-y")
                .arg("-ss").arg(&time_str)
                .arg("-i").arg(&input)
                .arg("-vframes").arg("1")
                .arg("-q:v").arg(&q_val)
                .arg(&output)
                .output()
        })
        .await
        .map_err(|e| MediaError::InternalError(format!("FFmpeg 任务 panic: {}", e)))?;

        match result {
            Ok(cmd_output) => {
                if !cmd_output.status.success() {
                    let err = String::from_utf8_lossy(&cmd_output.stderr);
                    error!("FFmpeg 截图失败: {}", err);
                    return Err(MediaError::FFmpegFailed(err.into_owned()));
                }
            }
            Err(e) => return Err(MediaError::FFmpegFailed(e.to_string())),
        }

        // 构建衍生路径
        let s3_key = format!("_derivative/{}_thumb.{}", source_key.replace('/', "_"), format);
        let content_type = match format {
            "png" => "image/png",
            "webp" => "image/webp",
            _ => "image/jpeg",
        };

        Ok(vec![ProcessOutput {
            local_path: output_path,
            s3_key,
            content_type: content_type.to_string(),
            output_type: "thumbnail".to_string(),
        }])
    }
}
