/// AiChatRole
/// 
/// 表名: `ai_chat_role`
/// 主键: `id`
/// 逻辑删除字段: `deleted`
/// 字段数: 19

#[derive(Debug, Default, sqlx::FromRow, serde::Serialize, serde::Deserialize, sqlxplus::ModelMeta, sqlxplus::CRUD)]
#[model(table = "ai_chat_role", pk = "id", soft_delete = "deleted")]
pub struct AiChatRole {
    /// 主键 | id (bigint) | 非空
    pub id: Option<i64>,
    /// user_id (bigint) | 可空
    pub user_id: Option<i64>,
    /// model_id (bigint) | 可空
    pub model_id: Option<i64>,
    /// name (varchar(128)) | 非空
    pub name: String,
    /// avatar (varchar(256)) | 非空
    pub avatar: String,
    /// category (varchar(32)) | 可空
    pub category: Option<String>,
    /// sort (int) | 非空
    /// 默认值: 0
    pub sort: Option<i32>,
    /// description (varchar(256)) | 非空
    pub description: String,
    /// system_message (varchar(1024)) | 可空
    pub system_message: Option<String>,
    /// knowledge_ids (varchar(256)) | 可空
    pub knowledge_ids: Option<String>,
    /// tool_ids (varchar(256)) | 可空
    pub tool_ids: Option<String>,
    /// public_status (bit(1)) | 非空
    pub public_status: bool,
    /// status (tinyint) | 可空
    pub status: Option<i16>,
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
