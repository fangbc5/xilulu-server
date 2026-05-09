/// Tenant
/// 
/// 表名: `tenant`
/// 主键: `id`
/// 逻辑删除字段: `is_del`
/// 字段数: 16

#[derive(Debug, Default, sqlx::FromRow, serde::Serialize, serde::Deserialize, sqlxplus::ModelMeta, sqlxplus::CRUD)]
#[model(table = "tenant", pk = "id", soft_delete = "is_del", table_comment = "租户表")]
pub struct Tenant {
    /// 主键 | id (bigint) | 非空
    #[column(primary_key, auto_increment, comment = "租户编号")]
    pub id: Option<i64>,
    /// name (varchar(30)) | 非空
    #[column(not_null, length = 30, comment = "租户名")]
    pub name: String,
    /// contact_user_id (bigint) | 可空
    #[column(comment = "联系人的用户编号")]
    pub contact_user_id: Option<i64>,
    /// contact_name (varchar(30)) | 非空
    #[column(not_null, length = 30, comment = "联系人")]
    pub contact_name: String,
    /// contact_mobile (varchar(500)) | 可空
    #[column(length = 500, comment = "联系手机")]
    pub contact_mobile: Option<String>,
    /// pid (bigint) | 非空
    #[column(not_null, default = 0, comment = "父租户id")]
    pub pid: i64,
    /// status (tinyint) | 非空
    /// 默认值: 0
    #[column(not_null, default = "0", comment = "租户状态（0正常 1停用）")]
    pub status: Option<i16>,
    /// website (varchar(256)) | 可空
    /// 默认值: 
    #[column(length = 256, comment = "绑定域名")]
    pub website: Option<String>,
    /// package_id (bigint) | 非空
    #[column(not_null, comment = "租户套餐编号")]
    pub package_id: i64,
    /// expire_time (timestamp(3)) | 非空
    #[column(not_null, length = 3, comment = "过期时间")]
    pub expire_time: chrono::DateTime<chrono::Utc>,
    /// account_count (int) | 非空
    #[column(not_null, comment = "账号数量")]
    pub account_count: i32,
    /// create_by (bigint) | 可空
    #[column(comment = "创建人id")]
    pub create_by: Option<i64>,
    /// create_time (timestamp(3)) | 可空
    /// 默认值: CURRENT_TIMESTAMP(3)
    #[column(default = "CURRENT_TIMESTAMP(3)", length = 3, comment = "创建时间")]
    pub create_time: Option<chrono::DateTime<chrono::Utc>>,
    /// update_by (bigint) | 可空
    #[column(comment = "更新人id")]
    pub update_by: Option<i64>,
    /// update_time (timestamp(3)) | 可空
    /// 默认值: CURRENT_TIMESTAMP(3)
    #[column(default = "CURRENT_TIMESTAMP(3)", length = 3, comment = "更新时间")]
    pub update_time: Option<chrono::DateTime<chrono::Utc>>,
    /// tenant_type (tinyint) | 非空
    /// 默认值: 1
    /// 租户类型: 1-个人租户, 2-团队租户
    #[column(not_null, default = "1", comment = "租户类型: 1-个人租户, 2-团队租户")]
    pub tenant_type: Option<i16>,
    /// is_del (tinyint(1)) | 非空
    /// 默认值: 0
    #[column(not_null, default = "0", length = 1, soft_delete, comment = "")]
    pub is_del: Option<bool>,
}
