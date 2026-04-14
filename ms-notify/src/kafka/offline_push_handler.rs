/// 离线推送 Kafka 消费者
///
/// 功能：
/// 1. 消费 `notify_push_offline` topic 的离线推送事件
/// 2. 读取 Redis 免打扰缓存（`chat:contact:is_mute:string:{uid}:{room_id}`）
/// 3. 过滤已静音用户，发送 APNs/FCM 推送通知
///
/// 设计约定：
/// - Redis 中存在 `chat:contact:is_mute:string:{uid}:{room_id}` 且值为 "1" 表示已开启免打扰
/// - Redis 中不存在该 key 或 Redis 查询失败，默认为不静音（即进行推送）
/// - 实际推送通过 HTTP 调用 ms-identity 获取设备 Token
use async_trait::async_trait;
use fbc_starter::{AppState, KafkaMessageHandler, Message as KafkaMessage};
use serde::Deserialize;
use std::sync::Arc;
use tracing::{error, info, warn};

/// 离线推送 Kafka Topic
const OFFLINE_PUSH_TOPIC: &str = "notify_push_offline";

/// 离线推送事件数据结构
#[derive(Debug, Deserialize)]
struct OfflinePushEvent {
    /// 发送者 UID
    sender_uid: u64,
    /// 离线用户 UID 列表
    offline_uids: Vec<u64>,
    /// 房间 ID
    room_id: i64,
    /// 消息类型
    #[allow(dead_code)]
    msg_type: i32,
    /// 消息数据（包含 content 等）
    msg_data: serde_json::Value,
}

use crate::modules::push::service::PushService;

/// 离线推送处理器
pub struct OfflinePushHandler {
    _app_state: Arc<AppState>,
    push_service: PushService,
}

impl OfflinePushHandler {
    /// 创建离线推送处理器
    pub fn new(_app_state: Arc<AppState>, push_service: PushService) -> Self {
        Self { _app_state, push_service }
    }

    /// 处理离线推送事件
    async fn handle_offline_push(&self, event: OfflinePushEvent) {
        info!(
            "收到离线推送事件: sender={}, room_id={}, offline_users={:?}",
            event.sender_uid, event.room_id, event.offline_uids
        );

        // 1. 过滤已静音的用户
        let push_uids = match self.filter_muted_users(&event.offline_uids, event.room_id).await {
            Ok(uids) => uids,
            Err(e) => {
                // Redis 不可用时获取不到免打扰设置，默认不静音（允许推送）
                warn!("Redis 免打扰缓存查询失败，降级为默认推送: {}", e);
                event.offline_uids.clone()
            }
        };

        if push_uids.is_empty() {
            info!("所有离线用户均已静音，跳过推送");
            return;
        }

        // 2. 提取推送内容
        let content = event.msg_data
            .get("content")
            .and_then(|v| v.as_str())
            .unwrap_or("[新消息]");

        // 3. 调用 ms-identity 获取设备 Token
        use crate::client::device::DeviceClient;

        info!(
            "准备推送通知: push_uids={:?}, content={}, room_id={}",
            push_uids,
            if content.len() > 50 { &content[..50] } else { content },
            event.room_id
        );

        let devices = match DeviceClient::get_user_devices(push_uids.clone()).await {
            Ok(res) => res.devices,
            Err(e) => {
                error!("调用 ms-identity 获取设备 Token 失败: {}", e);
                return;
            }
        };

        if devices.is_empty() {
            info!("未找到任何活跃的设备 Token");
            return;
        }

        // 实际调用 APNs/FCM 推送系统
        for device in &devices {
            if let Err(e) = self.push_service.dispatch_push(
                &device.platform,
                &device.push_token,
                "新消息提醒", // default title placeholder
                content,
            ).await {
                warn!("使用 {} 推送到 uid={} ({}) 失败: {}", device.platform, device.user_id, device.push_token, e);
            }
        }
    }

    async fn filter_muted_users(&self, uids: &[u64], room_id: i64) -> anyhow::Result<Vec<u64>> {
        let mut push_uids = Vec::new();

        if uids.is_empty() {
            return Ok(push_uids);
        }

        use crate::client::im::ImClient;
        let uids_vec: Vec<u64> = uids.to_vec();
        
        match ImClient::batch_get_contact_mute_status(room_id, uids_vec.clone()).await {
            Ok(res) => {
                for status in res.statuses {
                    if status.is_mute {
                        info!("GRPC: 用户 uid={} 已静音 room_id={}, 跳过推送", status.uid, room_id);
                    } else {
                        push_uids.push(status.uid as u64);
                    }
                }
            }
            Err(e) => {
                warn!("调用 ms-im 获取免打扰状态失败: {}，降级为默认推送", e);
                push_uids.extend(uids);
            }
        }

        Ok(push_uids)
    }
}

#[async_trait]
impl KafkaMessageHandler for OfflinePushHandler {
    fn topics(&self) -> Vec<String> {
        vec![OFFLINE_PUSH_TOPIC.to_string()]
    }

    fn group_id(&self) -> String {
        "notify_offline_push_group".to_string()
    }

    async fn handle(&self, message: KafkaMessage) {
        match serde_json::from_value::<OfflinePushEvent>(message.data) {
            Ok(event) => {
                self.handle_offline_push(event).await;
            }
            Err(e) => {
                error!("解析离线推送事件失败: {}", e);
            }
        }
    }
}
