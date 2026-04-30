use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use crate::modules::{
    contact::model::Contact,
    friend::model::UserFriend,
    room::model::{GroupMember, Room, RoomFriend, RoomGroup},
};

/// 增量同步请求
#[derive(Debug, Deserialize, ToSchema)]
pub struct SyncRequest {
    /// 上次同步时间的时间戳 (毫秒)
    pub since_ts: i64,
}

/// 增量同步响应
#[derive(Debug, Serialize, ToSchema)]
pub struct SyncResponse {
    /// 发生变更的好友关系（包含状态更新，如 status=2 表示删除）
    pub friends: Vec<UserFriend>,
    /// 发生变更的会话记录（包含 is_deleted=1 表示删除）
    pub contacts: Vec<Contact>,
    /// 发生变更的房间基础信息（新创建、或活跃时间更新）
    pub rooms: Vec<Room>,
    /// 发生变更的单聊房间映射（帮助客户端建立 room_id 与 friend_uid 的联系）
    pub room_friends: Vec<RoomFriend>,
    /// 发生变更的群聊信息（包含 is_deleted=1 的群组解散记录）
    pub room_groups: Vec<RoomGroup>,
    /// 如果某个群聊有变更，这里会返回该群聊全量的当前成员（客户端收到后直接覆盖该群的成员数据）
    pub group_members: Vec<GroupMember>,
    /// 由 BFF 层自动查出的相关用户资料
    pub user_profiles: Vec<crate::client::identity::UserBrief>,
}
