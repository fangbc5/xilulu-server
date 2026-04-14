/// AiImage
/// 
/// 表名: `ai_image`
/// 主键: `id`
/// 逻辑删除字段: `deleted`
/// 字段数: 24

#[derive(Debug, Default, sqlx::FromRow, serde::Serialize, serde::Deserialize, sqlxplus::ModelMeta, sqlxplus::CRUD)]
#[model(table = "ai_image", pk = "id", soft_delete = "deleted")]
pub struct AiImage {
    /// 主键 | id (bigint) | 非空
    pub id: Option<i64>,
    /// user_id (bigint) | 非空
    pub user_id: i64,
    /// prompt (varchar(2000)) | 非空
    pub prompt: String,
    /// platform (varchar(64)) | 非空
    pub platform: String,
    /// model_id (bigint) | 可空
    pub model_id: Option<i64>,
    /// model (varchar(64)) | 非空
    pub model: String,
    /// width (int) | 非空
    pub width: i32,
    /// height (int) | 非空
    pub height: i32,
    /// status (tinyint) | 非空
    pub status: i16,
    /// finish_time (datetime) | 可空
    pub finish_time: Option<chrono::NaiveDateTime>,
    /// error_message (varchar(1024)) | 可空
    pub error_message: Option<String>,
    /// public_status (bit(1)) | 非空
    /// 默认值: b'0'
    pub public_status: Option<bool>,
    /// chat_message_id (bigint) | 可空
    pub chat_message_id: Option<i64>,
    /// conversation_id (bigint) | 可空
    pub conversation_id: Option<i64>,
    /// pic_url (varchar(2048)) | 可空
    pub pic_url: Option<String>,
    /// options (json) | 可空
    pub options: Option<serde_json::Value>,
    /// task_id (varchar(1024)) | 可空
    pub task_id: Option<String>,
    /// buttons (varchar(2048)) | 可空
    pub buttons: Option<String>,
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
