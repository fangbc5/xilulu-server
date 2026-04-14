/// TenantUserRel
///
/// 表名: `tenant_user_rel`
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
#[model(table = "tenant_user_rel", pk = "id", table_comment = "租户对应的用户")]
pub struct TenantUserRel {
    /// 主键 | id (bigint) | 非空
    #[column(primary_key, auto_increment, comment = "主键")]
    pub id: Option<i64>,
    /// tenant_id (bigint) | 非空
    #[column(not_null, unique, index, comment = "租户ID")]
    pub tenant_id: i64,
    /// user_id (bigint) | 非空
    #[column(not_null, unique, index, comment = "用户ID")]
    pub user_id: i64,
    /// is_owner (tinyint) | 非空
    /// 默认值: 0
    #[column(not_null, default = "0", comment = "是否租户所有者")]
    pub is_owner: Option<i16>,
    /// status (tinyint) | 非空
    /// 默认值: 1
    #[column(
        not_null,
        default = "1",
        comment = "状态: 0-禁用 1-正常 2-待审核 3-已退出"
    )]
    pub status: Option<i16>,
    /// join_time (timestamp(3)) | 非空
    /// 默认值: CURRENT_TIMESTAMP(3)
    #[column(
        not_null,
        default = "CURRENT_TIMESTAMP(3)",
        length = 3,
        comment = "加入时间"
    )]
    pub join_time: Option<chrono::DateTime<chrono::Utc>>,
    /// leave_time (timestamp(3)) | 可空
    #[column(length = 3, comment = "退出时间")]
    pub leave_time: Option<chrono::DateTime<chrono::Utc>>,
    /// invited_by (bigint) | 可空
    #[column(comment = "邀请人用户ID")]
    pub invited_by: Option<i64>,
    /// created_by (bigint) | 可空
    #[column(comment = "创建人")]
    pub created_by: Option<i64>,
    /// created_time (timestamp(3)) | 非空
    /// 默认值: CURRENT_TIMESTAMP(3)
    #[column(not_null, default = "CURRENT_TIMESTAMP(3)", length = 3, comment = "")]
    pub created_time: Option<chrono::DateTime<chrono::Utc>>,
    /// updated_time (timestamp(3)) | 非空
    /// 默认值: CURRENT_TIMESTAMP(3)
    #[column(not_null, default = "CURRENT_TIMESTAMP(3)", length = 3, comment = "")]
    pub updated_time: Option<chrono::DateTime<chrono::Utc>>,
}
