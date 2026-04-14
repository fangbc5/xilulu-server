/// PlanEntitlement
/// 
/// 表名: `plan_entitlement`
/// 主键: `id`
/// 字段数: 11

#[derive(Debug, Default, sqlx::FromRow, serde::Serialize, serde::Deserialize, sqlxplus::ModelMeta, sqlxplus::CRUD)]
#[model(table = "plan_entitlement", pk = "id", soft_delete = "is_del", table_comment = "套餐权益表")]
pub struct PlanEntitlement {
    /// 主键 | id (bigint) | 非空
    #[column(primary_key, auto_increment, comment = "权益ID")]
    pub id: Option<i64>,
    /// plan_id (bigint) | 非空
    #[column(not_null, unique, index, comment = "套餐ID")]
    pub plan_id: i64,
    /// entitlement_key (varchar(64)) | 非空
    #[column(not_null, length = 64, unique, index, comment = "权益key，如 max_user / enable_audit")]
    pub entitlement_key: String,
    /// entitlement_value (varchar(128)) | 非空
    #[column(not_null, length = 128, comment = "权益值，如 10 / true / advanced")]
    pub entitlement_value: String,
    /// value_type (varchar(32)) | 非空
    #[column(not_null, length = 32, comment = "权益类型 limit/boolean/enum")]
    pub value_type: String,
    /// description (varchar(255)) | 可空
    #[column(length = 255, comment = "权益说明")]
    pub description: Option<String>,
    /// created_at (timestamp(3)) | 非空
    /// 默认值: CURRENT_TIMESTAMP(3)
    #[column(not_null, default = "CURRENT_TIMESTAMP(3)", length = 3, comment = "创建时间")]
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    /// created_by (bigint) | 可空
    #[column(comment = "创建人")]
    pub created_by: Option<i64>,
    /// updated_at (timestamp(3)) | 非空
    /// 默认值: CURRENT_TIMESTAMP(3)
    #[column(not_null, default = "CURRENT_TIMESTAMP(3)", length = 3, comment = "更新时间")]
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
    /// updated_by (bigint) | 可空
    #[column(comment = "更新人")]
    pub updated_by: Option<i64>,
    /// is_del (tinyint(1)) | 非空
    /// 默认值: 0
    #[column(not_null, default = "0", length = 1, soft_delete, comment = "是否删除")]
    pub is_del: Option<bool>,
}
