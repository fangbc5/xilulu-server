/// AiKnowledge
/// 
/// 表名: `ai_knowledge`
/// 主键: `id`
/// 逻辑删除字段: `deleted`
/// 字段数: 14

#[derive(Debug, Default, sqlx::FromRow, serde::Serialize, serde::Deserialize, sqlxplus::ModelMeta, sqlxplus::CRUD)]
#[model(table = "ai_knowledge", pk = "id", soft_delete = "deleted")]
pub struct AiKnowledge {
    /// 主键 | id (bigint) | 非空
    pub id: Option<i64>,
    /// name (varchar(255)) | 非空
    pub name: String,
    /// description (longtext) | 可空
    pub description: Option<String>,
    /// embedding_model_id (bigint) | 非空
    pub embedding_model_id: i64,
    /// embedding_model (varchar(32)) | 非空
    pub embedding_model: String,
    /// top_k (int) | 非空
    pub top_k: i32,
    /// similarity_threshold (double) | 非空
    pub similarity_threshold: f64,
    /// status (tinyint) | 非空
    pub status: i16,
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
