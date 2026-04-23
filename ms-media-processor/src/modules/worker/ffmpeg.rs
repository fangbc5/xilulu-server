use crate::error::MediaError;
use std::process::Command;
use tracing::{info, error};
use std::path::Path;
use tokio::fs;

pub struct FFmpegProcessor;

impl FFmpegProcessor {
    /// 截取视频首帧或指定时间的帧
    pub async fn extract_thumbnail(
        input_path: &Path,
        output_path: &Path,
        time_offset_ms: u64,
    ) -> Result<(), MediaError> {
        let time_str = format!("{}.{:03}", time_offset_ms / 1000, time_offset_ms % 1000);
        
        info!("Running FFmpeg: -ss {} -i {:?} -vframes 1 {:?}", time_str, input_path, output_path);
        
        // 使用 tokio::task::spawn_blocking 运行阻塞的系统命令
        let input = input_path.to_path_buf();
        let output = output_path.to_path_buf();
        
        let result = tokio::task::spawn_blocking(move || {
            Command::new("ffmpeg")
                .arg("-y")                // 强制覆盖
                .arg("-ss").arg(&time_str) // 偏移时间
                .arg("-i").arg(&input)     // 输入文件
                .arg("-vframes").arg("1") // 仅截取一帧
                .arg("-q:v").arg("2")     // 图片质量（越小越好，2为较高质量）
                .arg(&output)
                .output()
        })
        .await
        .map_err(|e| MediaError::InternalError(format!("Task panic: {}", e)))?;

        match result {
            Ok(output_cmd) => {
                if !output_cmd.status.success() {
                    let err = String::from_utf8_lossy(&output_cmd.stderr);
                    error!("FFmpeg failed: {}", err);
                    return Err(MediaError::FFmpegFailed(err.into_owned()));
                }
                Ok(())
            }
            Err(e) => Err(MediaError::FFmpegFailed(e.to_string())),
        }
    }

    /// 清理临时文件
    pub async fn cleanup(path: &Path) {
        if path.exists() {
            if let Err(e) = fs::remove_file(path).await {
                error!("Failed to remove temp file {:?}: {}", path, e);
            }
        }
    }
}
