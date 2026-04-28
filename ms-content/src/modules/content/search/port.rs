use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// 搜索文档（写入模型 — 由 Service 从 DB 组装）
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct SearchDocument {
    /// 内容 ID（作为 Meilisearch 主键，确保覆盖写幂等）
    pub id: i64,
    /// 内容类型
    pub content_type: String,
    /// 作者 ID
    pub author_id: i64,
    /// 标题
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// 摘要
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    /// 正文纯文本（用于全文检索）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_text: Option<String>,
    /// 封面图
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cover_image: Option<String>,
    /// 状态
    pub status: i16,
    /// 可见性
    pub visibility: i16,
    /// 发布时间
    pub published_at: i64,
    /// 创建时间
    pub created_at: i64,
    /// 浏览量
    pub view_count: i64,
    /// 点赞数
    pub like_count: i32,
}

/// 搜索条件
#[derive(Debug, Clone)]
pub struct SearchCriteria {
    /// 关键词
    pub keyword: Option<String>,
    /// 内容类型筛选
    pub content_type: Option<String>,
    /// 作者 ID 筛选
    pub author_id: Option<i64>,
    /// 排序（如 "published_at:desc"）
    pub sort_by: String,
    /// 偏移量（游标分页）
    pub offset: u32,
    /// 每页条数
    pub limit: u32,
}

/// 搜索结果
#[derive(Debug, Clone)]
pub struct SearchResult {
    /// 命中的文档列表
    pub hits: Vec<SearchDocument>,
    /// 总命中数
    pub total: u64,
}

/// 搜索端口 — 业务层唯一的搜索契约
///
/// 业务层不直接依赖任何搜索引擎 SDK，
/// 仅通过此 trait 进行索引和查询操作。
#[async_trait]
pub trait SearchPort: Send + Sync {
    /// 索引/更新文档（以 id 为主键覆盖写，保证幂等）
    async fn index(&self, doc: SearchDocument) -> anyhow::Result<()>;

    /// 批量索引文档
    async fn batch_index(&self, docs: Vec<SearchDocument>) -> anyhow::Result<()>;

    /// 删除文档
    async fn delete(&self, id: i64) -> anyhow::Result<()>;

    /// 搜索
    async fn search(&self, criteria: SearchCriteria) -> anyhow::Result<SearchResult>;
}
