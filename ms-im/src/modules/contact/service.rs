use std::collections::HashMap;
use std::sync::Arc;

use sqlxplus::{Crud, DbPool};
use tracing::info;

use crate::client::identity::IdentityClient;
use crate::error::ImError;
use crate::kafka::{ws_msg_type, KafkaPusher};
use crate::modules::message::repository::MessageRepo;
use crate::modules::room::repository::{RoomFriendRepo, RoomGroupRepo, RoomRepo};

use crate::cache::ContactMuteCacheKeyBuilder;
use fbc_starter::cache::CacheKeyBuilder;
use fbc_starter::AppState as FbcAppState;

use super::model::{Contact, ContactVO};
use super::repository::ContactRepo;

/// 会话服务
pub struct ContactService {
    db: Arc<DbPool>,
    fbc: Arc<FbcAppState>,
}

impl ContactService {
    pub fn new(db: Arc<DbPool>, fbc: Arc<FbcAppState>) -> Self {
        Self { db, fbc }
    }

    /// 创建会话（好友建立/入群时调用）
    pub async fn create_contact(&self, uid: i64, room_id: i64) -> Result<i64, ImError> {
        // 检查是否已存在（可能之前软删除过）
        if let Some(existing) = ContactRepo::find_by_uid_room(self.db.mysql_pool(), uid, room_id).await? {
            if existing.is_deleted == Some(1) {
                ContactRepo::restore(self.db.mysql_pool(), uid, room_id).await?;
                info!("会话已恢复: uid={}, room_id={}", uid, room_id);
                return Ok(existing.id.unwrap());
            }
            return Ok(existing.id.unwrap());
        }

        let now = Some(chrono::Utc::now().timestamp_millis());
        let contact = Contact {
            uid: Some(uid),
            room_id: Some(room_id),
            read_time: now,
            active_time: now,
            is_mute: Some(0),
            is_top: Some(0),
            is_deleted: Some(0),
            unread_count: Some(0),
            ..Default::default()
        };
        let id = contact.insert(self.db.mysql_pool()).await?;
        info!("会话已创建: uid={}, room_id={}, contact_id={}", uid, room_id, id);
        Ok(id)
    }

