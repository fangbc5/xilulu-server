use std::sync::Arc;

use sqlxplus::{Crud, DbPool};
use tracing::info;

use fbc_starter::AppState as FbcAppState;

use crate::error::ImError;
use crate::kafka::{ws_msg_type, KafkaPusher};

use super::model::{GroupMember, Room, RoomFriend, RoomGroup};
use super::repository::{GroupMemberRepo, RoomFriendRepo, RoomGroupRepo};

/// 群名最大字符数
const GROUP_NAME_MAX_LEN: usize = 32;

/// 房间服务
pub struct RoomService {
    db: Arc<DbPool>,
    fbc: Arc<FbcAppState>,
}

impl RoomService {
    pub fn new(db: Arc<DbPool>, fbc: Arc<FbcAppState>) -> Self {
        Self { db, fbc }
    }

    /// 创建或获取单聊房间（好友同意时调用）
    /// 幂等操作：room_key 唯一，已存在则直接返回
    pub async fn create_friend_room(&self, uid_a: i64, uid_b: i64) -> Result<i64, ImError> {
        let (uid1, uid2) = if uid_a < uid_b { (uid_a, uid_b) } else { (uid_b, uid_a) };
        let room_key = format!("{}_{}", uid1, uid2);

        // 检查是否已存在
        if let Some(existing) = RoomFriendRepo::find_by_room_key(self.db.mysql_pool(), &room_key).await? {
            info!("单聊房间已存在: room_id={}", existing.room_id.unwrap());
            return Ok(existing.room_id.unwrap());
        }

        // 创建 room
        let now = Some(chrono::Utc::now().timestamp_millis());
        let room = Room {
            r#type: Some(1), // 单聊
            hot_flag: Some(0),
            active_time: now,
            ..Default::default()
        };
        let room_id = room.insert(self.db.mysql_pool()).await?;

        // 创建 room_friend
        let rf = RoomFriend {
            room_id: Some(room_id),
            uid1: Some(uid1),
            uid2: Some(uid2),
            room_key: Some(room_key),
            status: Some(1),
            ..Default::default()
        };
        rf.insert(self.db.mysql_pool()).await?;

        info!("单聊房间已创建: room_id={}, {} <-> {}", room_id, uid1, uid2);
        Ok(room_id)
    }

    /// 创建群聊房间
    ///
    /// 返回 (room_id, 所有成员 UID 列表)，供 Handler 层编排跨模块会话创建
    pub async fn create_group_room(
        &self,
        creator_uid: i64,
        name: String,
        member_uids: Vec<i64>,
    ) -> Result<(i64, Vec<i64>), ImError> {
        if name.is_empty() {
            return Err(ImError::GroupNameEmpty);
        }

        // 截断群名至 32 字符
        let name = truncate_name(&name, GROUP_NAME_MAX_LEN);

        // 创建 room
        let now = Some(chrono::Utc::now().timestamp_millis());
        let room = Room {
            r#type: Some(2), // 群聊
            hot_flag: Some(0),
            active_time: now,
            ..Default::default()
        };
        let room_id = room.insert(self.db.mysql_pool()).await?;

        // 创建 room_group
        let rg = RoomGroup {
            room_id: Some(room_id),
            name: Some(name.clone()),
            is_deleted: Some(0),
            created_by: Some(creator_uid),
            updated_by: Some(creator_uid),
            ..Default::default()
        };
        let group_id = rg.insert(self.db.mysql_pool()).await?;

        // 收集所有成员 uid（去重，包含创建者）
        let mut all_uids = vec![creator_uid];
        for uid in &member_uids {
            if *uid != creator_uid {
                all_uids.push(*uid);
            }
        }

        // 插入群成员
        for uid in &all_uids {
            let role = if *uid == creator_uid { 1 } else { 3 };
            let member = GroupMember {
                group_id: Some(group_id),
                uid: Some(*uid),
                role: Some(role),
                created_at: now,
                updated_at: now,
                ..Default::default()
            };
            member.insert(self.db.mysql_pool()).await?;
        }

        // 异步推送 WebSocket 通知给所有人
        if !all_uids.is_empty() {
            KafkaPusher::push_async(
                self.fbc.clone(),
                ws_msg_type::GROUP_MEMBER_CHANGE,
                serde_json::json!({ "action": "create", "group_id": group_id, "room_id": room_id }),
                all_uids.iter().map(|&uid| uid as u64).collect(),
                creator_uid as u64,
            );
        }

        info!("群聊已创建: room_id={}, group_id={}, name={}, members={}", room_id, group_id, name, all_uids.len());
        Ok((room_id, all_uids))
    }

