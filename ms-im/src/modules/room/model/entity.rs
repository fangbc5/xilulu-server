/// 房间（统一单聊和群聊的抽象层）
#[derive(
    Debug, Default, Clone,
    sqlx::FromRow,
    serde::Serialize, serde::Deserialize,
    sqlxplus::ModelMeta, sqlxplus::CRUD,
)]
#[model(table = "room", pk = "id", table_comment = "房间")]
pub struct Room {
    #[column(primary_key, auto_increment, comment = "主键ID")]
    pub id: Option<i64>,
    /// 1单聊 2群聊
    #[column(not_null, comment = "1单聊 2群聊")]
    pub r#type: Option<i16>,
    #[column(not_null, default = "0", comment = "是否热点群")]
    pub hot_flag: Option<i16>,
    #[column(comment = "最新消息ID")]
    pub last_msg_id: Option<i64>,
    #[column(not_null, comment = "最后活跃时间")]
    pub active_time: Option<i64>,
    #[column(not_null, comment = "创建时间")]
    pub created_at: Option<i64>,
    #[column(not_null, comment = "更新时间")]
    pub updated_at: Option<i64>,
}

/// 单聊房间扩展
#[derive(
    Debug, Default, Clone,
    sqlx::FromRow,
    serde::Serialize, serde::Deserialize,
    sqlxplus::ModelMeta, sqlxplus::CRUD,
)]
#[model(table = "room_friend", pk = "id", table_comment = "单聊房间")]
pub struct RoomFriend {
    #[column(primary_key, auto_increment, comment = "主键ID")]
    pub id: Option<i64>,
    #[column(not_null, comment = "房间ID")]
    pub room_id: Option<i64>,
    #[column(not_null, comment = "较小的uid")]
    pub uid1: Option<i64>,
    #[column(not_null, comment = "较大的uid")]
    pub uid2: Option<i64>,
    #[column(not_null, length = 64, comment = "拼接的roomKey: uid1_uid2")]
    pub room_key: Option<String>,
    #[column(not_null, default = "1", comment = "1正常 2禁用")]
    pub status: Option<i16>,
    #[column(not_null, comment = "创建时间")]
    pub created_at: Option<i64>,
}

/// 群聊房间扩展
#[derive(
    Debug, Default, Clone,
    sqlx::FromRow,
    serde::Serialize, serde::Deserialize,
    sqlxplus::ModelMeta, sqlxplus::CRUD,
)]
#[model(table = "room_group", pk = "id", soft_delete = "is_deleted", table_comment = "群聊房间")]
pub struct RoomGroup {
    #[column(primary_key, auto_increment, comment = "主键ID")]
    pub id: Option<i64>,
    #[column(not_null, comment = "房间ID")]
    pub room_id: Option<i64>,
    #[column(not_null, length = 64, comment = "群名")]
    pub name: Option<String>,
    #[column(length = 256, comment = "群头像")]
    pub avatar: Option<String>,
    #[column(comment = "群公告")]
    pub notice: Option<String>,
    #[column(not_null, default = "0", comment = "0正常 1已解散")]
    pub is_deleted: Option<i16>,
    #[column(not_null, comment = "创建人UID")]
    pub created_by: Option<i64>,
    #[column(not_null, comment = "修改人UID")]
    pub updated_by: Option<i64>,
    #[column(not_null, comment = "创建时间")]
    pub created_at: Option<i64>,
    #[column(not_null, comment = "更新时间")]
    pub updated_at: Option<i64>,
}

/// 群成员
#[derive(
    Debug, Default, Clone,
    sqlx::FromRow,
    serde::Serialize, serde::Deserialize,
    sqlxplus::ModelMeta, sqlxplus::CRUD,
)]
#[model(table = "group_member", pk = "id", table_comment = "群成员")]
pub struct GroupMember {
    #[column(primary_key, auto_increment, comment = "主键ID")]
    pub id: Option<i64>,
    #[column(not_null, comment = "room_group.id")]
    pub group_id: Option<i64>,
    #[column(not_null, comment = "用户ID")]
    pub uid: Option<i64>,
    /// 1群主 2管理员 3普通成员
    #[column(not_null, default = "3", comment = "1群主 2管理员 3普通成员")]
    pub role: Option<i16>,
    #[column(not_null, comment = "创建时间")]
    pub created_at: Option<i64>,
    #[column(not_null, comment = "更新时间")]
    pub updated_at: Option<i64>,
}
