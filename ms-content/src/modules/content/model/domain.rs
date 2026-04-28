use serde::{Deserialize, Serialize};

/// Block DSL — 正文结构化组件
///
/// 所有内容形态共用这套协议，前端只需一套渲染器。
/// Phase 1 仅支持 6 种基础 Block，未知类型降级为 Unknown。
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Block {
    /// 纯文本/段落
    Text {
        value: String,
    },
    /// 标题
    Heading {
        level: u8,
        value: String,
    },
    /// 单张图片
    Image {
        key: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        caption: Option<String>,
    },
    /// 九宫格图片组
    Gallery {
        keys: Vec<String>,
    },
    /// 视频
    Video {
        key: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        cover: Option<String>,
    },
    /// 代码块
    Code {
        #[serde(skip_serializing_if = "Option::is_none")]
        lang: Option<String>,
        value: String,
    },
    /// 分隔线
    Divider {},
    /// 引用块
    Quote {
        value: String,
    },
}

impl Block {
    /// 提取 Block 内的纯文本内容（用于全文检索）
    pub fn extract_text(&self) -> Option<String> {
        match self {
            Block::Text { value } => Some(value.clone()),
            Block::Heading { value, .. } => Some(value.clone()),
            Block::Code { value, .. } => Some(value.clone()),
            Block::Quote { value } => Some(value.clone()),
            _ => None,
        }
    }
}

/// 从 Block DSL 数组提取全部纯文本（用于 body_text 字段和搜索索引）
pub fn extract_body_text(blocks: &[Block]) -> String {
    blocks
        .iter()
        .filter_map(|b| b.extract_text())
        .collect::<Vec<_>>()
        .join("\n")
}

/// 计算正文总字数
pub fn count_words(blocks: &[Block]) -> i32 {
    blocks
        .iter()
        .filter_map(|b| b.extract_text())
        .map(|t| t.chars().count() as i32)
        .sum()
}

/// 关系深度上限
pub const MAX_RELATION_DEPTH: u8 = 3;

/// 内容状态枚举
pub mod content_status {
    /// 草稿
    pub const DRAFT: i16 = 0;
    /// 待审核
    pub const PENDING_REVIEW: i16 = 1;
    /// 已发布
    pub const PUBLISHED: i16 = 2;
    /// 已下架
    pub const UNPUBLISHED: i16 = 3;
    /// 已删除
    pub const DELETED: i16 = 4;
}

/// 内容可见性枚举
pub mod visibility {
    /// 公开
    pub const PUBLIC: i16 = 0;
    /// 私密
    pub const PRIVATE: i16 = 1;
    /// 仅关注者可见
    pub const FOLLOWERS_ONLY: i16 = 2;
}