    /// 添加群成员
    ///
    /// Handler 层负责编排跨模块的会话创建
    pub async fn add_group_member(
        &self,
        room_id: i64,
        operator_uid: i64,
        uid: i64,
    ) -> Result<(), ImError> {
        let group = RoomGroupRepo::find_by_room_id(self.db.mysql_pool(), room_id)
            .await?
            .ok_or(ImError::GroupNotFound)?;
        let group_id = group.id.unwrap();

        // 检查操作者权限（群主或管理员）
        let operator = GroupMemberRepo::find_member(self.db.mysql_pool(), group_id, operator_uid)
            .await?
            .ok_or(ImError::NotInGroup)?;
        if operator.role.unwrap_or(3) > 2 {
            return Err(ImError::PermissionDenied("无权添加成员".to_string()));
        }

        // 检查是否已在群中
        if GroupMemberRepo::find_member(self.db.mysql_pool(), group_id, uid).await?.is_some() {
            return Err(ImError::AlreadyInGroup);
        }

        // 创建成员记录
        let member = GroupMember {
            group_id: Some(group_id),
            uid: Some(uid),
            role: Some(3),
            ..Default::default()
        };
        member.insert(self.db.mysql_pool()).await?;

        // 异步推送给所有成员
        if let Ok(members) = GroupMemberRepo::find_by_group_id(self.db.mysql_pool(), group_id).await {
            let all_uids: Vec<u64> = members.into_iter().filter_map(|m| m.uid).map(|uid| uid as u64).collect();
            if !all_uids.is_empty() {
                KafkaPusher::push_async(
                    self.fbc.clone(),
                    ws_msg_type::GROUP_MEMBER_CHANGE,
                    serde_json::json!({ "action": "add", "group_id": group_id, "room_id": room_id, "target_uid": uid }),
                    all_uids,
                    operator_uid as u64,
                );
            }
        }

        info!("群成员已添加: group_id={}, uid={}", group_id, uid);
        Ok(())
    }

    /// 移除群成员
    pub async fn remove_group_member(&self, room_id: i64, operator_uid: i64, uid: i64) -> Result<(), ImError> {
        let group = RoomGroupRepo::find_by_room_id(self.db.mysql_pool(), room_id)
            .await?
            .ok_or(ImError::GroupNotFound)?;
        let group_id = group.id.unwrap();

        if operator_uid == uid {
            return Err(ImError::CannotRemoveSelf);
        }

        // 检查操作者权限
        let operator = GroupMemberRepo::find_member(self.db.mysql_pool(), group_id, operator_uid)
            .await?
            .ok_or(ImError::NotInGroup)?;
        let target = GroupMemberRepo::find_member(self.db.mysql_pool(), group_id, uid)
            .await?
            .ok_or(ImError::NotInGroup)?;

        // 权限校验：只能移除比自己权限低的成员
        if operator.role.unwrap_or(3) >= target.role.unwrap_or(3) {
            return Err(ImError::PermissionDenied("无权移除该成员".to_string()));
        }

        GroupMemberRepo::delete(self.db.mysql_pool(), group_id, uid).await?;

        // 异步推送给被剔除的用户，以及剩下的人
        let mut emit_uids = vec![uid as u64];
        if let Ok(members) = GroupMemberRepo::find_by_group_id(self.db.mysql_pool(), group_id).await {
            emit_uids.extend(members.into_iter().filter_map(|m| m.uid).map(|u| u as u64));
        }

        KafkaPusher::push_async(
            self.fbc.clone(),
            ws_msg_type::GROUP_MEMBER_CHANGE,
            serde_json::json!({ "action": "remove", "group_id": group_id, "room_id": room_id, "target_uid": uid }),
            emit_uids,
            operator_uid as u64,
        );

        info!("群成员已移除: group_id={}, uid={}", group_id, uid);
        Ok(())
    }

