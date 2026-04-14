/// AiAudio
/// 
/// 表名: `ai_audio`
/// 主键: `id`
/// 逻辑删除字段: `deleted`
/// 字段数: 21

#[derive(Debug, Default, sqlx::FromRow, serde::Serialize, serde::Deserialize, sqlxplus::ModelMeta, sqlxplus::CRUD)]
#[model(table = "ai_audio", pk = "id", soft_delete = "deleted")]
pub struct AiAudio {
    /// 主键 | id (bigint) | 非空
    pub id: Option<i64>,
    /// user_id (bigint) | 非空
    pub user_id: i64,
    /// prompt (varchar(2000)) | 非空
    pub prompt: String,
    /// platform (varchar(50)) | 非空
    pub platform: String,
    /// model_id (bigint) | 非空
    pub model_id: i64,
    /// model (varchar(100)) | 非空
    pub model: String,
    /// status (int) | 非空
    pub status: i32,
    /// finish_time (datetime) | 可空
    pub finish_time: Option<chrono::NaiveDateTime>,
    /// error_message (varchar(500)) | 可空
    pub error_message: Option<String>,
    /// audio_url (varchar(500)) | 可空
    pub audio_url: Option<String>,
    /// public_status (bit(1)) | 非空
    /// 默认值: b'0'
    pub public_status: Option<bool>,
    /// options (json) | 可空
    pub options: Option<serde_json::Value>,
    /// task_id (varchar(100)) | 可空
    pub task_id: Option<String>,
    /// conversation_id (bigint) | 可空
    pub conversation_id: Option<i64>,
    /// chat_message_id (bigint) | 可空
    pub chat_message_id: Option<i64>,
    /// create_time (datetime(3)) | 非空
    /// 默认值: CURRENT_TIMESTAMP(3)
    pub create_time: Option<chrono::NaiveDateTime>,
    /// update_time (datetime(3)) | 非空
    /// 默认值: CURRENT_TIMESTAMP(3)
    pub update_time: Option<chrono::NaiveDateTime>,
    /// creator (bigint) | 可空
    pub creator: Option<i64>,
    /// updater (bigint) | 可空
    pub updater: Option<i64>,
    /// deleted (bit(1)) | 非空
    /// 默认值: b'0'
    pub deleted: Option<bool>,
    /// tenant_id (bigint) | 非空
    /// 默认值: 1
    pub tenant_id: Option<i64>,
}
