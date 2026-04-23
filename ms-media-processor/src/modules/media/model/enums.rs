#![allow(dead_code)]

use std::fmt;

use serde::{Deserialize, Serialize};

/// 任务类型枚举
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TaskType {
    /// 视频截图
    VideoSnapshot,
    /// 视频转码
    VideoTranscode,
    /// HLS 自适应码率切片
    VideoHls,
    /// 图片裁剪 / 缩放
    ImageResize,
    /// 图片加水印
    ImageWatermark,
    /// 音频提取
    AudioExtract,
}

impl fmt::Display for TaskType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TaskType::VideoSnapshot => write!(f, "VIDEO_SNAPSHOT"),
            TaskType::VideoTranscode => write!(f, "VIDEO_TRANSCODE"),
            TaskType::VideoHls => write!(f, "VIDEO_HLS"),
            TaskType::ImageResize => write!(f, "IMAGE_RESIZE"),
            TaskType::ImageWatermark => write!(f, "IMAGE_WATERMARK"),
            TaskType::AudioExtract => write!(f, "AUDIO_EXTRACT"),
        }
    }
}

impl TaskType {
    /// 从字符串解析任务类型
    pub fn from_str_value(s: &str) -> Option<Self> {
        match s {
            "VIDEO_SNAPSHOT" => Some(Self::VideoSnapshot),
            "VIDEO_TRANSCODE" => Some(Self::VideoTranscode),
            "VIDEO_HLS" => Some(Self::VideoHls),
            "IMAGE_RESIZE" => Some(Self::ImageResize),
            "IMAGE_WATERMARK" => Some(Self::ImageWatermark),
            "AUDIO_EXTRACT" => Some(Self::AudioExtract),
            _ => None,
        }
    }
}

/// 任务状态枚举
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TaskStatus {
    /// 待处理
    Init,
    /// 处理中
    Processing,
    /// 已完成
    Done,
    /// 已失败
    Failed,
}

impl fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TaskStatus::Init => write!(f, "INIT"),
            TaskStatus::Processing => write!(f, "PROCESSING"),
            TaskStatus::Done => write!(f, "DONE"),
            TaskStatus::Failed => write!(f, "FAILED"),
        }
    }
}

impl TaskStatus {
    /// 从字符串解析任务状态
    pub fn from_str_value(s: &str) -> Option<Self> {
        match s {
            "INIT" => Some(Self::Init),
            "PROCESSING" => Some(Self::Processing),
            "DONE" => Some(Self::Done),
            "FAILED" => Some(Self::Failed),
            _ => None,
        }
    }
}

/// 创建媒体任务状态机实例
///
/// 合法的状态转换：
/// - INIT → PROCESSING（乐观锁抢占）
/// - PROCESSING → DONE（处理成功）
/// - PROCESSING → FAILED（处理失败且超过重试次数）
/// - PROCESSING → INIT（处理失败但可重试）
pub fn create_task_state_machine() -> fbc_starter::state_machine::SimpleStateMachine<TaskStatus> {
    fbc_starter::state_machine::SimpleStateMachine::new(vec![
        (TaskStatus::Init, TaskStatus::Processing),
        (TaskStatus::Processing, TaskStatus::Done),
        (TaskStatus::Processing, TaskStatus::Failed),
        (TaskStatus::Processing, TaskStatus::Init), // 重试回退
    ])
}
