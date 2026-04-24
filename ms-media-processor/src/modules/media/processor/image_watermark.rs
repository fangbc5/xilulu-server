//! 图片水印处理器
//!
//! 使用 FFmpeg drawtext 滤镜在图片上添加文字水印。
//! 支持 5 个位置：top-left / top-right / bottom-left / bottom-right / center。

use crate::error::MediaError;
use async_trait::async_trait;
use std::path::Path;
use std::process::Command;
use tracing::{error, info};

use super::{MediaProcessor, ProcessOutput};

/// 图片水印处理器
pub struct ImageWatermarkProcessor;

#[async_trait]
impl MediaProcessor for ImageWatermarkProcessor {
    async fn process(
        &self,
        input_path: &Path,
        output_dir: &Path,
        source_key: &str,
        params: &serde_json::Value,
    ) -> Result<Vec<ProcessOutput>, MediaError> {
        let text = params.get("text").and_then(|v| v.as_str()).unwrap_or("Watermark");
        let position = params.get("position").and_then(|v| v.as_str()).unwrap_or("bottom-right");
        let opacity = params.get("opacity").and_then(|v| v.as_f64()).unwrap_or(0.5);
        let font_size = params.get("font_size").and_then(|v| v.as_u64()).unwrap_or(24) as u32;
        let font_color = params.get("font_color").and_then(|v| v.as_str()).unwrap_or("white");

        // 保持与源文件相同的格式
        let ext = input_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("jpg");
        let output_path = output_dir.join(format!("watermark.{}", ext));

        info!(
            "FFmpeg 水印: input={:?}, text='{}', position={}, opacity={}, font_size={}",
            input_path, text, position, opacity, font_size
        );

        // 创建输出目录
        tokio::fs::create_dir_all(output_dir)
            .await
            .map_err(|e| MediaError::InternalError(format!("创建输出目录失败: {}", e)))?;

        // 构建 drawtext 滤镜
        let filter = build_drawtext_filter(text, position, opacity, font_size, font_color);

        let input = input_path.to_path_buf();
        let output = output_path.clone();

        let result = tokio::task::spawn_blocking(move || {
            Command::new("ffmpeg")
                .arg("-y")
                .arg("-i").arg(&input)
                .arg("-vf").arg(&filter)
                .arg("-frames:v").arg("1")
                .arg(&output)
                .output()
        })
        .await
        .map_err(|e| MediaError::InternalError(format!("FFmpeg 水印任务 panic: {}", e)))?;

        match result {
            Ok(cmd_output) => {
                if !cmd_output.status.success() {
                    let err = String::from_utf8_lossy(&cmd_output.stderr);
                    error!("FFmpeg 水印添加失败: {}", err);
                    return Err(MediaError::FFmpegFailed(err.into_owned()));
                }
            }
            Err(e) => return Err(MediaError::FFmpegFailed(e.to_string())),
        }

        // 构建衍生路径
        let s3_key = format!(
            "_derivative/{}_watermark.{}",
            source_key.replace('/', "_"),
            ext
        );

        let content_type = match ext {
            "png" => "image/png",
            "webp" => "image/webp",
            _ => "image/jpeg",
        };

        info!("水印添加完成: {:?} → {}", output_path, s3_key);

        Ok(vec![ProcessOutput {
            local_path: output_path,
            s3_key,
            content_type: content_type.to_string(),
            output_type: "watermark".to_string(),
        }])
    }
}

/// 构建 FFmpeg drawtext 滤镜字符串
///
/// 位置映射：
/// - `top-left`     → x=20:y=20
/// - `top-right`    → x=w-tw-20:y=20
/// - `bottom-left`  → x=20:y=h-th-20
/// - `bottom-right`  → x=w-tw-20:y=h-th-20
/// - `center`       → x=(w-tw)/2:y=(h-th)/2
fn build_drawtext_filter(
    text: &str,
    position: &str,
    opacity: f64,
    font_size: u32,
    font_color: &str,
) -> String {
    let (x, y) = match position {
        "top-left" => ("20", "20"),
        "top-right" => ("w-tw-20", "20"),
        "bottom-left" => ("20", "h-th-20"),
        "center" => ("(w-tw)/2", "(h-th)/2"),
        // 默认右下角
        _ => ("w-tw-20", "h-th-20"),
    };

    // alpha 透明度（0.0-1.0）
    let alpha = opacity.clamp(0.0, 1.0);

    // 转义文本中的特殊字符（FFmpeg drawtext 语法要求）
    let escaped_text = text
        .replace('\\', "\\\\")
        .replace(':', "\\:")
        .replace('\'', "\\'");

    format!(
        "drawtext=text='{}':fontsize={}:fontcolor={}@{:.2}:x={}:y={}",
        escaped_text, font_size, font_color, alpha, x, y
    )
}
