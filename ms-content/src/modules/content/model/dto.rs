use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use super::domain::Block;

// ========================================
// 请求 DTO
// ========================================

/// 创建内容请求
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateContentReq {
    /// 内容类型（必须已在 content_schema 注册）
    #[schema(example = "article")]
    pub content_type: String,
    /// 标题（Moment 类型可为空）
    #[serde(default)]
    pub title: Option<String>,
    /// 摘要
    #[serde(default)]
    pub summary: Option<String>,
    /// 封面图 OSS Key
    #[serde(default)]
    pub cover_image: Option<String>,
    /// 正文 Block DSL
    pub body: Vec<Block>,
    /// 类型专属扩展字段
    #[serde(default)]
    pub ext_data: Option<serde_json::Value>,
    /// 可见性：0=公开 1=私密 2=仅关注者可见
    #[serde(default)]
    #[schema(example = 0)]
    pub visibility: Option<i16>,
}

/// 更新内容请求
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateContentReq {
    /// 标题
    #[serde(default)]
    pub title: Option<String>,
    /// 摘要
    #[serde(default)]
    pub summary: Option<String>,
    /// 封面图 OSS Key
    #[serde(default)]
    pub cover_image: Option<String>,
    /// 正文 Block DSL
    #[serde(default)]
    pub body: Option<Vec<Block>>,
    /// 类型专属扩展字段
    #[serde(default)]
    pub ext_data: Option<serde_json::Value>,
    /// 可见性
    #[serde(default)]
    pub visibility: Option<i16>,
    /// 乐观锁版本号（必须传）
    pub version: i32,
}

/// 变更内容状态请求
#[derive(Debug, Deserialize, ToSchema)]
pub struct ChangeStatusReq {
    /// 目标状态（0=草稿 1=待审核 2=已发布 3=已下架 4=已删除）
    #[schema(example = 2)]
    pub status: i16,
    /// 乐观锁版本号
    #[schema(example = 1)]
    pub version: i32,
}

/// 创建关系请求
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateRelationReq {
    /// 目标内容 ID
    pub target_id: i64,
    /// 关系类型（comment / reply / attach / quote / collection）
    #[schema(example = "comment")]
    pub relation_type: String,
    /// 方向：0=双向 1=单向
    #[serde(default = "default_direction")]
    #[schema(example = 1)]
    pub direction: i16,
    /// 边属性
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
}

fn default_direction() -> i16 {
    1
}

/// 搜索请求（游标分页）
#[derive(Debug, Deserialize, ToSchema)]
pub struct SearchContentReq {
    /// 游标分页参数
    #[serde(flatten)]
    pub page: fbc_starter::base::CursorPageBaseReq,
    /// 关键词
    #[serde(default)]
    pub keyword: Option<String>,
    /// 内容类型筛选
    #[serde(default)]
    #[schema(example = "article")]
    pub content_type: Option<String>,
    /// 作者 ID 筛选
    #[serde(default)]
    pub author_id: Option<i64>,
    /// 排序字段
    #[serde(default = "default_sort")]
    #[schema(example = "published_at:desc")]
    pub sort_by: String,
}

fn default_sort() -> String {
    "published_at:desc".to_string()
}

/// 关系查询参数（游标分页）
#[derive(Debug, Deserialize, ToSchema)]
pub struct RelationQueryParams {
    /// 游标分页参数
    #[serde(flatten)]
    pub page: fbc_starter::base::CursorPageBaseReq,
    /// 关系类型
    #[serde(rename = "type")]
    #[schema(example = "comment")]
    pub relation_type: String,
}

// ========================================
// 响应 DTO
// ========================================

/// 内容详情响应
#[derive(Debug, Serialize, ToSchema)]
pub struct ContentDetailResp {
    /// 内容 ID
    pub id: i64,
    /// 外部业务 ID（UUID）
    pub content_id: String,
    /// 内容类型
    pub content_type: String,
    /// 作者 ID
    pub author_id: i64,
    /// 状态
    pub status: i16,
    /// 可见性
    pub visibility: i16,
    /// 是否置顶
    pub pinned: i16,
    /// 标题
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// 摘要
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    /// 封面图
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cover_image: Option<String>,
    /// 正文 Block DSL
    pub body: serde_json::Value,
    /// 类型扩展数据
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ext_data: Option<serde_json::Value>,
    /// 字数统计
    pub word_count: i32,
    /// 统计数据
    pub stats: ContentStatsResp,
    /// 发布时间
    pub published_at: i64,
    /// 创建时间
    pub created_at: i64,
    /// 更新时间
    pub updated_at: i64,
    /// 版本号
    pub version: i32,
}

/// 统计数据响应
#[derive(Debug, Default, Serialize, ToSchema)]
pub struct ContentStatsResp {
    pub view_count: i64,
    pub like_count: i32,
    pub comment_count: i32,
    pub share_count: i32,
    pub collect_count: i32,
}

/// 关系响应
#[derive(Debug, Serialize, ToSchema)]
pub struct ContentRelationResp {
    pub id: i64,
    pub relation_id: String,
    pub source_id: i64,
    pub target_id: i64,
    pub relation_type: String,
    pub direction: i16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
    pub created_at: i64,
}