    /// 会话列表（游标分页，聚合房间名称/头像/类型）
    pub async fn list_contacts(
        &self,
        uid: i64,
        cursor: Option<u32>,
        page_size: u32,
    ) -> Result<(Vec<ContactVO>, Option<u32>, bool), ImError> {
        // 使用 sqlxplus 游标分页
        let builder = sqlxplus::QueryBuilder::new("SELECT * FROM `contact`")
            .and_eq("uid", uid)
            .and_eq("is_deleted", 0i16)
            .order_by("active_time", false);

        let result = Contact::paginate_cursor(
            self.db.mysql_pool(),
            builder,
            cursor.map(|c| c as i64),
            page_size,
        ).await?;

        let has_more = result.has_next;
        let items = result.items;

        if items.is_empty() {
            return Ok((vec![], None, true));
        }

        // 1. 收集 room_ids
        let room_ids: Vec<i64> = items
            .iter()
            .filter_map(|c| c.room_id)
            .collect();

        // 2. 批量查 Room → 得到 type
        let rooms = RoomRepo::find_by_ids(self.db.mysql_pool(), &room_ids).await?;
        let room_map: HashMap<i64, i16> = rooms
            .into_iter()
            .filter_map(|r| Some((r.id?, r.r#type.unwrap_or(1))))
            .collect();

        // 3. 按类型分类 room_ids
        let single_room_ids: Vec<i64> = room_ids
            .iter()
            .filter(|id| room_map.get(id).copied().unwrap_or(1) == 1)
            .copied()
            .collect();
        let group_room_ids: Vec<i64> = room_ids
            .iter()
            .filter(|id| room_map.get(id).copied().unwrap_or(1) == 2)
            .copied()
            .collect();

        // 4. 单聊 → RoomFriend → 找对方 uid → Identity 获取 name/avatar
        let room_friends = RoomFriendRepo::find_by_room_ids(self.db.mysql_pool(), &single_room_ids).await?;
        let friend_uid_map: HashMap<i64, i64> = room_friends
            .iter()
            .filter_map(|rf| {
                let room_id = rf.room_id?;
                let uid1 = rf.uid1?;
                let uid2 = rf.uid2?;
                let other_uid = if uid1 == uid { uid2 } else { uid1 };
                Some((room_id, other_uid))
            })
            .collect();

        let friend_uids: Vec<i64> = friend_uid_map.values().copied().collect();
        let user_map = if friend_uids.is_empty() {
            HashMap::new()
        } else {
            IdentityClient::batch_get_user_info(friend_uids)
                .await
                .unwrap_or_default()
        };

        // 5. 群聊 → RoomGroup → name/avatar
        let room_groups = RoomGroupRepo::find_by_room_ids(self.db.mysql_pool(), &group_room_ids).await?;
        let group_map: HashMap<i64, (String, String)> = room_groups
            .into_iter()
            .filter_map(|rg| {
                let room_id = rg.room_id?;
                let name = rg.name.unwrap_or_default();
                let avatar = rg.avatar.unwrap_or_default();
                Some((room_id, (name, avatar)))
            })
            .collect();

        // 5.5 批量查最新消息内容（用于会话列表摘要）
        let last_msg_ids: Vec<i64> = items
            .iter()
            .filter_map(|c| c.last_msg_id)
            .collect();
        let last_msgs = MessageRepo::find_by_ids(self.db.mysql_pool(), &last_msg_ids).await?;
        // msg_id → (content, type, from_uid)
        let msg_map: HashMap<i64, (Option<String>, Option<i16>, Option<i64>)> = last_msgs
            .into_iter()
            .filter_map(|m| {
                let id = m.id?;
                Some((id, (m.content.clone(), m.r#type, m.from_uid)))
            })
            .collect();

        // 5.6 批量查消息发送者昵称（合并到已有 user_map 中）
        let msg_sender_uids: Vec<i64> = msg_map
            .values()
            .filter_map(|(_, _, uid)| *uid)
            .filter(|uid| !user_map.contains_key(uid))
            .collect();
        let sender_map = if msg_sender_uids.is_empty() {
            HashMap::new()
        } else {
            IdentityClient::batch_get_user_info(msg_sender_uids)
                .await
                .unwrap_or_default()
        };

        // 6. 提取 next_cursor（从 paginate_cursor 返回的 PK 游标转换）
        let next_cursor = result.next_cursor.map(|c| c as u32);

        // 7. 组装 ContactVO（直接读取 entity 上的 unread_count）
        let list = items
            .into_iter()
            .map(|c| {
                let room_id = c.room_id.unwrap_or(0);
                let room_type = room_map.get(&room_id).copied().unwrap_or(1);

                let (name, avatar) = if room_type == 1 {
                    let other_uid = friend_uid_map.get(&room_id).copied().unwrap_or(0);
                    let user = user_map.get(&other_uid);
                    (
                        user.map(|u| u.nick_name.clone()).unwrap_or_default(),
                        user.map(|u| u.avatar.clone()).unwrap_or_default(),
                    )
                } else {
                    group_map
                        .get(&room_id)
                        .cloned()
                        .unwrap_or_default()
                };

                // 提取最新消息摘要
                let (last_msg_content, last_msg_type, last_msg_from_name) =
                    if let Some(msg_id) = c.last_msg_id {
                        if let Some((content, msg_type, from_uid)) = msg_map.get(&msg_id) {
                            let from_name = from_uid.and_then(|uid| {
                                user_map.get(&uid)
                                    .or_else(|| sender_map.get(&uid))
                                    .map(|u| u.nick_name.clone())
                            });
                            (content.clone(), *msg_type, from_name)
                        } else {
                            (None, None, None)
                        }
                    } else {
                        (None, None, None)
                    };

                ContactVO {
                    room_id,
                    active_time: c.active_time.unwrap_or_default(),
                    last_msg_id: c.last_msg_id,
                    read_msg_id: c.read_msg_id,
                    is_mute: c.is_mute.unwrap_or(0),
                    is_top: c.is_top.unwrap_or(0),
                    room_type,
                    name,
                    avatar,
                    unread_count: c.unread_count.unwrap_or(0),
                    last_msg_content,
                    last_msg_type,
                    last_msg_from_name,
                }
            })
            .collect();

        Ok((list, next_cursor, has_more))
    }

    /// 置顶/取消置顶
    pub async fn set_top(&self, uid: i64, room_id: i64, is_top: bool) -> Result<(), ImError> {
        let contact = ContactRepo::find_by_uid_room(self.db.mysql_pool(), uid, room_id)
            .await?
            .ok_or(ImError::ContactNotFound)?;
        let updated = Contact {
            id: contact.id,
            is_top: Some(if is_top { 1 } else { 0 }),
            ..Default::default()
        };
        updated.update(self.db.mysql_pool()).await?;
        info!("会话置顶状态: uid={}, room_id={}, top={}", uid, room_id, is_top);
        Ok(())
    }

    /// 免打扰/取消免打扰
    pub async fn set_mute(&self, uid: i64, room_id: i64, is_mute: bool) -> Result<(), ImError> {
        let contact = ContactRepo::find_by_uid_room(self.db.mysql_pool(), uid, room_id)
            .await?
            .ok_or(ImError::ContactNotFound)?;
        let updated = Contact {
            id: contact.id,
            is_mute: Some(if is_mute { 1 } else { 0 }),
            ..Default::default()
        };
        updated.update(self.db.mysql_pool()).await?;

        // 同步更新 Redis 缓存（ms-notify 仅读取此缓存来判断是否推送）
        // 设计约定：
        //   - SET key = "1" 表示该用户已开启免打扰
        //   - DEL key 表示未免打扰（ms-notify 读不到则默认不推送，安全兜底）
        let cache_key = ContactMuteCacheKeyBuilder.key(&[&uid, &room_id]);
        match self.fbc.redis().await {
            Ok(mut conn) => {
                use redis::AsyncCommands;
                if is_mute {
                    let ttl = cache_key.expire.unwrap().as_secs();
                    let _: Result<(), _> = conn.set_ex(&cache_key.key, "1", ttl).await;
                } else {
                    let _: Result<(), _> = conn.del(&cache_key.key).await;
                }
                info!("Redis 免打扰缓存已更新: key={}, mute={}", cache_key.key, is_mute);
            }
            Err(e) => {
                // Redis 不可用时仍以 DB 为准，不影响主流程
                tracing::warn!("Redis 免打扰缓存更新失败: key={}, err={}", cache_key.key, e);
            }
        }

        info!("会话免打扰状态: uid={}, room_id={}, mute={}", uid, room_id, is_mute);
        Ok(())
    }

    /// 已读上报
    pub async fn mark_read(&self, uid: i64, room_id: i64, read_msg_id: i64, diff_count: i64) -> Result<(), ImError> {
        tracing::info!("mark_read 调用: uid={}, room_id={}, read_msg_id={}, diff_count={}", uid, room_id, read_msg_id, diff_count);
        ContactRepo::update_read_offset(self.db.mysql_pool(), uid, room_id, read_msg_id, diff_count).await?;

        // 异步推送已读回执给房间内其他成员
        self.push_read_ack_event(room_id, uid, read_msg_id).await;

        Ok(())
    }

    /// 推送已读回执事件（通知对方"我已读到哪条"）
    async fn push_read_ack_event(&self, room_id: i64, uid: i64, read_msg_id: i64) {
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
            "room_id": room_id,
            "uid": uid,
            "read_msg_id": read_msg_id,
        });
        KafkaPusher::push_async(
            self.fbc.clone(),
            ws_msg_type::READ_ACK,
            data,
            uid_list,
            uid as u64,
        );
    }

    /// 标记未读
    pub async fn mark_unread(&self, uid: i64, room_id: i64) -> Result<(), ImError> {
        ContactRepo::set_unread(self.db.mysql_pool(), uid, room_id).await?;
        info!("会话标记为未读: uid={}, room_id={}", uid, room_id);
        Ok(())
    }

    /// 删除会话
    pub async fn delete_contact(&self, uid: i64, room_id: i64) -> Result<(), ImError> {
        ContactRepo::soft_delete(self.db.mysql_pool(), uid, room_id).await?;
        info!("会话已删除: uid={}, room_id={}", uid, room_id);
        Ok(())
    }
}