    /// 退出群聊
    pub async fn quit_group(&self, room_id: i64, uid: i64) -> Result<(), ImError> {
        let group = RoomGroupRepo::find_by_room_id(self.db.mysql_pool(), room_id)
            .await?
            .ok_or(ImError::GroupNotFound)?;
        let group_id = group.id.unwrap();

        let member = GroupMemberRepo::find_member(self.db.mysql_pool(), group_id, uid)
            .await?
            .ok_or(ImError::NotInGroup)?;

        if member.role == Some(1) {
            return Err(ImError::OwnerCannotQuit);
        }

        GroupMemberRepo::delete(self.db.mysql_pool(), group_id, uid).await?;

        // 推送给剩下的人和退群的人
        let mut emit_uids = vec![uid as u64];
        if let Ok(members) = GroupMemberRepo::find_by_group_id(self.db.mysql_pool(), group_id).await {
            emit_uids.extend(members.into_iter().filter_map(|m| m.uid).map(|u| u as u64));
        }
        KafkaPusher::push_async(
            self.fbc.clone(),
            ws_msg_type::GROUP_MEMBER_CHANGE,
            serde_json::json!({ "action": "quit", "group_id": group_id, "room_id": room_id, "target_uid": uid }),
            emit_uids,
            uid as u64,
        );

        info!("已退出群聊: group_id={}, uid={}", group_id, uid);
        Ok(())
    }

    /// 查询群成员列表
    pub async fn list_group_members(&self, room_id: i64) -> Result<Vec<GroupMember>, ImError> {
        let group = RoomGroupRepo::find_by_room_id(self.db.mysql_pool(), room_id)
            .await?
            .ok_or(ImError::GroupNotFound)?;
        GroupMemberRepo::find_by_group_id(self.db.mysql_pool(), group.id.unwrap()).await
    }

    /// 查询群详情
    pub async fn get_group_info(&self, room_id: i64) -> Result<Option<RoomGroup>, ImError> {
        RoomGroupRepo::find_by_room_id(self.db.mysql_pool(), room_id).await
    }

    /// 更新群名/公告
    pub async fn update_group_info(&self, room_id: i64, operator_uid: i64, name: Option<String>, notice: Option<String>) -> Result<(), ImError> {
        let group = RoomGroupRepo::find_by_room_id(self.db.mysql_pool(), room_id)
            .await?
            .ok_or(ImError::GroupNotFound)?;
        let group_id = group.id.unwrap();

        // 检查操作者权限
        let operator = GroupMemberRepo::find_member(self.db.mysql_pool(), group_id, operator_uid)
            .await?
            .ok_or(ImError::NotInGroup)?;
        if operator.role.unwrap_or(3) > 2 {
            return Err(ImError::PermissionDenied("无权修改群信息".to_string()));
        }

        // 校验群名长度
        if let Some(ref n) = name {
            if n.chars().count() > GROUP_NAME_MAX_LEN {
                return Err(ImError::GroupNameTooLong);
            }
        }

        let updated = RoomGroup {
            id: Some(group_id),
            name: name.or(group.name),
            notice: notice.or(group.notice),
            updated_by: Some(operator_uid),
            ..Default::default()
        };
        updated.update(self.db.mysql_pool()).await?;

        // 异步推送给所有成员
        if let Ok(members) = GroupMemberRepo::find_by_group_id(self.db.mysql_pool(), group_id).await {
            let all_uids: Vec<u64> = members.into_iter().filter_map(|m| m.uid).map(|uid| uid as u64).collect();
            if !all_uids.is_empty() {
                KafkaPusher::push_async(
                    self.fbc.clone(),
                    ws_msg_type::GROUP_MEMBER_CHANGE,
                    serde_json::json!({ "action": "update", "group_id": group_id, "room_id": room_id }),
                    all_uids,
                    operator_uid as u64,
                );
            }
        }

        info!("群信息已更新: group_id={}", group_id);
        Ok(())
    }

