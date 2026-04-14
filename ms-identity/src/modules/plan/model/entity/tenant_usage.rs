/// TenantUsage
/// 
/// 表名: `tenant_usage`
/// 主键: `id`
/// 字段数: 10

#[derive(Debug, Default, sqlx::FromRow, serde::Serialize, serde::Deserialize, sqlxplus::ModelMeta, sqlxplus::CRUD)]
#[model(table = "tenant_usage", pk = "id", table_comment = "租户套餐用量统计表")]
pub struct TenantUsage {
    /// 主键 | id (bigint) | 非空
    #[column(primary_key, auto_increment, comment = "")]
    pub id: Option<i64>,
    /// tenant_id (bigint) | 非空
    #[column(not_null, unique, index, comment = "租户ID")]
    pub tenant_id: i64,
    /// plan_id (bigint) | 非空
    #[column(not_null, comment = "套餐ID")]
    pub plan_id: i64,
    /// entitlement_key (varchar(64)) | 非空
    #[column(not_null, length = 64, unique, index, comment = "用量项标识，如 api_calls / doc_count")]
    pub entitlement_key: String,
    /// cycle_type (varchar(32)) | 非空
    #[column(not_null, length = 32, comment = "monthly / quarterly / yearly")]
    pub cycle_type: String,
    /// cycle_start (date) | 非空
    #[column(not_null, unique, index, comment = "周期开始")]
    pub cycle_start: chrono::NaiveDate,
    /// cycle_end (date) | 非空
    #[column(not_null, comment = "周期结束")]
    pub cycle_end: chrono::NaiveDate,
    /// used_value (bigint) | 非空
    /// 默认值: 0
    #[column(not_null, default = "0", comment = "已使用量")]
    pub used_value: Option<i64>,
    /// created_at (timestamp(3)) | 非空
    /// 默认值: CURRENT_TIMESTAMP(3)
    #[column(not_null, default = "CURRENT_TIMESTAMP(3)", length = 3, comment = "")]
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    /// updated_at (timestamp(3)) | 非空
    /// 默认值: CURRENT_TIMESTAMP(3)
    #[column(not_null, default = "CURRENT_TIMESTAMP(3)", length = 3, comment = "")]
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}
