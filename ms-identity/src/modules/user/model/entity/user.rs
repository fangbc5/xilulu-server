/// User
///
/// 表名: `user`
/// 主键: `id`
/// 逻辑删除字段: `is_del`
/// 字段数: 36

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
    table = "user",
    pk = "id",
    soft_delete = "is_del",
    table_comment = "用户表"
)]
pub struct User {
    /// 主键 | id (bigint) | 非空
    #[column(primary_key, auto_increment, comment = "主键ID")]
    pub id: Option<i64>,
    /// system_type (tinyint) | 非空
    /// 默认值: 1
    #[column(
        not_null,
        default = "1",
        unique,
        index,
        comment = "系统类型: 1-后台登录; 2-IM系统登录"
    )]
    pub system_type: Option<i16>,
    /// user_type (tinyint) | 可空
    /// 默认值: 3
    #[column(default = "3", comment = "用户类型: 1-系统用户,2-机器人,3-普通用户")]
    pub user_type: Option<i16>,
    /// username (varchar(255)) | 可空
    #[column(length = 255, unique, index, comment = "用户名（认证系统用）")]
    pub username: Option<String>,
    /// nick_name (varchar(255)) | 可空
    #[column(length = 255, comment = "昵称/长名")]
    pub nick_name: Option<String>,
    /// real_name (varchar(255)) | 可空
    #[column(length = 255, comment = "真实姓名")]
    pub real_name: Option<String>,
    /// avatar (varchar(255)) | 非空
    /// 默认值:
    #[column(not_null, length = 255, comment = "头像")]
    pub avatar: Option<String>,
    /// avatar_update_time (timestamp(3)) | 可空
    #[column(length = 3, comment = "头像修改时间")]
    pub avatar_update_time: Option<chrono::DateTime<chrono::Utc>>,
    /// email (varchar(255)) | 可空
    #[column(length = 255, unique, index, comment = "邮箱")]
    pub email: Option<String>,
    /// region (varchar(5)) | 可空
    #[column(length = 5, comment = "国家码")]
    pub region: Option<String>,
    /// mobile (varchar(11)) | 可空
    #[column(length = 11, unique, index, comment = "手机号")]
    pub mobile: Option<String>,
    /// id_card (varchar(18)) | 可空
    #[column(length = 18, unique, index, comment = "身份证")]
    pub id_card: Option<String>,
    /// wx_open_id (varchar(255)) | 可空
    #[column(length = 255, comment = "微信OpenId")]
    pub wx_open_id: Option<String>,
    /// dd_open_id (varchar(255)) | 可空
    #[column(length = 255, comment = "钉钉OpenId")]
    pub dd_open_id: Option<String>,
    /// sex (tinyint) | 可空
    /// 默认值: 0
    #[column(default = "0", comment = "性别 1-男 2-女 3-未知")]
    pub sex: Option<i16>,
    /// state (tinyint) | 可空
    /// 默认值: 1
    #[column(default = "1", comment = "状态: 0-禁用/拉黑, 1-启用/正常")]
    pub state: Option<i16>,
    /// user_state_id (bigint) | 可空
    #[column(comment = "用户状态ID (IM用)")]
    pub user_state_id: Option<i64>,
    /// resume (varchar(200)) | 可空
    #[column(length = 200, comment = "个人简介")]
    pub resume: Option<String>,
    /// work_describe (varchar(255)) | 可空
    #[column(length = 255, comment = "工作描述")]
    pub work_describe: Option<String>,
    /// item_id (bigint) | 可空
    #[column(comment = "徽章ID")]
    pub item_id: Option<i64>,
    /// context (tinyint) | 可空
    /// 默认值: 0
    #[column(default = "0", comment = "AI上下文开关")]
    pub context: Option<i16>,
    /// num (bigint) | 可空
    /// 默认值: 10
    #[column(default = "10", comment = "AI模块相关字段")]
    pub num: Option<i64>,
    /// password (varchar(128)) | 非空
    /// 默认值:
    #[column(not_null, length = 128, comment = "用户密码")]
    pub password: Option<String>,
    /// salt (varchar(20)) | 可空
    #[column(length = 20, comment = "密码盐")]
    pub salt: Option<String>,
    /// password_error_num (int) | 可空
    /// 默认值: 0
    #[column(default = "0", comment = "密码错误次数")]
    pub password_error_num: Option<i32>,
    /// password_error_last_time (timestamp(3)) | 可空
    #[column(length = 3, comment = "密码错误最后时间")]
    pub password_error_last_time: Option<chrono::DateTime<chrono::Utc>>,
    /// password_expire_time (timestamp(3)) | 可空
    #[column(length = 3, comment = "密码过期时间")]
    pub password_expire_time: Option<chrono::DateTime<chrono::Utc>>,
    /// last_opt_time (timestamp(3)) | 可空
    /// 默认值: CURRENT_TIMESTAMP(3)
    #[column(
        default = "CURRENT_TIMESTAMP(3)",
        length = 3,
        index,
        comment = "最后上下线时间"
    )]
    pub last_opt_time: Option<chrono::DateTime<chrono::Utc>>,
    /// last_login_time (timestamp(3)) | 可空
    #[column(length = 3, comment = "最后登录时间")]
    pub last_login_time: Option<chrono::DateTime<chrono::Utc>>,
    /// ip_info (json) | 可空
    #[column(comment = "IP信息")]
    pub ip_info: Option<serde_json::Value>,
    /// create_time (timestamp(3)) | 非空
    /// 默认值: CURRENT_TIMESTAMP(3)
    #[column(
        not_null,
        default = "CURRENT_TIMESTAMP(3)",
        length = 3,
        index,
        comment = "创建时间"
    )]
    pub create_time: Option<chrono::DateTime<chrono::Utc>>,
    /// update_time (timestamp(3)) | 非空
    /// 默认值: CURRENT_TIMESTAMP(3)
    #[column(
        not_null,
        default = "CURRENT_TIMESTAMP(3)",
        length = 3,
        index,
        comment = "更新时间"
    )]
    pub update_time: Option<chrono::DateTime<chrono::Utc>>,
    /// create_by (bigint) | 可空
    /// 默认值: 1
    #[column(default = "1", comment = "创建人ID")]
    pub create_by: Option<i64>,
    /// update_by (bigint) | 可空
    #[column(comment = "更新人ID")]
    pub update_by: Option<i64>,
    /// is_del (tinyint) | 非空
    /// 默认值: 0
    #[column(not_null, default = "0", soft_delete, comment = "是否删除")]
    pub is_del: Option<bool>,
    /// readonly (tinyint) | 可空
    /// 默认值: 0
    #[column(default = "0", comment = "内置用户标记")]
    pub readonly: Option<bool>,
}
