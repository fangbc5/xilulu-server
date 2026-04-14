// Kafka 消息推送模块
//
// 负责将 IM 事件发布到 Kafka，由 ms-websocket 消费并推送给用户

use fbc_starter::{AppState, Message};
use serde::Serialize;
use tracing::{error, warn};

/// WebSocket 推送 Kafka topic
const WS_PUSH_TOPIC: &str = "websocket_push";

/// WebSocket 推送消息类型（对应前端 WSRespTypeEnum）
#[allow(dead_code)]
pub mod ws_msg_type {
    /// 新消息通知
    pub const MESSAGE: i32 = 1001;
    /// 消息撤回通知
    pub const RECALL: i32 = 1002;
    /// 好友申请通知
    pub const FRIEND_APPLY: i32 = 1003;
    /// 好友关系变更
    pub const FRIEND_CHANGE: i32 = 1004;
    /// 群成员变更
    pub const GROUP_MEMBER_CHANGE: i32 = 1005;
    /// 消息标记变更（点赞/举报）
    pub const MESSAGE_MARK: i32 = 1006;
    /// 已读回执通知
    pub const READ_ACK: i32 = 1007;
}

/// 路由推送 DTO（与 ms-websocket RouterPushDto 对齐）
#[derive(Debug, Clone, Serialize)]
pub struct RouterPushDto {
    /// WebSocket 推送消息体
    pub ws_base_msg: WsBaseResp,
    /// 目标用户 ID 列表
    pub uid_list: Vec<u64>,
    /// 操作人 UID
    pub uid: u64,
}

/// WebSocket 基础响应（与 ms-websocket WsBaseResp 对齐）
#[derive(Debug, Clone, Serialize)]
pub struct WsBaseResp {
    /// 消息类型
    pub r#type: i32,
    /// 消息数据
    pub data: serde_json::Value,
}

/// Kafka 推送工具
pub struct KafkaPusher;

impl KafkaPusher {
    /// 推送消息到 ms-websocket（异步，不阻塞业务流程）
    ///
    /// 失败仅记录日志，不影响业务返回
    pub fn push_async(
        fbc: std::sync::Arc<AppState>,
        msg_type: i32,
        data: serde_json::Value,
        uid_list: Vec<u64>,
        operator_uid: u64,
    ) {
        tokio::spawn(async move {
            if let Err(e) = Self::push_inner(&fbc, msg_type, data, uid_list, operator_uid).await {
                error!("Kafka 推送失败: {}", e);
            }
        });
    }

    /// 内部推送方法
    async fn push_inner(
        fbc: &AppState,
        msg_type: i32,
        data: serde_json::Value,
        uid_list: Vec<u64>,
        operator_uid: u64,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let producer = fbc.message_producer()
            .map_err(|e| format!("Kafka producer 未初始化: {}", e))?;

        if uid_list.is_empty() {
            warn!("推送目标用户列表为空，跳过");
            return Ok(());
        }

        let push_dto = RouterPushDto {
            ws_base_msg: WsBaseResp {
                r#type: msg_type,
                data,
            },
            uid_list,
            uid: operator_uid,
        };

        let message = Message::new(
            WS_PUSH_TOPIC,
            "ms-im",
            serde_json::to_value(&push_dto)?,
        );

        producer.publish(WS_PUSH_TOPIC, message).await?;
        Ok(())
    }
}
