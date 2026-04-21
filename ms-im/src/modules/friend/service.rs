use std::sync::Arc;

use sqlxplus::{Crud, DbPool};
use tracing::info;

use fbc_starter::AppState as FbcAppState;

use crate::client::identity::IdentityClient;
use crate::error::ImError;
use crate::kafka::{ws_msg_type, KafkaPusher};
use crate::modules::contact::service::ContactService;
use crate::modules::room::service::RoomService;

use super::model::{ApplyVO, FriendVO, UserApply, UserFriend};
use super::repository::{ApplyRepo, FriendRepo};

/// 好友服务
pub struct FriendService {
    db: Arc<DbPool>,
    fbc: Arc<FbcAppState>,
}

impl FriendService {
    pub fn new(db: Arc<DbPool>, fbc: Arc<FbcAppState>) -> Self {
        Self { db, fbc }
    }

    /// 发送好友申请
    pub async fn apply(
        &self,
        uid: i64,
        target_id: i64,
        msg: Option<String>,
    ) -> Result<i64, ImError> {
        if uid == target_id {
            return Err(ImError::CannotAddSelf);
        }

        // 检查是否已是好友
        if FriendRepo::is_friend(self.db.mysql_pool(), uid, target_id).await? {
            return Err(ImError::AlreadyFriend);
        }

        // 检查是否有待审批的申请
        if ApplyRepo::find_pending(self.db.mysql_pool(), uid, target_id)
            .await?
            .is_some()
        {
            return Err(ImError::PendingApplyExists);
        }

        let apply = UserApply {
            uid: Some(uid),
            target_id: Some(target_id),
            msg,
            r#type: Some(1), // 好友申请
            status: Some(0), // 待审批
            read_status: Some(0),
            ..Default::default()
        };

        let id = apply.insert(self.db.mysql_pool()).await?;

        // 异步推送 WebSocket 通知
        KafkaPusher::push_async(
            self.fbc.clone(),
            ws_msg_type::FRIEND_APPLY,
            serde_json::json!({ "apply_id": id, "action": "add" }),
            vec![target_id as u64],
            uid as u64,
        );

        info!(
            "好友申请已发送: uid={}, target={}, apply_id={}",
            uid, target_id, id
        );
        Ok(id)
    }

    /// 同意好友申请
    ///
    /// 业务流程：更新申请状态 → 创建双向好友 → 创建单聊房间 → 创建双方会话
    pub async fn approve(
        &self,
        apply_id: i64,
        uid: i64,
        room_service: &RoomService,
        contact_service: &ContactService,
    ) -> Result<(), ImError> {
        let apply = UserApply::find_by_id(self.db.mysql_pool(), apply_id)
            .await?
            .ok_or(ImError::ApplyNotFound)?;

        if apply.target_id != Some(uid) {
            return Err(ImError::PermissionDenied("只有目标用户能同意".to_string()));
        }
        if apply.status != Some(0) {
            return Err(ImError::ApplyAlreadyHandled);
        }

        let applicant_uid = apply.uid.unwrap();

        // 1. 更新申请状态
        ApplyRepo::update_status(self.db.mysql_pool(), apply_id, 1).await?;

        // 2. 并发创建双向好友关系
        let pool = self.db.mysql_pool();
        let f1 = UserFriend {
            uid: Some(applicant_uid),
            friend_uid: Some(uid),
            status: Some(1),
            ..Default::default()
        };
        let f2 = UserFriend {
            uid: Some(uid),
            friend_uid: Some(applicant_uid),
            status: Some(1),
            ..Default::default()
        };
        tokio::try_join!(f1.insert(pool), f2.insert(pool))?;

        // 3. 创建单聊房间（幂等，已存在则返回现有 room_id）
        let room_id = room_service.create_friend_room(applicant_uid, uid).await?;

        // 4. 并发为双方创建会话
        tokio::try_join!(
            contact_service.create_contact(applicant_uid, uid),
            contact_service.create_contact(uid, room_id)
        )?;

        // 异步推送 WebSocket 通知
        KafkaPusher::push_async(
            self.fbc.clone(),
            ws_msg_type::FRIEND_CHANGE,
            serde_json::json!({ "action": "approve", "uid": applicant_uid, "target_id": uid }),
            vec![applicant_uid as u64, uid as u64],
            uid as u64,
        );

        info!(
            "好友关系已建立: {} <-> {}, room_id={}",
            applicant_uid, uid, room_id
        );
        Ok(())
    }

