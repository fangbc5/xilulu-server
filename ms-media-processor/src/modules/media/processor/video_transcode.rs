//! 视频转码处理器
//!
//! 使用 FFmpeg 将视频转码为指定编码器/分辨率/码率的 MP4 文件。
//! 支持 H.264 (libx264) 和 H.265 (libx265) 编码。

use crate::error::MediaError;
use async_trait::async_trait;
use std::path::Path;
use std::process::Command;
use tracing::{error, info};

use super::{MediaProcessor, ProcessOutput};

/// 视频转码处理器
pub struct VideoTranscodeProcessor;

#[async_trait]
impl MediaProcessor for VideoTranscodeProcessor {
    async fn process(
        &self,
        input_path: &Path,
        output_dir: &Path,
        source_key: &str,
        params: &serde_json::Value,
    ) -> Result<Vec<ProcessOutput>, MediaError> {
        // 解析转码参数
        let codec = params.get("codec").and_then(|v| v.as_str()).unwrap_or("h264");
        let resolution = params.get("resolution").and_then(|v| v.as_str());
        let bitrate = params.get("bitrate").and_then(|v| v.as_str()).unwrap_or("2000k");
        let preset = params.get("preset").and_then(|v| v.as_str()).unwrap_or("medium");
        let audio_bitrate = params.get("audio_bitrate").and_then(|v| v.as_str()).unwrap_or("128k");

        // 确定输出文件名后缀（使用分辨率标识）
        let res_suffix = resolution.unwrap_or("original");
        let output_path = output_dir.join(format!("transcode_{}.mp4", res_suffix.replace('x', "_")));

        info!(
            "FFmpeg 转码: input={:?}, codec={}, resolution={:?}, bitrate={}, preset={}",
            input_path, codec, resolution, bitrate, preset
        );

        // 创建输出目录
        tokio::fs::create_dir_all(output_dir)
            .await
            .map_err(|e| MediaError::InternalError(format!("创建输出目录失败: {}", e)))?;

        // 构建 FFmpeg 命令
        let input = input_path.to_path_buf();
        let output = output_path.clone();
        let codec = codec.to_string();
        let bitrate = bitrate.to_string();
        let preset = preset.to_string();
        let audio_bitrate = audio_bitrate.to_string();
        let resolution = resolution.map(|s| s.to_string());

        let result = tokio::task::spawn_blocking(move || {
            let mut cmd = Command::new("ffmpeg");
            cmd.arg("-y")
                .arg("-i").arg(&input);

            // 视频编码器
            match codec.as_str() {
                "h265" | "hevc" => {
                    cmd.arg("-c:v").arg("libx265")
                        .arg("-tag:v").arg("hvc1");
                }
                _ => {
                    cmd.arg("-c:v").arg("libx264")
                        .arg("-profile:v").arg("high");
                }
            }

            // 码率
            cmd.arg("-b:v").arg(&bitrate)
                .arg("-maxrate").arg(&bitrate)
                .arg("-bufsize").arg(format!("{}k", parse_bitrate_kbps(&bitrate) * 2));

            // 预设
            cmd.arg("-preset").arg(&preset);

            // 分辨率缩放
            if let Some(ref res) = resolution {
                if let Some((w, h)) = parse_resolution(res) {
                    // 使用 scale 滤镜，-2 保持偶数尺寸（编码器要求）
                    cmd.arg("-vf").arg(format!("scale={}:{}", w, h));
                }
            }

            // 音频编码
            cmd.arg("-c:a").arg("aac")
                .arg("-b:a").arg(&audio_bitrate);

            // 输出
            cmd.arg("-movflags").arg("+faststart")
                .arg(&output);

            cmd.output()
        })
        .await
        .map_err(|e| MediaError::InternalError(format!("FFmpeg 转码任务 panic: {}", e)))?;

        match result {
            Ok(cmd_output) => {
                if !cmd_output.status.success() {
                    let err = String::from_utf8_lossy(&cmd_output.stderr);
                    error!("FFmpeg 转码失败: {}", err);
                    return Err(MediaError::FFmpegFailed(err.into_owned()));
                }
            }
            Err(e) => return Err(MediaError::FFmpegFailed(e.to_string())),
        }

        // 构建衍生路径
        let s3_key = format!(
            "_derivative/{}_transcode_{}.mp4",
            source_key.replace('/', "_"),
            res_suffix.replace('x', "_")
        );

        info!("视频转码完成: {:?} → {}", output_path, s3_key);

        Ok(vec![ProcessOutput {
            local_path: output_path,
            s3_key,
            content_type: "video/mp4".to_string(),
            output_type: "transcode".to_string(),
        }])
    }
}

/// 解析码率字符串为 kbps 整数值（例如 "2000k" → 2000）
fn parse_bitrate_kbps(bitrate: &str) -> u64 {
    let s = bitrate.trim().to_lowercase();
    if let Some(num) = s.strip_suffix('k') {
        num.parse().unwrap_or(2000)
    } else if let Some(num) = s.strip_suffix('m') {
        num.parse::<u64>().unwrap_or(2) * 1000
    } else {
        s.parse::<u64>().unwrap_or(2000000) / 1000
    }
}

/// 解析分辨率字符串（例如 "1280x720" → (1280, -2)）
/// 高度使用 -2 以保持偶数尺寸
fn parse_resolution(res: &str) -> Option<(String, String)> {
    let parts: Vec<&str> = res.split('x').collect();
    if parts.len() == 2 {
        let w = parts[0].parse::<u32>().ok()?;
        // 使用 -2 让 FFmpeg 自动计算保持宽高比的偶数高度
        Some((w.to_string(), "-2".to_string()))
    } else {
        None
    }
}
