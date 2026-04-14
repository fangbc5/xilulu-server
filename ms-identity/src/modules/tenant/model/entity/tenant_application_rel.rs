/// TenantApplicationRel
/// 
/// 表名: `tenant_application_rel`
/// 主键: `id`
/// 字段数: 8

#[derive(Debug, Default, sqlx::FromRow, serde::Serialize, serde::Deserialize, sqlxplus::ModelMeta, sqlxplus::CRUD)]
#[model(table = "tenant_application_rel", pk = "id", table_comment = "租户的应用")]
pub struct TenantApplicationRel {
    /// 主键 | id (bigint) | 非空
    #[column(primary_key, auto_increment, comment = "ID")]
    pub id: Option<i64>,
    /// tenant_id (bigint) | 非空
    #[column(not_null, unique, index, comment = "租户ID")]
    pub tenant_id: i64,
    /// application_id (bigint) | 非空
    #[column(not_null, unique, index, comment = "应用ID")]
    pub application_id: i64,
    /// expiration_time (timestamp(3)) | 可空
    #[column(length = 3, comment = "过期时间")]
    pub expiration_time: Option<chrono::DateTime<chrono::Utc>>,
    /// create_by (bigint) | 可空
    #[column(comment = "创建人")]
    pub create_by: Option<i64>,
    /// create_time (timestamp(3)) | 可空
    /// 默认值: CURRENT_TIMESTAMP(3)
    #[column(default = "CURRENT_TIMESTAMP(3)", length = 3, comment = "创建时间")]
    pub create_time: Option<chrono::DateTime<chrono::Utc>>,
    /// update_by (bigint) | 可空
    #[column(comment = "最后更新人")]
    pub update_by: Option<i64>,
    /// update_time (timestamp(3)) | 可空
    /// 默认值: CURRENT_TIMESTAMP(3)
    #[column(default = "CURRENT_TIMESTAMP(3)", length = 3, comment = "最后更新时间")]
    pub update_time: Option<chrono::DateTime<chrono::Utc>>,
}
