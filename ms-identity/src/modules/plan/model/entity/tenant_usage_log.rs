/// TenantUsageLog
/// 
/// 表名: `tenant_usage_log`
/// 主键: `id`
/// 字段数: 7

#[derive(Debug, Default, sqlx::FromRow, serde::Serialize, serde::Deserialize, sqlxplus::ModelMeta, sqlxplus::CRUD)]
#[model(table = "tenant_usage_log", pk = "id", table_comment = "租户用量明细日志")]
pub struct TenantUsageLog {
    /// 主键 | id (bigint) | 非空
    #[column(primary_key, auto_increment, comment = "")]
    pub id: Option<i64>,
    /// tenant_id (bigint) | 非空
    #[column(not_null, index, comment = "")]
    pub tenant_id: i64,
    /// entitlement_key (varchar(64)) | 非空
    #[column(not_null, length = 64, index, comment = "")]
    pub entitlement_key: String,
    /// delta (bigint) | 非空
    #[column(not_null, comment = "本次消耗量")]
    pub delta: i64,
    /// source (varchar(64)) | 非空
    #[column(not_null, length = 64, comment = "来源：api / job / import")]
    pub source: String,
    /// ref_id (varchar(128)) | 可空
    #[column(length = 128, comment = "业务ID")]
    pub ref_id: Option<String>,
    /// created_at (timestamp(3)) | 非空
    /// 默认值: CURRENT_TIMESTAMP(3)
    #[column(not_null, default = "CURRENT_TIMESTAMP(3)", length = 3, index, comment = "")]
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}
