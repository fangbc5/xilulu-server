/// AiPlatform
/// 
/// 表名: `ai_platform`
/// 主键: `id`
/// 逻辑删除字段: `deleted`
/// 字段数: 15

#[derive(Debug, Default, sqlx::FromRow, serde::Serialize, serde::Deserialize, sqlxplus::ModelMeta, sqlxplus::CRUD)]
#[model(table = "ai_platform", pk = "id", soft_delete = "deleted")]
pub struct AiPlatform {
    /// 主键 | id (bigint) | 非空
    pub id: Option<i64>,
    /// platform (varchar(50)) | 非空
    pub platform: String,
    /// name (varchar(100)) | 非空
    pub name: String,
    /// label (varchar(100)) | 非空
    pub label: String,
    /// examples (text) | 可空
    pub examples: Option<String>,
    /// docs (varchar(500)) | 可空
    pub docs: Option<String>,
    /// hint (varchar(500)) | 可空
    pub hint: Option<String>,
    /// sort (int) | 非空
    /// 默认值: 0
    pub sort: Option<i32>,
    /// status (tinyint) | 非空
    /// 默认值: 0
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
