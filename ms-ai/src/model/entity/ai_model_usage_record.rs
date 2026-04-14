/// AiModelUsageRecord
/// 
/// 表名: `ai_model_usage_record`
/// 主键: `id`
/// 逻辑删除字段: `deleted`
/// 字段数: 9

#[derive(Debug, Default, sqlx::FromRow, serde::Serialize, serde::Deserialize, sqlxplus::ModelMeta, sqlxplus::CRUD)]
#[model(table = "ai_model_usage_record", pk = "id", soft_delete = "deleted")]
pub struct AiModelUsageRecord {
    /// 主键 | id (bigint) | 非空
    pub id: Option<i64>,
    /// user_id (bigint) | 非空
    pub user_id: i64,
    /// model_id (bigint) | 非空
    pub model_id: i64,
    /// usage_count (int) | 非空
    /// 默认值: 0
    pub usage_count: Option<i32>,
    /// remaining_count (int) | 非空
    /// 默认值: 10
    pub remaining_count: Option<i32>,
    /// create_time (datetime) | 非空
    /// 默认值: CURRENT_TIMESTAMP
    pub create_time: Option<chrono::NaiveDateTime>,
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
