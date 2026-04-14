/// AiMindMap
/// 
/// 表名: `ai_mind_map`
/// 主键: `id`
/// 逻辑删除字段: `deleted`
/// 字段数: 14

#[derive(Debug, Default, sqlx::FromRow, serde::Serialize, serde::Deserialize, sqlxplus::ModelMeta, sqlxplus::CRUD)]
#[model(table = "ai_mind_map", pk = "id", soft_delete = "deleted")]
pub struct AiMindMap {
    /// 主键 | id (bigint) | 非空
    pub id: Option<i64>,
    /// user_id (bigint) | 非空
    pub user_id: i64,
    /// prompt (text) | 非空
    pub prompt: String,
    /// generated_content (text) | 可空
    pub generated_content: Option<String>,
    /// platform (varchar(64)) | 非空
    pub platform: String,
    /// model_id (bigint) | 非空
    pub model_id: i64,
    /// model (varchar(50)) | 非空
    pub model: String,
    /// error_message (varchar(1024)) | 可空
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