    /// 解散群聊（仅群主可操作，软删除）
    /// 返回所有成员 UID 列表，供 Handler 层编排跨模块会话删除
    pub async fn dissolve_group(
        &self,
        room_id: i64,
        operator_uid: i64,
    ) -> Result<Vec<i64>, ImError> {
        let group = RoomGroupRepo::find_by_room_id(self.db.mysql_pool(), room_id)
            .await?
            .ok_or(ImError::GroupNotFound)?;
        let group_id = group.id.unwrap();

        // 只有群主可解散
        let operator = GroupMemberRepo::find_member(self.db.mysql_pool(), group_id, operator_uid)
            .await?
            .ok_or(ImError::NotInGroup)?;
        if operator.role != Some(1) {
            return Err(ImError::PermissionDenied("只有群主可以解散群聊".to_string()));
        }

        // 软删除群（sqlxplus 自动处理 is_deleted）
        RoomGroup::delete_by_id(self.db.mysql_pool(), group_id).await?;

        // 收集所有成员 UID
        let members = GroupMemberRepo::find_by_group_id(self.db.mysql_pool(), group_id).await?;
        let member_uids: Vec<i64> = members.iter().filter_map(|m| m.uid).collect();

        // 异步推送给所有群成员
        if !member_uids.is_empty() {
            let uids: Vec<u64> = member_uids.iter().map(|&uid| uid as u64).collect();
            KafkaPusher::push_async(
                self.fbc.clone(),
                ws_msg_type::GROUP_MEMBER_CHANGE,
                serde_json::json!({ "action": "dissolve", "group_id": group_id, "room_id": room_id }),
                uids,
                operator_uid as u64,
            );
        }

        info!("群聊已解散: group_id={}, operator={}", group_id, operator_uid);
        Ok(member_uids)
    }

    /// 转让群主
    pub async fn transfer_owner(
        &self,
        room_id: i64,
        operator_uid: i64,
        new_owner_uid: i64,
    ) -> Result<(), ImError> {
        if operator_uid == new_owner_uid {
            return Err(ImError::CannotTransferToSelf);
        }

        let group = RoomGroupRepo::find_by_room_id(self.db.mysql_pool(), room_id)
            .await?
            .ok_or(ImError::GroupNotFound)?;
        let group_id = group.id.unwrap();

        // 检查操作者是群主
        let operator = GroupMemberRepo::find_member(self.db.mysql_pool(), group_id, operator_uid)
            .await?
            .ok_or(ImError::NotInGroup)?;
        if operator.role != Some(1) {
            return Err(ImError::PermissionDenied("只有群主可以转让".to_string()));
        }

        // 检查目标是群成员
        GroupMemberRepo::find_member(self.db.mysql_pool(), group_id, new_owner_uid)
            .await?
            .ok_or(ImError::NotInGroup)?;

        // 更新角色：原群主降为普通成员，新群主升级
        let old_owner = GroupMember {
            id: operator.id,
            role: Some(3),
            ..Default::default()
        };
        old_owner.update(self.db.mysql_pool()).await?;

        let new_target = GroupMemberRepo::find_member(self.db.mysql_pool(), group_id, new_owner_uid)
            .await?
            .ok_or(ImError::NotInGroup)?;
        let new_owner = GroupMember {
            id: new_target.id,
            role: Some(1),
            ..Default::default()
        };
        new_owner.update(self.db.mysql_pool()).await?;

        // 异步推送给所有群成员
        if let Ok(members) = GroupMemberRepo::find_by_group_id(self.db.mysql_pool(), group_id).await {
            let all_uids: Vec<u64> = members.into_iter().filter_map(|m| m.uid).map(|uid| uid as u64).collect();
            if !all_uids.is_empty() {
                KafkaPusher::push_async(
                    self.fbc.clone(),
                    ws_msg_type::GROUP_MEMBER_CHANGE,
                    serde_json::json!({ "action": "transfer", "group_id": group_id, "room_id": room_id, "target_uid": new_owner_uid }),
                    all_uids,
                    operator_uid as u64,
                );
            }
        }

        info!("群主已转让: group_id={}, {} -> {}", group_id, operator_uid, new_owner_uid);
        Ok(())
    }
}

/// 截断字符串至指定字符数，超出部分拼 "..."
fn truncate_name(s: &str, max_chars: usize) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= max_chars {
        s.to_string()
    } else {
        let truncated: String = chars[..max_chars.saturating_sub(3)].iter().collect();
        format!("{}...", truncated)
    }
}
