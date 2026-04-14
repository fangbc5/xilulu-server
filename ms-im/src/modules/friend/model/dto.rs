use serde::{Deserialize, Serialize};

/// 好友申请请求
#[derive(Debug, Deserialize)]
pub struct ApplyRequest {
    pub target_id: i64,
    pub msg: Option<String>,
}

/// 删除好友请求
#[derive(Debug, Deserialize)]
pub struct DeleteFriendRequest {
    pub friend_uid: i64,
}

/// 好友列表请求（使用框架 CursorPageBaseReq）
#[derive(Debug, Deserialize)]
pub struct ListFriendsRequest {
    #[serde(flatten)]
    pub page: fbc_starter::base::CursorPageBaseReq,
}

/// 好友信息 VO（聚合用户信息后的响应）
#[derive(Debug, Serialize)]
pub struct FriendVO {
    /// 好友 UID
    pub friend_uid: i64,
    /// 备注
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remark: Option<String>,
    /// 昵称
    pub nick_name: String,
    /// 头像 URL
    pub avatar: String,
}

/// 好友申请 VO（聚合申请人信息后的响应）
#[derive(Debug, Serialize)]
pub struct ApplyVO {
    pub id: i64,
    /// 申请人 UID
    pub uid: i64,
    /// 申请消息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub msg: Option<String>,
    /// 审批状态: 0待审批 1同意 2拒绝
    pub status: i16,
    /// 创建时间
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// 是否为发出的申请
    pub is_sent: bool,
    /// 申请人或目标对象的昵称
    pub nick_name: String,
    /// 申请人头像
    pub avatar: String,
}

/// 搜索用户请求
#[derive(Debug, Deserialize)]
pub struct SearchFriendRequest {
    pub keyword: String,
}

/// 搜索用户返回信息（包含好友标识）
#[derive(Debug, Serialize)]
pub struct FriendSearchVO {
    pub id: i64,
    pub nick_name: String,
    pub avatar: String,
    pub is_friend: bool,
    pub is_applying: bool,
}
