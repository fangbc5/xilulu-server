use serde::{Deserialize, Serialize};

/// ========================================
/// 内容类型 Schema 注册表实体
/// ========================================
#[derive(
    Debug,
    Default,
    Clone,
    sqlx::FromRow,
    Serialize,
    Deserialize,
    sqlxplus::ModelMeta,
    sqlxplus::CRUD,
)]
#[model(table = "content_schema", pk = "id")]
pub struct ContentSchema {
    /// 主键 ID
    pub id: Option<i64>,
    /// 内容类型唯一标识
    pub content_type: Option<String>,
    /// 类型中文名
    pub display_name: Option<String>,
    /// ext_data 的 JSON Schema 定义
    pub schema_definition: Option<serde_json::Value>,
    /// 0=禁用 1=启用
    pub status: Option<i16>,
    /// 创建时间
    pub created_at: Option<i64>,
    /// 更新时间
    pub updated_at: Option<i64>,
}

/// ========================================
/// 内容主表实体（路由与控制）
/// ========================================
#[derive(
    Debug,
    Default,
    Clone,
    sqlx::FromRow,
    Serialize,
    Deserialize,
    sqlxplus::ModelMeta,
    sqlxplus::CRUD,
)]
#[model(table = "content_main", pk = "id")]
pub struct ContentMain {
    /// 主键 ID
    pub id: Option<i64>,
    /// 对外暴露的业务标识（UUID v4）
    pub content_id: Option<String>,
    /// 内容类型
    pub content_type: Option<String>,
    /// 作者 ID
    pub author_id: Option<i64>,
    /// 状态：0=草稿 1=待审核 2=已发布 3=已下架 4=已删除
    pub status: Option<i16>,
    /// 可见性：0=公开 1=私密 2=仅关注者可见
    pub visibility: Option<i16>,
    /// 0=普通 1=置顶
    pub pinned: Option<i16>,
    /// 发布时间
    pub published_at: Option<i64>,
    /// 创建时间
    pub created_at: Option<i64>,
    /// 更新时间
    pub updated_at: Option<i64>,
    /// 乐观锁版本号
    pub version: Option<i32>,
}

/// ========================================
/// 内容详情表实体（Block DSL + 扩展）
/// ========================================
#[derive(
    Debug,
    Default,
    Clone,
    sqlx::FromRow,
    Serialize,
    Deserialize,
    sqlxplus::ModelMeta,
    sqlxplus::CRUD,
)]
#[model(table = "content_detail", pk = "id")]
pub struct ContentDetail {
    /// 主键 ID
    pub id: Option<i64>,
    /// → content_main.id
    pub content_id: Option<i64>,
    /// 标题
    pub title: Option<String>,
    /// 摘要/简介
    pub summary: Option<String>,
    /// 封面图 OSS Key
    pub cover_image: Option<String>,
    /// 正文 Block DSL
    pub body: Option<serde_json::Value>,
    /// 正文纯文本
    pub body_text: Option<String>,
    /// 类型专属扩展字段
    pub ext_data: Option<serde_json::Value>,
    /// 字数统计
    pub word_count: Option<i32>,
}

/// ========================================
/// 内容统计计数表实体
/// ========================================
#[derive(
    Debug,
    Default,
    Clone,
    sqlx::FromRow,
    Serialize,
    Deserialize,
    sqlxplus::ModelMeta,
    sqlxplus::CRUD,
)]
#[model(table = "content_stats", pk = "id")]
pub struct ContentStats {
    /// 主键 ID
    pub id: Option<i64>,
    /// → content_main.id
    pub content_id: Option<i64>,
    /// 浏览量
    pub view_count: Option<i64>,
    /// 点赞数
    pub like_count: Option<i32>,
    /// 评论数
    pub comment_count: Option<i32>,
    /// 分享数
    pub share_count: Option<i32>,
    /// 收藏数
    pub collect_count: Option<i32>,
}

/// ========================================
/// 内容关系图表实体
/// ========================================
#[derive(
    Debug,
    Default,
    Clone,
    sqlx::FromRow,
    Serialize,
    Deserialize,
    sqlxplus::ModelMeta,
    sqlxplus::CRUD,
)]
#[model(table = "content_relation", pk = "id")]
pub struct ContentRelation {
    /// 主键
    pub id: Option<i64>,
    /// 关系外部标识
    pub relation_id: Option<String>,
    /// 发起方内容 ID
    pub source_id: Option<i64>,
    /// 目标方内容 ID
    pub target_id: Option<i64>,
    /// 关系类型
    pub relation_type: Option<String>,
    /// 0=双向 1=单向
    pub direction: Option<i16>,
    /// 边属性
    pub metadata: Option<serde_json::Value>,
    /// 创建时间
    pub created_at: Option<i64>,
}
