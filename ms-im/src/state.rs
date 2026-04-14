use std::sync::Arc;

use fbc_starter::AppState as FbcAppState;
use sqlxplus::DbPool;

use crate::modules::contact::service::ContactService;
use crate::modules::friend::service::FriendService;
use crate::modules::message::service::MessageService;
use crate::modules::room::service::RoomService;

/// IM 服务应用状态
#[derive(Clone)]
#[allow(dead_code)]
pub struct ImState {
    /// 框架 AppState（访问 Redis、Kafka 等）
    pub fbc: Arc<FbcAppState>,
    /// 数据库连接池
    pub db_pool: Arc<DbPool>,
    /// 好友服务
    pub friend_service: Arc<FriendService>,
    /// 房间服务
    pub room_service: Arc<RoomService>,
    /// 会话服务
    pub contact_service: Arc<ContactService>,
    /// 消息服务
    pub message_service: Arc<MessageService>,
}
