/// AiWrite
/// 
/// 表名: `ai_write`
/// 主键: `id`
/// 逻辑删除字段: `deleted`
/// 字段数: 20

#[derive(Debug, Default, sqlx::FromRow, serde::Serialize, serde::Deserialize, sqlxplus::ModelMeta, sqlxplus::CRUD)]
#[model(table = "ai_write", pk = "id", soft_delete = "deleted")]
pub struct AiWrite {
    /// 主键 | id (bigint) | 非空
    pub id: Option<i64>,
    /// user_id (bigint) | 非空
    pub user_id: i64,
    /// type (int) | 可空
    pub r#type: Option<i32>,
    /// platform (varchar(255)) | 非空
    pub platform: String,
    /// model_id (bigint) | 非空
    pub model_id: i64,
    /// model (varchar(255)) | 非空
    pub model: String,
    /// prompt (varchar(512)) | 非空
    pub prompt: String,
    /// generated_content (varchar(5120)) | 可空
    pub generated_content: Option<String>,
    /// original_content (varchar(5120)) | 可空
    pub original_content: Option<String>,
    /// length (tinyint) | 可空
    pub length: Option<i16>,
    /// format (tinyint) | 可空
    pub format: Option<i16>,
    /// tone (tinyint) | 可空
    pub tone: Option<i16>,
    /// language (tinyint) | 可空
    pub language: Option<i16>,
    /// error_message (varchar(255)) | 可空
    pub error_message: Option<String>,
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
