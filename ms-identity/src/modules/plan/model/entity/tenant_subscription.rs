/// TenantSubscription
///
/// 表名: `tenant_subscription`
/// 主键: `id`
/// 字段数: 12

#[derive(
    Debug,
    Default,
    sqlx::FromRow,
    serde::Serialize,
    serde::Deserialize,
    sqlxplus::ModelMeta,
    sqlxplus::CRUD,
)]
#[model(
    table = "tenant_subscription",
    pk = "id",
    table_comment = "租户套餐订阅表"
)]
pub struct TenantSubscription {
    /// 主键 | id (bigint) | 非空
    #[column(primary_key, auto_increment, comment = "订阅ID")]
    pub id: Option<i64>,
    /// tenant_id (bigint) | 非空
    #[column(not_null, index, comment = "租户ID")]
    pub tenant_id: i64,
    /// plan_id (bigint) | 非空
    #[column(not_null, index, comment = "当前套餐ID")]
    pub plan_id: i64,
    /// status (varchar(32)) | 非空
    #[column(
        not_null,
        length = 32,
        index,
        comment = "订阅状态 active/expired/cancelled/suspended"
    )]
    pub status: String,
    /// start_at (timestamp(3)) | 非空
    #[column(not_null, length = 3, comment = "订阅开始时间")]
    pub start_at: chrono::DateTime<chrono::Utc>,
    /// expire_at (timestamp(3)) | 非空
    #[column(not_null, length = 3, index, comment = "订阅到期时间")]
    pub expire_at: chrono::DateTime<chrono::Utc>,
    /// auto_renew (tinyint(1)) | 非空
    /// 默认值: 0
    #[column(not_null, default = "0", length = 1, comment = "是否自动续费")]
    pub auto_renew: Option<bool>,
    /// created_at (timestamp(3)) | 非空
    /// 默认值: CURRENT_TIMESTAMP(3)
    #[column(
        not_null,
        default = "CURRENT_TIMESTAMP(3)",
        length = 3,
        comment = "创建时间"
    )]
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    /// created_by (bigint) | 可空
    #[column(comment = "创建人")]
    pub created_by: Option<i64>,
    /// updated_at (timestamp(3)) | 非空
    /// 默认值: CURRENT_TIMESTAMP(3)
    #[column(
        not_null,
        default = "CURRENT_TIMESTAMP(3)",
        length = 3,
        comment = "更新时间"
    )]
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
    /// updated_by (bigint) | 可空
    #[column(comment = "更新人")]
    pub updated_by: Option<i64>,
    /// deleted_at (timestamp(3)) | 可空
    #[column(length = 3, comment = "软删除时间")]
    pub deleted_at: Option<chrono::DateTime<chrono::Utc>>,
}
