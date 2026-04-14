use std::sync::Arc;

use fbc_starter::AppState as FbcAppState;
use sqlxplus::{Crud, DbPool};
use tracing::info;

use crate::client::identity::IdentityClient;
use crate::error::ImError;
use crate::kafka::{ws_msg_type, KafkaPusher};
use crate::modules::contact::repository::ContactRepo;

use super::model::{
    CursorPageResponse, Message, MessageCursorQuery, SendMessageRequest,
};
use super::repository::{MessageMarkRepo, MessageRepo};

/// 消息服务
pub struct MessageService {
    db: Arc<DbPool>,
    fbc: Arc<FbcAppState>,
}

impl MessageService {
    pub fn new(db: Arc<DbPool>, fbc: Arc<FbcAppState>) -> Self {
        Self { db, fbc }
    }

    /// 发送消息
    ///
    /// 写流程：插入消息 → 并发更新 room + contacts → 异步 Kafka 推送
    pub async fn send_message(&self, from_uid: i64, req: SendMessageRequest) -> Result<Message, ImError> {
        let room_id = req.room_id;

        let now = Some(chrono::Utc::now());
        let msg = Message {
            room_id: Some(room_id),
            from_uid: Some(from_uid),
            content: Some(req.content),
            r#type: Some(req.r#type),
            reply_msg_id: req.reply_msg_id,
            status: Some(0),
            extra: req.extra,
            created_at: now,
            updated_at: now,
            ..Default::default()
        };
        let msg_id = msg.insert(self.db.mysql_pool()).await?;

        // 并发更新 room 和所有 contact 的活跃状态
        let pool = self.db.mysql_pool();
        tokio::try_join!(
            MessageRepo::update_room_active(pool, room_id, msg_id),
            MessageRepo::update_contacts_active(pool, room_id, msg_id, from_uid)
        )?;

        // 返回完整消息（含 id）
        let saved = Message::find_by_id(self.db.mysql_pool(), msg_id)
            .await?
            .ok_or(ImError::DatabaseError("消息插入后查询失败".to_string()))?;

        info!("消息已发送: msg_id={}, room_id={}, from={}", msg_id, room_id, from_uid);

        // 异步 Kafka 推送（不阻塞 HTTP 响应）
        self.push_new_message_event(room_id, from_uid, &saved).await;

        Ok(saved)
    }

    /// 消息列表（游标分页，高性能）
    pub async fn list_messages(&self, uid: i64, query: MessageCursorQuery) -> Result<CursorPageResponse<Message>, ImError> {
        // 获取当前用户的会话，取出 clear_msg_id
        let contact_builder = sqlxplus::QueryBuilder::new("SELECT * FROM `contact`")
            .and_eq("uid", uid)
            .and_eq("room_id", query.room_id);
        let contact = crate::modules::contact::model::Contact::find_all(self.db.mysql_pool(), Some(contact_builder))
            .await?
            .into_iter()
            .next();
        let clear_msg_id = contact.and_then(|c| c.clear_msg_id).unwrap_or(0);

        // 使用 sqlxplus 游标分页
        let mut builder = sqlxplus::QueryBuilder::new("SELECT * FROM `message`")
            .and_eq("room_id", query.room_id)
            .and_gt("id", clear_msg_id); // 核心修改：过滤掉被删除隐藏的历史消息

        // 抓取方向
        if query.fetch_mode == 1 {
            // 正向游标：id > cursor（加载更新的消息，用于进房定位未读）
            if let Some(cursor_id) = query.cursor {
                builder = builder.and_gt("id", cursor_id);
            }
            builder = builder.order_by("id", true); // ASC
        } else {
            // 降序游标：id < cursor（加载更早的历史消息）
            if let Some(cursor_id) = query.cursor {
                builder = builder.and_lt("id", cursor_id);
            }
            builder = builder.order_by("id", false); // DESC
        }

        // paginate_cursor 内部：传 None 让它添加 id > 0（对正整数 ID 无影响）
        let result = Message::paginate_cursor(
            self.db.mysql_pool(),
            builder,
            None,
            query.size as u32,
        ).await?;

        Ok(CursorPageResponse {
            list: result.items,
            cursor: result.next_cursor,
            has_more: result.has_next,
        })
    }

