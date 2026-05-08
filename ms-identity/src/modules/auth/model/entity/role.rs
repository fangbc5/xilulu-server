/// Role
/// 
/// 表名: `role`
/// 主键: `id`
/// 逻辑删除字段: `is_del`
/// 字段数: 15

#[derive(Debug, Default, sqlx::FromRow, serde::Serialize, serde::Deserialize, sqlxplus::ModelMeta, sqlxplus::CRUD)]
#[model(table = "role", pk = "id", soft_delete = "is_del", table_comment = "角色")]
pub struct Role {
    /// 主键 | id (bigint) | 非空
    #[column(primary_key, auto_increment, comment = "ID")]
    pub id: Option<i64>,
    /// category (char(2)) | 非空
    /// 默认值: 10
    #[column(not_null, default = "10", length = 2, comment = "角色类别;[10-功能角色 20-桌面角色 30-数据角色]@Echo(api = EchoApi.DICTIONARY_ITEM_FEIGN_CLASS, dictType = EchoDictType.Base.ROLE_CATEGORY)")]
    pub category: Option<String>,
    /// type_ (char(2)) | 非空
    /// 默认值: 20
    #[column(not_null, default = "20", length = 2, comment = "角色类型;[10-系统角色 20-自定义角色]; 
@Echo(api = EchoApi.DICTIONARY_ITEM_FEIGN_CLASS, dictType = EchoDictType.Global.DATA_TYPE)")]
    pub type_: Option<String>,
    /// name (varchar(50)) | 非空
    #[column(not_null, length = 50, comment = "名称")]
    pub name: String,
    /// code (varchar(20)) | 非空
    #[column(not_null, length = 20, unique, index, comment = "编码")]
    pub code: String,
    /// remarks (varchar(255)) | 可空
    #[column(length = 255, comment = "备注")]
    pub remarks: Option<String>,
    /// state (bit(1)) | 可空
    /// 默认值: b'1'
    #[column(default = "b\'1\'", length = 1, comment = "状态")]
    pub state: Option<bool>,
    /// readonly_ (bit(1)) | 可空
    /// 默认值: b'0'
    #[column(default = "b\'0\'", length = 1, comment = "内置角色")]
    pub readonly_: Option<bool>,
    /// create_by (bigint) | 可空
    #[column(comment = "创建人")]
    pub create_by: Option<i64>,
    /// create_time (timestamp(3)) | 可空
    /// 默认值: CURRENT_TIMESTAMP(3)
    #[column(default = "CURRENT_TIMESTAMP(3)", length = 3, comment = "创建时间")]
    pub create_time: Option<chrono::DateTime<chrono::Utc>>,
    /// update_by (bigint) | 可空
    #[column(comment = "更新人")]
    pub update_by: Option<i64>,
    /// update_time (timestamp(3)) | 可空
    /// 默认值: CURRENT_TIMESTAMP(3)
    #[column(default = "CURRENT_TIMESTAMP(3)", length = 3, comment = "更新时间")]
    pub update_time: Option<chrono::DateTime<chrono::Utc>>,
    /// created_org_id (bigint) | 可空
    #[column(comment = "创建人组织")]
    pub created_org_id: Option<i64>,
    /// is_del (tinyint(1)) | 非空
    /// 默认值: 0
    #[column(not_null, default = "0", length = 1, soft_delete, comment = "")]
    pub is_del: Option<bool>,
    /// tenant_id (bigint) | 非空
    #[column(not_null, comment = "")]
    pub tenant_id: i64,
    /// biz_id (bigint) | 可空
    #[column(comment = "业务关联ID（组织角色=org_id）")]
    pub biz_id: Option<i64>,
}
