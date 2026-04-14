/// AiChatMessage
/// 
/// 表名: `ai_chat_message`
/// 主键: `id`
/// 逻辑删除字段: `deleted`
/// 字段数: 19

#[derive(Debug, Default, sqlx::FromRow, serde::Serialize, serde::Deserialize, sqlxplus::ModelMeta, sqlxplus::CRUD)]
#[model(table = "ai_chat_message", pk = "id", soft_delete = "deleted")]
pub struct AiChatMessage {
    /// 主键 | id (bigint) | 非空
    pub id: Option<i64>,
    /// conversation_id (bigint) | 非空
    pub conversation_id: i64,
    /// reply_id (bigint) | 可空
    pub reply_id: Option<i64>,
    /// user_id (bigint) | 非空
    pub user_id: i64,
    /// role_id (bigint) | 可空
    pub role_id: Option<i64>,
    /// type (varchar(16)) | 非空
    pub r#type: String,
    /// model (varchar(100)) | 非空
    pub model: String,
    /// model_id (bigint) | 非空
    pub model_id: i64,
    /// content (varchar(10240)) | 非空
    pub content: String,
    /// reasoning_content (text) | 可空
    pub reasoning_content: Option<String>,
    /// use_context (bit(1)) | 非空
    /// 默认值: b'0'
    pub use_context: Option<bool>,
    /// segment_ids (varchar(2048)) | 可空
    pub segment_ids: Option<String>,
    /// msg_type (int) | 可空
    pub msg_type: Option<i32>,
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
    /// tenant_id (bigint) | 非空
    /// 默认值: 1
    pub tenant_id: Option<i64>,
}