    /// 撤回消息
    pub async fn recall_message(&self, msg_id: i64, uid: i64) -> Result<(), ImError> {
        let msg = Message::find_by_id(self.db.mysql_pool(), msg_id)
            .await?
            .ok_or(ImError::MessageNotFound)?;

        // 只有发送者能撤回
        if msg.from_uid != Some(uid) {
            return Err(ImError::PermissionDenied("只有发送者能撤回消息".to_string()));
        }

        // 检查是否已撤回
        if msg.status == Some(1) {
            return Err(ImError::MessageAlreadyRecalled);
        }

        // 更新状态为撤回
        let updated = Message {
            id: Some(msg_id),
            status: Some(1),
            updated_at: Some(chrono::Utc::now()),
            ..Default::default()
        };
        updated.update(self.db.mysql_pool()).await?;

        info!("消息已撤回: msg_id={}, uid={}", msg_id, uid);

        // 异步 Kafka 推送撤回通知
        let room_id = msg.room_id.unwrap_or(0);
        self.push_recall_event(room_id, uid, msg_id).await;

        Ok(())
    }

    /// 标记消息（点赞/举报，toggle 行为）
    pub async fn toggle_mark(&self, msg_id: i64, uid: i64, mark_type: i16) -> Result<bool, ImError> {
        // 验证消息存在
        Message::find_by_id(self.db.mysql_pool(), msg_id)
            .await?
            .ok_or(ImError::MessageNotFound)?;

        let active = MessageMarkRepo::toggle_mark(
            self.db.mysql_pool(),
            msg_id,
            uid,
            mark_type,
        ).await?;

        let action = if active { "添加" } else { "取消" };
        info!("消息标记{}: msg_id={}, uid={}, type={}", action, msg_id, uid, mark_type);
        Ok(active)
    }

    // === Kafka 推送方法 ===

    /// 推送新消息事件
    async fn push_new_message_event(&self, room_id: i64, from_uid: i64, saved_msg: &Message) {
        // 查询房间内所有用户（排除发送者自己）
        let uid_list = match ContactRepo::find_uids_by_room_id(self.db.mysql_pool(), room_id).await {
            Ok(uids) => uids.into_iter()
                .filter(|uid| *uid != from_uid)
                .map(|uid| uid as u64)
                .collect::<Vec<u64>>(),
            Err(e) => {
                tracing::error!("查询房间用户失败: room_id={}, err={}", room_id, e);
                return;
            }
        };

        if uid_list.is_empty() {
            return;
        }

        // 序列化消息并注入 from_name（用于前端群聊摘要前缀）
        let mut data = serde_json::to_value(saved_msg).unwrap_or_default();
        if let Ok(user_map) = IdentityClient::batch_get_user_info(vec![from_uid]).await {
            if let Some(user) = user_map.get(&from_uid) {
                data["from_name"] = serde_json::Value::String(user.nick_name.clone());
            }
        }

        KafkaPusher::push_async(
            self.fbc.clone(),
            ws_msg_type::MESSAGE,
            data,
            uid_list,
            from_uid as u64,
        );
    }

    /// 推送撤回事件
    async fn push_recall_event(&self, room_id: i64, uid: i64, msg_id: i64) {
        let uid_list = match ContactRepo::find_uids_by_room_id(self.db.mysql_pool(), room_id).await {
            Ok(uids) => uids.into_iter()
                .filter(|u| *u != uid)
                .map(|u| u as u64)
                .collect::<Vec<u64>>(),
            Err(e) => {
                tracing::error!("查询房间用户失败: room_id={}, err={}", room_id, e);
                return;
            }
        };

        if uid_list.is_empty() {
            return;
        }

        let data = serde_json::json!({
            "msg_id": msg_id,
            "room_id": room_id,
        });
        KafkaPusher::push_async(
            self.fbc.clone(),
            ws_msg_type::RECALL,
            data,
            uid_list,
            uid as u64,
        );
    }
}
