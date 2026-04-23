use serde::{Deserialize, Serialize};

/// ms-oss 发来的简单事件（纯事件驱动，不含 task_id）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OssMediaEvent {
    /// 源文件 Bucket
    pub bucket: String,
    /// 源文件 Key
    pub key: String,
    /// 动作类型（extract_video_thumbnail 等）
    pub action: String,
}

/// 源文件描述
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceObject {
    pub bucket: String,
    pub key: String,
}

/// 任务提交事件（内部流转用，由 handler 自行构建）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitTaskEvent {
    /// 任务 ID（UUID）
    pub task_id: String,
    /// 任务类型
    pub task_type: String,
    /// 源文件
    pub source: SourceObject,
    /// 处理参数 JSON
    pub parameters: Option<serde_json::Value>,
    /// 优先级
    pub priority: Option<String>,
    /// 完成回调 topic（可选）
    pub callback_topic: Option<String>,
}

/// 任务完成事件（Kafka 出站消息，发回给 ms-oss）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletedTaskEvent {
    /// 任务 ID
    pub task_id: String,
    /// 任务状态
    pub status: String,
    /// 任务类型
    pub task_type: String,
    /// 源文件 Bucket（ms-oss 需要用此定位 file_meta）
    pub source_bucket: String,
    /// 原始源路径
    pub original_source: String,
    /// 处理结果
    pub result: Option<TaskResult>,
    /// 错误信息
    pub error_msg: Option<String>,
    /// 处理耗时（毫秒）
    pub processing_time_ms: Option<i64>,
}

/// 处理结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    /// 主产物路径
    pub primary_key: String,
    /// 所有产物列表
    pub outputs: Vec<OutputItem>,
}

/// 单个产物描述
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputItem {
    /// S3 路径
    pub key: String,
    /// 产物类型
    pub output_type: String,
    /// MIME 类型
    pub content_type: String,
    /// 文件大小（字节）
    pub size: Option<i64>,
}
