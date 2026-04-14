/// UserRole
/// 
/// 表名: `user_role`
/// 主键: `id`
/// 字段数: 10

#[derive(Debug, Default, sqlx::FromRow, serde::Serialize, serde::Deserialize, sqlxplus::ModelMeta, sqlxplus::CRUD)]
#[model(table = "user_role", pk = "id", table_comment = "用户角色表")]
pub struct UserRole {
    /// 主键 | id (bigint) | 非空
    #[column(primary_key, auto_increment, comment = "主键ID")]
    pub id: Option<i64>,
    /// user_id (bigint) | 非空
    #[column(not_null, unique, index, comment = "用户ID")]
    pub user_id: i64,
    /// role_id (bigint) | 非空
    #[column(not_null, unique, index, comment = "角色ID")]
    pub role_id: i64,
    /// role_code (varchar(20)) | 非空
    #[column(not_null, length = 20, comment = "角色编码")]
    pub role_code: String,
    /// tenant_id (bigint) | 非空
    #[column(not_null, unique, index, comment = "租户ID")]
    pub tenant_id: i64,
    /// created_at (timestamp(3)) | 非空
    /// 默认值: CURRENT_TIMESTAMP(3)
    #[column(not_null, default = "CURRENT_TIMESTAMP(3)", length = 3, comment = "创建时间")]
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    /// created_by (bigint) | 可空
    #[column(comment = "创建人")]
    pub created_by: Option<i64>,

}
