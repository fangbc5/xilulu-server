/// UserDevice 实体
///
/// 表名: `user_device`
/// 主键: `id`
/// 字段数: 9

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
    table = "user_device",
    pk = "id",
    table_comment = "用户推送设备表"
)]
pub struct UserDevice {
    /// 主键ID
    #[column(primary_key, auto_increment, comment = "主键ID")]
    pub id: Option<i64>,
    /// 用户ID
    #[column(not_null, index, comment = "用户ID")]
    pub uid: Option<i64>,
    /// 设备指纹（与 WS 的 clientId 一致）
    #[column(not_null, length = 64, comment = "设备指纹")]
    pub client_id: Option<String>,
    /// APNs/FCM 推送 Token
    #[column(not_null, length = 512, comment = "推送Token")]
    pub device_token: Option<String>,
    /// 平台类型：ios / android
    #[column(not_null, length = 16, comment = "平台类型: ios/android")]
    pub platform: Option<String>,
    /// 客户端版本号
    #[column(length = 32, comment = "客户端版本号")]
    pub app_version: Option<String>,
    /// 是否有效（1=有效，0=已注销/过期）
    #[column(not_null, default = "1", comment = "是否有效: 1=有效 0=无效")]
    pub is_active: Option<i16>,
    /// 创建时间
    #[column(
        not_null,
        default = "CURRENT_TIMESTAMP(3)",
        length = 3,
        comment = "创建时间"
    )]
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    /// 更新时间
    #[column(
        not_null,
        default = "CURRENT_TIMESTAMP(3)",
        length = 3,
        comment = "更新时间"
    )]
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}
