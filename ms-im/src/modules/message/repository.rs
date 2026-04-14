use sqlxplus::Crud;

use crate::error::ImError;
use crate::modules::contact::model::Contact;
use crate::modules::room::model::Room;

use super::model::{Message, MessageMark};

/// 消息 Repository
pub struct MessageRepo;

impl MessageRepo {
    /// 更新房间最后活跃时间和最新消息 ID（发消息后调用）
    pub async fn update_room_active(
        pool: &sqlx::Pool<sqlx::MySql>,
        room_id: i64,
        msg_id: i64,
    ) -> Result<(), ImError> {
        let now = Some(chrono::Utc::now());
        let updated = Room {
            id: Some(room_id),
            active_time: now,
            last_msg_id: Some(msg_id),
            updated_at: now,
            ..Default::default()
        };
        updated.update(pool).await?;
        Ok(())
    }

    /// 批量更新会话的 active_time 和 last_msg_id（发消息后更新所有相关用户的会话）
    ///
    /// - 发送者：unread_count 置 0（自己发的消息不算未读）
    /// - 其他人：unread_count + 1
    pub async fn update_contacts_active(
        pool: &sqlx::Pool<sqlx::MySql>,
        room_id: i64,
        msg_id: i64,
        from_uid: i64,
    ) -> Result<(), ImError> {
        let now = Some(chrono::Utc::now());

        // 发送者：使用 UpdateBuilder 更新活跃 + 未读清零
        let sender_model = Contact {
            active_time: now,
            last_msg_id: Some(msg_id),
            read_msg_id: Some(msg_id),
            unread_count: Some(0),
            is_deleted: Some(0),
            ..Default::default()
        };
        sqlxplus::UpdateBuilder::new(sender_model)
            .fields(&["active_time", "last_msg_id", "read_msg_id", "unread_count", "is_deleted"])
            .condition(|b| b.and_eq("room_id", room_id).and_eq("uid", from_uid))
            .execute::<sqlx::MySql, _>(pool)
            .await?;

        // 其他人：unread_count + 1 是 SQL 表达式，UpdateBuilder 无法表达，需使用原始 SQL
        sqlx::query(
            "UPDATE `contact` SET `active_time` = NOW(), `last_msg_id` = ?, `unread_count` = `unread_count` + 1, `is_deleted` = 0, `updated_at` = NOW() WHERE `room_id` = ? AND `uid` != ?"
        )
        .bind(msg_id)
        .bind(room_id)
        .bind(from_uid)
        .execute(pool)
        .await?;
        Ok(())
    }

    /// 批量查询消息（根据 ID 列表）
    pub async fn find_by_ids(
        pool: &sqlx::Pool<sqlx::MySql>,
        ids: &[i64],
    ) -> Result<Vec<Message>, ImError> {
        if ids.is_empty() {
            return Ok(vec![]);
        }
        let builder = sqlxplus::QueryBuilder::new("SELECT * FROM `message`")
            .and_in("id", ids.to_vec());
        let list = Message::find_all(pool, Some(builder)).await?;
        Ok(list)
    }
}

/// 消息标记 Repository
pub struct MessageMarkRepo;

impl MessageMarkRepo {
    /// 查询用户是否已标记（用于切换/幂等）
    pub async fn find_mark(
        pool: &sqlx::Pool<sqlx::MySql>,
        msg_id: i64,
        uid: i64,
        mark_type: i16,
    ) -> Result<Option<MessageMark>, ImError> {
        let builder = sqlxplus::QueryBuilder::new("SELECT * FROM `message_mark`")
            .and_eq("msg_id", msg_id)
            .and_eq("uid", uid)
            .and_eq("type", mark_type);
        let list = MessageMark::find_all(pool, Some(builder)).await?;
        Ok(list.into_iter().next())
    }

    /// 切换标记状态（已存在则切换，不存在则创建）
    pub async fn toggle_mark(
        pool: &sqlx::Pool<sqlx::MySql>,
        msg_id: i64,
        uid: i64,
        mark_type: i16,
    ) -> Result<bool, ImError> {
        if let Some(existing) = Self::find_mark(pool, msg_id, uid, mark_type).await? {
            // 切换状态: 0正常 <-> 1取消
            let new_status = if existing.status == Some(0) { 1i16 } else { 0i16 };
            let updated = MessageMark {
                id: existing.id,
                status: Some(new_status),
                ..Default::default()
            };
            updated.update(pool).await?;
            Ok(new_status == 0) // 返回 true 表示标记生效
        } else {
            // 新建标记
            let mark = MessageMark {
                msg_id: Some(msg_id),
                uid: Some(uid),
                r#type: Some(mark_type),
                status: Some(0),
                created_at: Some(chrono::Utc::now()),
                ..Default::default()
            };
            mark.insert(pool).await?;
            Ok(true)
        }
    }
}
