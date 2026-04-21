/// 通知发送日志
///
/// 表名: `notify_log`
/// 主键: `id`

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
#[model(table = "notify_log", pk = "id", table_comment = "通知发送日志")]
pub struct NotifyLog {
    /// 主键
    #[column(primary_key, auto_increment, comment = "主键ID")]
    pub id: Option<i64>,
    /// 渠道：email / sms / im_feishu / im_dingding / im_wechat
    #[column(not_null, length = 32, index, comment = "渠道")]
    pub channel: Option<String>,
    /// 发送者
    #[column(length = 255, comment = "发送者")]
    pub sender: Option<String>,
    /// 接收者
    #[column(length = 255, comment = "接收者")]
    pub receiver: Option<String>,
    /// 主题
    #[column(length = 255, comment = "主题")]
    pub subject: Option<String>,
    /// 发送内容
    #[column(comment = "发送内容")]
    pub body: Option<String>,
    /// 状态：0=待发送 1=发送中 2=成功 3=失败
    #[column(not_null, default = "0", comment = "状态")]
    pub status: Option<i16>,
    /// 失败原因
    #[column(length = 500, comment = "失败原因")]
    pub error_msg: Option<String>,
    /// 重试次数
    #[column(not_null, default = "0", comment = "重试次数")]
    pub retry_count: Option<i32>,
    /// 业务类型
    #[column(length = 64, comment = "业务类型")]
    pub biz_type: Option<String>,
    /// 业务关联 ID
    #[column(length = 128, comment = "业务关联ID")]
    pub biz_id: Option<String>,
    /// 创建时间
    #[column(not_null, comment = "")]
    pub created_at: Option<i64>,
    /// updated_at (datetime)
    #[column(not_null, comment = "")]
    pub updated_at: Option<i64>,
}
