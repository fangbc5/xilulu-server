//! 音频提取处理器
//!
//! 使用 FFmpeg 从视频文件中提取音频轨道。
//! 支持 AAC 和 MP3 两种输出编码。

use crate::error::MediaError;
use async_trait::async_trait;
use std::path::Path;
use std::process::Command;
use tracing::{error, info};

use super::{MediaProcessor, ProcessOutput};

/// 音频提取处理器
pub struct AudioExtractProcessor;

#[async_trait]
impl MediaProcessor for AudioExtractProcessor {
    async fn process(
        &self,
        input_path: &Path,
        output_dir: &Path,
        source_key: &str,
        params: &serde_json::Value,
    ) -> Result<Vec<ProcessOutput>, MediaError> {
        let codec = params.get("codec").and_then(|v| v.as_str()).unwrap_or("aac");
        let bitrate = params.get("bitrate").and_then(|v| v.as_str()).unwrap_or("128k");

        // 根据编码器确定输出格式和 MIME 类型
        let (ext, ffmpeg_codec, content_type) = match codec {
            "mp3" => ("mp3", "libmp3lame", "audio/mpeg"),
            // 默认 AAC
            _ => ("aac", "aac", "audio/aac"),
        };

        let output_path = output_dir.join(format!("audio.{}", ext));

        info!(
            "FFmpeg 音频提取: input={:?}, codec={}, bitrate={}",
            input_path, codec, bitrate
        );

        // 创建输出目录
        tokio::fs::create_dir_all(output_dir)
            .await
            .map_err(|e| MediaError::InternalError(format!("创建输出目录失败: {}", e)))?;

        let input = input_path.to_path_buf();
        let output = output_path.clone();
        let ffmpeg_codec = ffmpeg_codec.to_string();
        let bitrate = bitrate.to_string();
        let ext_str = ext.to_string();

        let result = tokio::task::spawn_blocking(move || {
            let mut cmd = Command::new("ffmpeg");
            cmd.arg("-y")
                .arg("-i").arg(&input)
                .arg("-vn"); // 移除视频流

            // 音频编码器
            cmd.arg("-c:a").arg(&ffmpeg_codec)
                .arg("-b:a").arg(&bitrate);

            // AAC 需要 ADTS 容器格式（独立播放兼容）
            if ext_str == "aac" {
                cmd.arg("-f").arg("adts");
            }

            cmd.arg(&output).output()
        })
        .await
        .map_err(|e| MediaError::InternalError(format!("FFmpeg 音频提取任务 panic: {}", e)))?;

        match result {
            Ok(cmd_output) => {
                if !cmd_output.status.success() {
                    let err = String::from_utf8_lossy(&cmd_output.stderr);
                    error!("FFmpeg 音频提取失败: {}", err);
                    return Err(MediaError::FFmpegFailed(err.into_owned()));
                }
            }
            Err(e) => return Err(MediaError::FFmpegFailed(e.to_string())),
        }

        // 构建衍生路径
        let s3_key = format!(
            "_derivative/{}_audio.{}",
            source_key.replace('/', "_"),
            ext
        );

        info!("音频提取完成: {:?} → {}", output_path, s3_key);

        Ok(vec![ProcessOutput {
            local_path: output_path,
            s3_key,
            content_type: content_type.to_string(),
            output_type: "audio".to_string(),
        }])
    }
}
