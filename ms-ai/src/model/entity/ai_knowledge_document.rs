/// AiKnowledgeDocument
/// 
/// 表名: `ai_knowledge_document`
/// 主键: `id`
/// 逻辑删除字段: `deleted`
/// 字段数: 16

#[derive(Debug, Default, sqlx::FromRow, serde::Serialize, serde::Deserialize, sqlxplus::ModelMeta, sqlxplus::CRUD)]
#[model(table = "ai_knowledge_document", pk = "id", soft_delete = "deleted")]
pub struct AiKnowledgeDocument {
    /// 主键 | id (bigint) | 非空
    pub id: Option<i64>,
    /// knowledge_id (bigint) | 非空
    pub knowledge_id: i64,
    /// name (varchar(255)) | 非空
    pub name: String,
    /// url (varchar(1024)) | 非空
    pub url: String,
    /// content (text) | 非空
    pub content: String,
    /// content_length (int) | 非空
    pub content_length: i32,
    /// tokens (int) | 非空
    pub tokens: i32,
    /// segment_max_tokens (int) | 非空
    pub segment_max_tokens: i32,
    /// retrieval_count (int) | 非空
    /// 默认值: 0
    pub retrieval_count: Option<i32>,
    /// status (tinyint) | 非空
    pub status: i16,
    /// creator (varchar(64)) | 可空
    /// 默认值: 
    pub creator: Option<String>,
    /// create_time (datetime) | 非空
    /// 默认值: CURRENT_TIMESTAMP
    pub create_time: Option<chrono::NaiveDateTime>,
    /// updater (varchar(64)) | 可空
    /// 默认值: 
    pub updater: Option<String>,
    /// update_time (datetime) | 非空
    /// 默认值: CURRENT_TIMESTAMP
    pub update_time: Option<chrono::NaiveDateTime>,
    /// deleted (bit(1)) | 非空
    /// 默认值: b'0'
    pub deleted: Option<bool>,
    /// tenant_id (bigint) | 非空
    /// 默认值: 0
    pub tenant_id: Option<i64>,
}
