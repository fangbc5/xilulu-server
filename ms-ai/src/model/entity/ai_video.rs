/// AiVideo
/// 
/// 表名: `ai_video`
/// 主键: `id`
/// 逻辑删除字段: `deleted`
/// 字段数: 25

#[derive(Debug, Default, sqlx::FromRow, serde::Serialize, serde::Deserialize, sqlxplus::ModelMeta, sqlxplus::CRUD)]
#[model(table = "ai_video", pk = "id", soft_delete = "deleted")]
pub struct AiVideo {
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
    /// width (int) | 可空
    pub width: Option<i32>,
    /// height (int) | 可空
    pub height: Option<i32>,
    /// duration (int) | 可空
    pub duration: Option<i32>,
    /// status (int) | 非空
    pub status: i32,
    /// finish_time (datetime) | 可空
    pub finish_time: Option<chrono::NaiveDateTime>,
    /// error_message (varchar(500)) | 可空
    pub error_message: Option<String>,
    /// video_url (varchar(500)) | 可空
    pub video_url: Option<String>,
    /// cover_url (varchar(500)) | 可空
    pub cover_url: Option<String>,
    /// chat_message_id (bigint) | 可空
    pub chat_message_id: Option<i64>,
    /// conversation_id (bigint) | 可空
    pub conversation_id: Option<i64>,
    /// public_status (bit(1)) | 非空
    /// 默认值: b'0'
    pub public_status: Option<bool>,
    /// options (json) | 可空
    pub options: Option<serde_json::Value>,
    /// task_id (varchar(100)) | 可空
    pub task_id: Option<String>,
    /// tenant_id (bigint) | 非空
    /// 默认值: 1
    pub tenant_id: Option<i64>,
    /// creator (varchar(64)) | 可空
    /// 默认值: 
    pub creator: Option<String>,
    /// create_time (datetime(3)) | 非空
    /// 默认值: CURRENT_TIMESTAMP(3)
    pub create_time: Option<chrono::NaiveDateTime>,
    /// updater (bigint) | 可空
    pub updater: Option<i64>,
    /// update_time (datetime(3)) | 非空
    /// 默认值: CURRENT_TIMESTAMP(3)
    pub update_time: Option<chrono::NaiveDateTime>,
    /// deleted (tinyint(1)) | 非空
    /// 默认值: 0
    pub deleted: Option<i16>,
}
