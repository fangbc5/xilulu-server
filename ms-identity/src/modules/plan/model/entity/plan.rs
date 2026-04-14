/// Plan
///
/// 表名: `plan`
/// 主键: `id`
/// 字段数: 13

#[derive(
    Debug,
    Default,
    Clone,
    sqlx::FromRow,
    serde::Serialize,
    serde::Deserialize,
    sqlxplus::ModelMeta,
    sqlxplus::CRUD,
)]
#[model(table = "plan", pk = "id", soft_delete = "is_del", table_comment = "商业化套餐定义表")]
pub struct Plan {
    /// 主键 | id (bigint) | 非空
    #[column(primary_key, auto_increment, comment = "套餐ID")]
    pub id: Option<i64>,
    /// name (varchar(64)) | 非空
    #[column(
        not_null,
        length = 64,
        unique,
        index,
        comment = "套餐名称，如 Free / Pro / Creator / Starter / Business / Advanced"
    )]
    pub name: String,
    /// type (varchar(32)) | 非空
    #[column(
        not_null,
        length = 32,
        index,
        comment = "套餐类型, personal/enterprise"
    )]
    pub r#type: String,
    /// price (decimal(10,2)) | 非空
    /// 默认值: 0.00
    #[column(not_null, default = "0.00", comment = "价格，单位：元")]
    pub price: Option<String>,
    /// billing_cycle (varchar(32)) | 非空
    #[column(
        not_null,
        length = 32,
        comment = "计费周期 monthly/quarterly/yearly/one_time"
    )]
    pub billing_cycle: String,
    /// description (varchar(255)) | 可空
    #[column(length = 255, comment = "套餐描述")]
    pub description: Option<String>,
    /// is_active (tinyint(1)) | 非空
    /// 默认值: 1
    #[column(not_null, default = "1", length = 1, comment = "是否可售")]
    pub is_active: Option<bool>,
    /// sort_order (int) | 非空
    /// 默认值: 0
    #[column(not_null, default = "0", comment = "排序")]
    pub sort_order: Option<i32>,
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
    /// is_del (tinyint(1)) | 非空
    /// 默认值: 0
    #[column(not_null, default = "0", length = 1, soft_delete, comment = "是否删除")]
    pub is_del: Option<bool>,
}
