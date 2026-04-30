use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// 置顶/免打扰请求
#[derive(Debug, Deserialize, ToSchema)]
pub struct ContactSettingRequest {
    pub room_id: i64,
    pub value: bool,
}

/// 会话操作请求
#[derive(Debug, Deserialize, ToSchema)]
pub struct ContactRequest {
    pub room_id: i64,
}

/// 标记已读请求
#[derive(Debug, Deserialize, ToSchema)]
pub struct MarkReadRequest {
    /// 房间 ID
    pub room_id: i64,
    /// 视口中新曝光的最大消息ID
    pub read_msg_id: i64,
    /// 这一次一共观看了多少条未读消息
    pub diff_count: i64,
}

/// 会话列表请求（游标分页）
#[derive(Debug, Deserialize)]
pub struct ListContactsRequest {
    #[serde(flatten)]
    pub page: fbc_starter::base::CursorPageBaseReq,
}

/// 会话 VO（聚合房间信息后的响应）
#[derive(Debug, Serialize, ToSchema)]
pub struct ContactVO {
    pub room_id: i64,
    pub active_time: i64,
    pub last_msg_id: Option<i64>,
    pub read_msg_id: Option<i64>,
    pub is_mute: i16,
    pub is_top: i16,
    /// 1单聊 2群聊
    pub room_type: i16,
    /// 单聊=对方昵称 群聊=群名
    pub name: String,
    /// 单聊=对方头像 群聊=群头像
    pub avatar: String,
    /// 未读消息数
    pub unread_count: i64,
    /// 最新消息内容（文本消息为正文，其他类型可为空）
    pub last_msg_content: Option<String>,
    /// 最新消息类型（1文本 2图片 3文件 4语音 5视频 8表情）
    pub last_msg_type: Option<i16>,
    /// 最新消息发送者昵称（群聊摘要前缀）
    pub last_msg_from_name: Option<String>,
}
