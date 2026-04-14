/// AiChatConversation
/// 
/// 表名: `ai_chat_conversation`
/// 主键: `id`
/// 逻辑删除字段: `deleted`
/// 字段数: 18

#[derive(Debug, Default, sqlx::FromRow, serde::Serialize, serde::Deserialize, sqlxplus::ModelMeta, sqlxplus::CRUD)]
#[model(table = "ai_chat_conversation", pk = "id", soft_delete = "deleted")]
pub struct AiChatConversation {
    /// 主键 | id (bigint) | 非空
    pub id: Option<i64>,
    /// user_id (bigint) | 非空
    pub user_id: i64,
    /// role_id (bigint) | 可空
    pub role_id: Option<i64>,
    /// title (varchar(256)) | 非空
    pub title: String,
    /// model_id (bigint) | 非空
    pub model_id: i64,
    /// model (varchar(100)) | 非空
    pub model: String,
    /// pinned (bit(1)) | 非空
    pub pinned: bool,
    /// pinned_time (datetime) | 可空
    pub pinned_time: Option<chrono::NaiveDateTime>,
    /// system_message (varchar(1024)) | 可空
    pub system_message: Option<String>,
    /// temperature (double) | 非空
    pub temperature: f64,
    /// max_tokens (int) | 非空
    pub max_tokens: i32,
    /// max_contexts (int) | 非空
    pub max_contexts: i32,
    /// creator (varchar(64)) | 可空
    pub creator: Option<String>,
    /// create_time (datetime) | 可空
    pub create_time: Option<chrono::NaiveDateTime>,
    /// updater (varchar(64)) | 可空
    pub updater: Option<String>,
    /// update_time (datetime) | 可空
    pub update_time: Option<chrono::NaiveDateTime>,
    /// deleted (bit(1)) | 非空
    /// 默认值: b'0'
    pub deleted: Option<bool>,
    /// tenant_id (bigint) | 可空
    pub tenant_id: Option<i64>,
}
