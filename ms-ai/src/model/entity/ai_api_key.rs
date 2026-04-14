/// AiApiKey
/// 
/// 表名: `ai_api_key`
/// 主键: `id`
/// 逻辑删除字段: `deleted`
/// 字段数: 14

#[derive(Debug, Default, sqlx::FromRow, serde::Serialize, serde::Deserialize, sqlxplus::ModelMeta, sqlxplus::CRUD)]
#[model(table = "ai_api_key", pk = "id", soft_delete = "deleted")]
pub struct AiApiKey {
    /// 主键 | id (bigint) | 非空
    pub id: Option<i64>,
    /// user_id (bigint) | 非空
    pub user_id: i64,
    /// public_status (bit(1)) | 非空
    /// 默认值: b'1'
    pub public_status: Option<bool>,
    /// name (varchar(255)) | 非空
    pub name: String,
    /// api_key (varchar(1024)) | 非空
    pub api_key: String,
    /// platform (varchar(255)) | 非空
    pub platform: String,
    /// url (varchar(255)) | 可空
    pub url: Option<String>,
    /// status (int) | 非空
    pub status: i32,
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
