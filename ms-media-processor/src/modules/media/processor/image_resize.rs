//! 图片裁剪/缩放处理器
//!
//! 使用 FFmpeg scale/crop 滤镜进行图片裁剪和缩放。
//! 支持三种模式：cover（裁剪填充）、contain（等比缩放）、fill（拉伸）。

use crate::error::MediaError;
use async_trait::async_trait;
use std::path::Path;
use std::process::Command;
use tracing::{error, info};

use super::{MediaProcessor, ProcessOutput};

/// 图片裁剪/缩放处理器
pub struct ImageResizeProcessor;

#[async_trait]
impl MediaProcessor for ImageResizeProcessor {
    async fn process(
        &self,
        input_path: &Path,
        output_dir: &Path,
        source_key: &str,
        params: &serde_json::Value,
    ) -> Result<Vec<ProcessOutput>, MediaError> {
        let width = params.get("width").and_then(|v| v.as_u64()).unwrap_or(800) as u32;
        let height = params.get("height").and_then(|v| v.as_u64()).unwrap_or(600) as u32;
        let mode = params.get("mode").and_then(|v| v.as_str()).unwrap_or("cover");
        let format = params.get("format").and_then(|v| v.as_str()).unwrap_or("jpg");
        let quality = params.get("quality").and_then(|v| v.as_u64()).unwrap_or(85);

        let output_path = output_dir.join(format!("resize_{}x{}.{}", width, height, format));

        info!(
            "FFmpeg 图片裁剪: input={:?}, {}x{}, mode={}, format={}, quality={}",
            input_path, width, height, mode, format, quality
        );

        // 创建输出目录
        tokio::fs::create_dir_all(output_dir)
            .await
            .map_err(|e| MediaError::InternalError(format!("创建输出目录失败: {}", e)))?;

        // 构建 FFmpeg 滤镜表达式
        let filter = build_scale_filter(width, height, mode);

        let input = input_path.to_path_buf();
        let output = output_path.clone();
        let format_str = format.to_string();

        let result = tokio::task::spawn_blocking(move || {
            let mut cmd = Command::new("ffmpeg");
            cmd.arg("-y")
                .arg("-i").arg(&input)
                .arg("-vf").arg(&filter)
                .arg("-frames:v").arg("1");

            // 根据输出格式设置质量参数
            match format_str.as_str() {
                "webp" => {
                    // WebP 质量范围 0-100，直接使用
                    cmd.arg("-quality").arg(quality.to_string());
                }
                "png" => {
                    // PNG 无损，不需要质量参数
                }
                _ => {
                    // JPEG: -q:v 范围 2-31，将 0-100 转换为 31-2
                    let q = quality_to_jpeg_qv(quality);
                    cmd.arg("-q:v").arg(q.to_string());
                }
            }

            cmd.arg(&output).output()
        })
        .await
        .map_err(|e| MediaError::InternalError(format!("FFmpeg 图片裁剪任务 panic: {}", e)))?;

        match result {
            Ok(cmd_output) => {
                if !cmd_output.status.success() {
                    let err = String::from_utf8_lossy(&cmd_output.stderr);
                    error!("FFmpeg 图片裁剪失败: {}", err);
                    return Err(MediaError::FFmpegFailed(err.into_owned()));
                }
            }
            Err(e) => return Err(MediaError::FFmpegFailed(e.to_string())),
        }

        // 构建衍生路径
        let s3_key = format!(
            "_derivative/{}_resize_{}x{}.{}",
            source_key.replace('/', "_"),
            width, height, format
        );

        let content_type = match format {
            "png" => "image/png",
            "webp" => "image/webp",
            _ => "image/jpeg",
        };

        info!("图片裁剪完成: {:?} → {}", output_path, s3_key);

        Ok(vec![ProcessOutput {
            local_path: output_path,
            s3_key,
            content_type: content_type.to_string(),
            output_type: "resize".to_string(),
        }])
    }
}

/// 根据模式构建 FFmpeg scale/crop 滤镜
fn build_scale_filter(width: u32, height: u32, mode: &str) -> String {
    match mode {
        // cover: 等比放大后居中裁剪（保证填满目标尺寸）
        "cover" => format!(
            "scale={}:{}:force_original_aspect_ratio=increase,crop={}:{}",
            width, height, width, height
        ),
        // contain: 等比缩小（保证完全显示在目标尺寸内）
        "contain" => format!(
            "scale={}:{}:force_original_aspect_ratio=decrease,pad={}:{}:(ow-iw)/2:(oh-ih)/2",
            width, height, width, height
        ),
        // fill: 拉伸到目标尺寸
        "fill" => format!("scale={}:{}", width, height),
        // 默认使用 cover
        _ => format!(
            "scale={}:{}:force_original_aspect_ratio=increase,crop={}:{}",
            width, height, width, height
        ),
    }
}

/// 将 0-100 的质量百分比转换为 JPEG 的 q:v 参数（范围 2-31）
/// 100 → 2（最高质量），0 → 31（最低质量）
fn quality_to_jpeg_qv(quality: u64) -> u32 {
    let q = quality.min(100) as f64;
    // 线性映射: quality 100 → qv 2, quality 0 → qv 31
    let qv = 31.0 - (q / 100.0 * 29.0);
    qv.round() as u32
}
