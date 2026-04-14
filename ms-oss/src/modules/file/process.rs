//! 文件后处理框架
//!
//! 上传回调成功后，按配置异步触发文件处理 pipeline。
//! 当前为占位实现，待有实际业务需求后补充处理逻辑。

use async_trait::async_trait;
use super::model::entity::FileMeta;

/// 文件处理上下文
pub struct ProcessContext {
    pub bucket: String,
    pub key: String,
    pub meta: FileMeta,
}

/// 处理结果
pub struct ProcessResult {
    /// 处理后的新 key（如缩略图 key）
    pub output_key: Option<String>,
    /// 处理类型标记
    pub processor_name: String,
}

/// 文件处理器 trait
#[async_trait]
pub trait FileProcessor: Send + Sync {
    /// 处理器名称
    fn name(&self) -> &str;

    /// 是否支持处理该文件
    fn supports(&self, content_type: &str, scene: &str) -> bool;

    /// 执行处理
    async fn process(&self, ctx: &ProcessContext) -> anyhow::Result<ProcessResult>;
}

// ============================================
// 水印处理器（占位）
// ============================================

pub struct WatermarkProcessor;

#[async_trait]
impl FileProcessor for WatermarkProcessor {
    fn name(&self) -> &str {
        "watermark"
    }

    fn supports(&self, content_type: &str, _scene: &str) -> bool {
        content_type.starts_with("image/")
    }

    async fn process(&self, ctx: &ProcessContext) -> anyhow::Result<ProcessResult> {
        // TODO: 实际水印处理逻辑（需引入 image crate）
        tracing::info!("🔲 水印处理（占位）: bucket={}, key={}", ctx.bucket, ctx.key);
        Ok(ProcessResult {
            output_key: None,
            processor_name: self.name().to_string(),
        })
    }
}

// ============================================
// 缩略图处理器（占位）
// ============================================

pub struct ThumbnailProcessor;

#[async_trait]
impl FileProcessor for ThumbnailProcessor {
    fn name(&self) -> &str {
        "thumbnail"
    }

    fn supports(&self, content_type: &str, _scene: &str) -> bool {
        content_type.starts_with("image/")
    }

    async fn process(&self, ctx: &ProcessContext) -> anyhow::Result<ProcessResult> {
        // TODO: 实际缩略图生成逻辑（需引入 image crate）
        let thumb_key = format!("{}_thumb", ctx.key);
        tracing::info!("🖼️ 缩略图生成（占位）: bucket={}, key={} → {}", ctx.bucket, ctx.key, thumb_key);
        Ok(ProcessResult {
            output_key: Some(thumb_key),
            processor_name: self.name().to_string(),
        })
    }
}
