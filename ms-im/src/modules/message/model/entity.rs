/// 消息
#[derive(
    Debug, Default, Clone,
    sqlx::FromRow,
    serde::Serialize, serde::Deserialize,
    sqlxplus::ModelMeta, sqlxplus::CRUD,
)]
#[model(table = "message", pk = "id", table_comment = "消息")]
pub struct Message {
    #[column(primary_key, auto_increment, comment = "主键ID")]
    pub id: Option<i64>,
    #[column(not_null, comment = "房间ID")]
    pub room_id: Option<i64>,
    #[column(not_null, comment = "发送者ID")]
    pub from_uid: Option<i64>,
    #[column(comment = "消息内容(JSON)")]
    pub content: Option<String>,
    /// 消息类型: 1文本 2图片 3文件 4语音 5视频 6撤回 7系统
    #[column(not_null, comment = "消息类型")]
    pub r#type: Option<i16>,
    #[column(comment = "回复的消息ID")]
    pub reply_msg_id: Option<i64>,
    /// 0正常 1撤回
    #[column(not_null, default = "0", comment = "0正常 1撤回")]
    pub status: Option<i16>,
    #[column(comment = "扩展信息(JSON)")]
    pub extra: Option<serde_json::Value>,
    #[column(not_null, comment = "创建时间")]
    pub created_at: Option<i64>,
    #[column(not_null, comment = "更新时间")]
    pub updated_at: Option<i64>,
}

/// 消息标记（点赞/举报）
#[derive(
    Debug, Default, Clone,
    sqlx::FromRow,
    serde::Serialize, serde::Deserialize,
    sqlxplus::ModelMeta, sqlxplus::CRUD,
)]
#[model(table = "message_mark", pk = "id", table_comment = "消息标记")]
pub struct MessageMark {
    #[column(primary_key, auto_increment, comment = "主键ID")]
    pub id: Option<i64>,
    #[column(not_null, comment = "消息ID")]
    pub msg_id: Option<i64>,
    #[column(not_null, comment = "标记用户ID")]
    pub uid: Option<i64>,
    /// 1点赞 2举报
    #[column(not_null, comment = "1点赞 2举报")]
    pub r#type: Option<i16>,
    /// 0正常 1取消
    #[column(not_null, default = "0", comment = "0正常 1取消")]
    pub status: Option<i16>,
    #[column(not_null, comment = "创建时间")]
    pub created_at: Option<i64>,
}
