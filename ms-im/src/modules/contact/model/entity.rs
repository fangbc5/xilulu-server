/// 会话（用户维度的房间视图）
#[derive(
    Debug, Default, Clone,
    sqlx::FromRow,
    serde::Serialize, serde::Deserialize,
    sqlxplus::ModelMeta, sqlxplus::CRUD,
)]
#[model(table = "contact", pk = "id", table_comment = "会话")]
pub struct Contact {
    #[column(primary_key, auto_increment, comment = "主键ID")]
    pub id: Option<i64>,
    #[column(not_null, comment = "用户ID")]
    pub uid: Option<i64>,
    #[column(not_null, comment = "房间ID")]
    pub room_id: Option<i64>,
    #[column(not_null, default = "CURRENT_TIMESTAMP", comment = "已读到的时间")]
    pub read_time: Option<chrono::DateTime<chrono::Utc>>,
    #[column(not_null, default = "CURRENT_TIMESTAMP", comment = "活跃时间")]
    pub active_time: Option<chrono::DateTime<chrono::Utc>>,
    #[column(comment = "最后一条消息ID")]
    pub last_msg_id: Option<i64>,
    #[column(comment = "最后一次已读的消息ID")]
    pub read_msg_id: Option<i64>,
    #[column(not_null, default = "0", comment = "清空聊天记录的最后游标ID")]
    pub clear_msg_id: Option<i64>,
    #[column(not_null, default = "0", comment = "是否免打扰")]
    pub is_mute: Option<i16>,
    #[column(not_null, default = "0", comment = "是否置顶")]
    pub is_top: Option<i16>,
    #[column(not_null, default = "0", comment = "是否删除")]
    pub is_deleted: Option<i16>,
    #[column(not_null, default = "0", comment = "未读消息数")]
    pub unread_count: Option<i64>,
    #[column(not_null, default = "CURRENT_TIMESTAMP", comment = "创建时间")]
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    #[column(not_null, default = "CURRENT_TIMESTAMP", comment = "更新时间")]
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}
