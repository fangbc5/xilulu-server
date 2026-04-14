/// AiModel
/// 
/// 表名: `ai_model`
/// 主键: `id`
/// 逻辑删除字段: `deleted`
/// 字段数: 20

#[derive(Debug, Default, sqlx::FromRow, serde::Serialize, serde::Deserialize, sqlxplus::ModelMeta, sqlxplus::CRUD)]
#[model(table = "ai_model", pk = "id", soft_delete = "deleted")]
pub struct AiModel {
    /// 主键 | id (bigint) | 非空
    pub id: Option<i64>,
    /// user_id (bigint) | 非空
    pub user_id: i64,
    /// key_id (bigint) | 非空
    pub key_id: i64,
    /// name (varchar(64)) | 非空
    pub name: String,
    /// avatar (varchar(500)) | 可空
    pub avatar: Option<String>,
    /// model (varchar(64)) | 非空
    pub model: String,
    /// platform (varchar(32)) | 非空
    pub platform: String,
    /// type (tinyint) | 非空
    pub r#type: i16,
    /// sort (int) | 非空
    pub sort: i32,
    /// status (tinyint) | 非空
    pub status: i16,
    /// public_status (bit(1)) | 非空
    /// 默认值: b'1'
    pub public_status: Option<bool>,
    /// temperature (double) | 可空
    pub temperature: Option<f64>,
    /// max_tokens (int) | 可空
    pub max_tokens: Option<i32>,
    /// max_contexts (int) | 可空
    pub max_contexts: Option<i32>,
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
