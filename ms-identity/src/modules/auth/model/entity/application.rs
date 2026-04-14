/// Application
///
/// 表名: `application`
/// 主键: `id`
/// 逻辑删除字段: `is_del`
/// 字段数: 19

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
    table = "application",
    pk = "id",
    soft_delete = "is_del",
    table_comment = "应用"
)]
pub struct Application {
    /// 主键 | id (bigint) | 非空
    #[column(primary_key, auto_increment, comment = "ID")]
    pub id: Option<i64>,
    /// app_key (varchar(255)) | 可空
    /// 默认值:
    #[column(length = 255, unique, index, comment = "应用标识")]
    pub app_key: Option<String>,
    /// app_secret (varchar(255)) | 可空
    /// 默认值:
    #[column(length = 255, comment = "应用秘钥")]
    pub app_secret: Option<String>,
    /// name (varchar(255)) | 非空
    /// 默认值:
    #[column(not_null, length = 255, comment = "应用名称")]
    pub name: Option<String>,
    /// version (varchar(255)) | 可空
    /// 默认值:
    #[column(length = 255, comment = "版本")]
    pub version: Option<String>,
    /// type (char(2)) | 非空
    /// 默认值: 10
    /// 注意：Rust 字段名使用 r#type（因为 type 是关键字），但数据库列名是 type
    #[column(
        not_null,
        default = "10",
        length = 2,
        comment = "应用类型;[10-自建应用 20-第三方应用]@Echo(api = EchoApi.DICTIONARY_ITEM_FEIGN_CLASS, dictType = EchoDictType.System.APPLICATION_TYPE)"
    )]
    pub r#type: Option<String>,
    /// redirect (varchar(255)) | 可空
    #[column(length = 255, comment = "重定向地址")]
    pub redirect: Option<String>,
    /// introduce (varchar(255)) | 可空
    /// 默认值:
    #[column(length = 255, comment = "简介")]
    pub introduce: Option<String>,
    /// remark (varchar(255)) | 可空
    /// 默认值:
    #[column(length = 255, comment = "备注")]
    pub remark: Option<String>,
    /// url (varchar(255)) | 可空
    /// 默认值:
    #[column(length = 255, comment = "应用地址")]
    pub url: Option<String>,
    /// is_general (bit(1)) | 可空
    /// 默认值: b'0'
    #[column(default = "b\'0\'", length = 1, comment = "是否公共应用;0-否 1-是")]
    pub is_general: Option<bool>,
    /// is_visible (bit(1)) | 可空
    /// 默认值: b'1'
    #[column(default = "b\'1\'", length = 1, comment = "是否可见;0-否 1-是")]
    pub is_visible: Option<bool>,
    /// sort_value (int) | 可空
    /// 默认值: 1
    #[column(default = "1", comment = "排序")]
    pub sort_value: Option<i32>,
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
    /// is_del (tinyint(1)) | 非空
    /// 默认值: 0
    #[column(not_null, default = "0", length = 1, soft_delete, comment = "")]
    pub is_del: Option<bool>,
}
