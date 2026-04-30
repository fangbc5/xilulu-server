/// 好友关系
#[derive(
    Debug, Default, Clone,
    sqlx::FromRow,
    serde::Serialize, serde::Deserialize,
    sqlxplus::ModelMeta, sqlxplus::CRUD,
    utoipa::ToSchema,
)]
#[model(table = "user_friend", pk = "id", table_comment = "好友关系")]
pub struct UserFriend {
    #[column(primary_key, auto_increment, comment = "主键ID")]
    pub id: Option<i64>,
    #[column(not_null, comment = "用户ID")]
    pub uid: Option<i64>,
    #[column(not_null, comment = "好友ID")]
    pub friend_uid: Option<i64>,
    #[column(length = 64, comment = "好友备注")]
    pub remark: Option<String>,
    #[column(not_null, default = "1", comment = "1正常 2删除")]
    pub status: Option<i16>,
    #[column(not_null, comment = "创建时间")]
    pub created_at: Option<i64>,
    #[column(not_null, comment = "更新时间")]
    pub updated_at: Option<i64>,
}

/// 好友申请
#[derive(
    Debug, Default, Clone,
    sqlx::FromRow,
    serde::Serialize, serde::Deserialize,
    sqlxplus::ModelMeta, sqlxplus::CRUD,
)]
#[model(table = "user_apply", pk = "id", table_comment = "好友申请")]
pub struct UserApply {
    #[column(primary_key, auto_increment, comment = "主键ID")]
    pub id: Option<i64>,
    #[column(not_null, comment = "申请人ID")]
    pub uid: Option<i64>,
    #[column(not_null, comment = "目标ID")]
    pub target_id: Option<i64>,
    #[column(length = 256, comment = "申请消息")]
    pub msg: Option<String>,
    /// 1好友申请 2群申请
    #[column(not_null, default = "1", comment = "申请类型")]
    pub r#type: Option<i16>,
    /// 0待审批 1同意 2拒绝
    #[column(not_null, default = "0", comment = "审批状态")]
    pub status: Option<i16>,
    /// 0未读 1已读
    #[column(not_null, default = "0", comment = "已读状态")]
    pub read_status: Option<i16>,
    #[column(not_null, comment = "创建时间")]
    pub created_at: Option<i64>,
    #[column(not_null, comment = "更新时间")]
    pub updated_at: Option<i64>,
}