    /// 拒绝好友申请
    pub async fn reject(&self, apply_id: i64, uid: i64) -> Result<(), ImError> {
        let apply = UserApply::find_by_id(self.db.mysql_pool(), apply_id)
            .await?
            .ok_or(ImError::ApplyNotFound)?;

        if apply.target_id != Some(uid) {
            return Err(ImError::PermissionDenied("只有目标用户能拒绝".to_string()));
        }
        if apply.status != Some(0) {
            return Err(ImError::ApplyAlreadyHandled);
        }

        ApplyRepo::update_status(self.db.mysql_pool(), apply_id, 2).await?;

        // 异步推送 WebSocket 通知给申请人
        if let Some(applicant_uid) = apply.uid {
            KafkaPusher::push_async(
                self.fbc.clone(),
                ws_msg_type::FRIEND_APPLY,
                serde_json::json!({ "apply_id": apply_id, "action": "reject" }),
                vec![applicant_uid as u64],
                uid as u64,
            );
        }

        info!("好友申请已拒绝: apply_id={}", apply_id);
        Ok(())
    }

    /// 删除好友（单向删除）
    pub async fn delete_friend(&self, uid: i64, friend_uid: i64) -> Result<(), ImError> {
        FriendRepo::delete_friend(self.db.mysql_pool(), uid, friend_uid).await?;

        // 异步推送 WebSocket 通知，两边都推，让对方也知道你把他删了（或者不推对方，看业务需求，我们选择推双方以便即时更新视图）
        KafkaPusher::push_async(
            self.fbc.clone(),
            ws_msg_type::FRIEND_CHANGE,
            serde_json::json!({ "action": "delete", "uid": uid, "target_id": friend_uid }),
            vec![uid as u64, friend_uid as u64],
            uid as u64,
        );

        info!("好友已删除: {} -> {}", uid, friend_uid);
        Ok(())
    }

    /// 好友列表（分页 + 聚合用户信息）
    pub async fn list_friends(
        &self,
        uid: i64,
        cursor: Option<u32>,
        page_size: u32,
    ) -> Result<(Vec<FriendVO>, Option<u32>, bool), ImError> {
        // 1. 构造查询条件，使用 sqlxplus 游标分页
        let builder = sqlxplus::QueryBuilder::new("SELECT * FROM `user_friend`")
            .and_eq("uid", uid)
            .and_eq("status", 1i16);

        let result = UserFriend::paginate_cursor(
            self.db.mysql_pool(),
            builder,
            cursor.map(|c| c as i64),
            page_size,
        )
        .await?;

        let has_next = result.has_next;
        let next_cursor = result.next_cursor.map(|c| c as u32);
        let friends = result.items;

        if friends.is_empty() {
            return Ok((vec![], None, true));
        }

        // 2. 提取好友 uid 列表，批量查询用户信息
        let friend_uids: Vec<i64> = friends.iter().filter_map(|f| f.friend_uid).collect();

        let user_map = IdentityClient::batch_get_user_info(friend_uids)
            .await
            .map_err(|e| ImError::RpcError(format!("调用 identity 服务失败: {}", e)))?;

        // 3. 组装 FriendVO
        let list: Vec<FriendVO> = friends
            .into_iter()
            .map(|f| {
                let friend_uid = f.friend_uid.unwrap_or(0);
                let user = user_map.get(&friend_uid);
                FriendVO {
                    friend_uid,
                    remark: f.remark.clone(),
                    nick_name: user
                        .map(|u| u.nick_name.clone())
                        .unwrap_or_else(|| format!("用户 {}", friend_uid)),
                    avatar: user.map(|u| u.avatar.clone()).unwrap_or_default(),
                }
            })
            .collect();

        Ok((list, next_cursor, has_next))
    }

    /// 收到的申请列表（包含发出的申请，聚合对应人的信息）
    pub async fn list_applies(&self, uid: i64) -> Result<Vec<ApplyVO>, ImError> {
        let applies = ApplyRepo::find_all_applies(self.db.mysql_pool(), uid).await?;
        if applies.is_empty() {
            return Ok(vec![]);
        }

        // 收集涉及到的用户 uid（去重在后面由 batch_get_user_info 处理）
        let mut uids: Vec<i64> = Vec::new();
        for a in &applies {
            if let Some(fid) = a.uid {
                uids.push(fid);
            }
            if let Some(target_id) = a.target_id {
                uids.push(target_id);
            }
        }

        // gRPC 批量获取用户信息
        let user_map = IdentityClient::batch_get_user_info(uids)
            .await
            .unwrap_or_default();

        let list = applies
            .into_iter()
            .map(|a| {
                let applicant_uid = a.uid.unwrap_or(0);
                let target_uid = a.target_id.unwrap_or(0);
                let is_sent = applicant_uid == uid;

                // 如果是发出的申请，显示对方（目标用户）的信息；
                // 否则显示申请人的信息。
                let display_uid = if is_sent { target_uid } else { applicant_uid };
                let user = user_map.get(&display_uid);

                ApplyVO {
                    id: a.id.unwrap_or(0),
                    uid: display_uid,
                    msg: a.msg,
                    status: a.status.unwrap_or(0),
                    created_at: a.created_at.unwrap_or_default(),
                    is_sent,
                    nick_name: user.map(|u| u.nick_name.clone()).unwrap_or_default(),
                    avatar: user.map(|u| u.avatar.clone()).unwrap_or_default(),
                }
            })
            .collect();

        Ok(list)
    }
}
