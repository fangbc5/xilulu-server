/// 消息路由服务
///
/// 消息中转工具，用于其他没有依赖路由服务的服务（如 oauth 服务）
/// 需要将消息推送给用户时，先将消息推送到当前消费者，再由当前消费者将消息推送到目标 uidList 所在的 ws 节点
///
/// 功能特点：
/// - 动态路由: 使用 Redis 存储设备节点映射关系，批量查询用户所在节点
/// - 高效分发: 从本质上避免广播风暴，减少网络开销
/// - 节点隔离: 每个节点只处理自己的消息，推送时只处理本节点连接的用户
/// - 离线推送: 检测不在线的用户，投递离线推送事件给 ms-notify
use async_trait::async_trait;
use fbc_starter::{KafkaMessageHandler, Message};
use std::sync::Arc;
use tracing::{error, info, warn};

use crate::model::dto::RouterPushDto;
use crate::state::WsState;

/// 离线推送 Kafka Topic
const OFFLINE_PUSH_TOPIC: &str = "notify_push_offline";

/// 消息路由服务
pub struct MessageRouterService {
    ws_state: Arc<WsState>,
}

impl MessageRouterService {
    /// 创建新的消息路由服务
    pub fn new(ws_state: Arc<WsState>) -> Self {
        Self { ws_state }
    }

    /// 处理路由推送消息
    async fn handle_router_push(&self, dto: RouterPushDto) {
        // 1. 获取推送的成员
        if dto.uid_list.is_empty() {
            warn!("路由推送消息的用户列表为空，跳过处理");
            return;
        }
        // 2. 推送消息
        if let Err(e) = self
            .ws_state
            .services
            .push_service
            .send_push_msg(dto.ws_base_msg.clone(), dto.uid_list.clone(), dto.uid)
            .await
        {
            error!("推送消息失败: {}", e);
        }

        // 3. 检测离线用户并投递离线推送事件
        self.publish_offline_push_event(&dto).await;
    }

    /// 检测离线用户并向 ms-notify 投递离线推送事件
    async fn publish_offline_push_event(&self, dto: &RouterPushDto) {
        // 仅对聊天消息类型触发离线推送（type=1 为 MESSAGE）
        // 其他类型（如 READ_ACK、TYPING 等）不需要推送
        if dto.ws_base_msg.r#type != 1 {
            return;
        }

        let offline_uids = match self
            .ws_state
            .services
            .push_service
            .find_offline_uids(&dto.uid_list)
            .await
        {
            Ok(uids) => uids,
            Err(e) => {
                error!("检测离线用户失败: {}", e);
                return;
            }
        };

        if offline_uids.is_empty() {
            return;
        }

        // 提取消息内容用于推送通知
        let room_id = dto.ws_base_msg.data.get("roomId")
            .or_else(|| dto.ws_base_msg.data.get("room_id"))
            .and_then(|v| v.as_i64())
            .unwrap_or(0);

        let push_event = serde_json::json!({
            "sender_uid": dto.uid,
            "offline_uids": offline_uids,
            "room_id": room_id,
            "msg_type": dto.ws_base_msg.r#type,
            "msg_data": dto.ws_base_msg.data,
        });

        // 通过 Kafka 投递到 ms-notify
        let producer = match self.ws_state.app_state.message_producer() {
            Ok(p) => p,
            Err(e) => {
                warn!("获取 Kafka Producer 失败，跳过离线推送: {}", e);
                return;
            }
        };

        let message = Message::new(
            OFFLINE_PUSH_TOPIC.to_string(),
            dto.uid.to_string(),
            push_event,
        );

        if let Err(e) = producer.publish(OFFLINE_PUSH_TOPIC, message).await {
            error!("离线推送事件投递失败: {}", e);
        } else {
            info!(
                "离线推送事件已投递: offline_uids={:?}, room_id={}, sender={}",
                offline_uids, room_id, dto.uid
            );
        }
    }
}

#[async_trait]
impl KafkaMessageHandler for MessageRouterService {
    /// 获取 Kafka 主题列表
    ///
    /// 对应 Java 中的 `MqConstant.PUSH_TOPIC`
    fn topics(&self) -> Vec<String> {
        vec!["websocket_push".to_string()]
    }

    fn group_id(&self) -> String {
        "websocket_push_group".to_string()
    }

    /// 处理消息
    async fn handle(&self, message: Message) {
        match serde_json::from_value::<RouterPushDto>(message.data) {
            Ok(dto) => {
                self.handle_router_push(dto).await;
            }
            Err(e) => {
                error!("解析路由推送消息失败: {}, topic={}", e, message.topic);
            }
        }
    }
}

