use std::sync::Arc;

use sqlxplus::{Crud, DbPool};
use tracing::info;

use super::model::{SyncRequest, SyncResponse};
use crate::error::ImError;
use crate::modules::contact::model::Contact;
use crate::modules::friend::model::UserFriend;
use crate::modules::room::model::{GroupMember, Room, RoomFriend, RoomGroup};
use crate::client::identity::IdentityClient;

pub struct SyncService {
    db: Arc<DbPool>,
}

impl SyncService {
    pub fn new(db: Arc<DbPool>) -> Self {
        Self { db }
    }

    pub async fn pull_sync(&self, uid: i64, req: &SyncRequest) -> Result<SyncResponse, ImError> {
        info!("开始执行增量同步: uid={}, since={}", uid, req.since_ts);

        let pool = self.db.mysql_pool();

        // 1. 同步好友关系变动（双向关系，仅拉取自己的即可，包含被软删除的 status=2）
        let friend_builder = sqlxplus::QueryBuilder::new("SELECT * FROM `user_friend`")
            .and_eq("uid", uid)
            .and_gt("updated_at", req.since_ts);
        let friends = UserFriend::find_all(pool, Some(friend_builder)).await?;

        // 2. 同步会话列表变动（包含软删除的 is_deleted=1）
        let contact_builder = sqlxplus::QueryBuilder::new("SELECT * FROM `contact`")
            .and_eq("uid", uid)
            .and_gt("updated_at", req.since_ts);
        let contacts = Contact::find_all(pool, Some(contact_builder)).await?;

        // 3. 找出需要同步的房间基础信息（由于会话关联了 room，我们需要拉取变动的房间）
        let mut room_ids = std::collections::HashSet::new();
        for c in &contacts {
            if let Some(r_id) = c.room_id {
                room_ids.insert(r_id);
            }
        }

        // 我们也需要同步所有已加入的群聊是否有变更
        let mut group_ids = std::collections::HashSet::new();
        let my_group_members =
            sqlxplus::QueryBuilder::new("SELECT * FROM `group_member`").and_eq("uid", uid);
        let my_groups = GroupMember::find_all(pool, Some(my_group_members)).await?;
        for mg in my_groups {
            if let Some(g_id) = mg.group_id {
                group_ids.insert(g_id);
            }
        }

        // 找出这些群聊属于哪个 room_id（群组和房间有一层映射关系可以一起拉取）
        // 但直接查有变更的组更直接
        let mut room_groups = vec![];
        let mut changed_group_ids = vec![];
        if !group_ids.is_empty() {
            let group_ids_vec: Vec<i64> = group_ids.into_iter().collect();
            let group_builder = sqlxplus::QueryBuilder::new("SELECT * FROM `room_group`")
                .and_in("id", group_ids_vec)
                .and_gt("updated_at", req.since_ts);
            room_groups = RoomGroup::find_all(pool, Some(group_builder)).await?;

            for rg in &room_groups {
                if let Some(r_id) = rg.room_id {
                    room_ids.insert(r_id);
                }
                if let Some(g_id) = rg.id {
                    changed_group_ids.push(g_id);
                }
            }
        }

        let mut rooms = vec![];
        if !room_ids.is_empty() {
            let r_ids_vec: Vec<i64> = room_ids.into_iter().collect();
            let room_builder = sqlxplus::QueryBuilder::new("SELECT * FROM `room`")
                .and_in("id", r_ids_vec.clone())
                .and_gt("updated_at", req.since_ts); // 或者直接不管时间全部下发有变动的 contact 的关联房间
            rooms = Room::find_all(pool, Some(room_builder)).await?;
        }

        // 我们也需要下发变动的 room_friend，让客户端知道哪些单聊 room_id 对应了哪个 friend_uid
        let rf_builder = sqlxplus::QueryBuilder::new("SELECT * FROM `room_friend`")
            .and_group(|mut b| {
                b = b.or_eq("uid1", uid);
                b = b.or_eq("uid2", uid);
                b
            })
            .and_gt("created_at", req.since_ts); // room_friend 只有 created_at
        let room_friends = RoomFriend::find_all(pool, Some(rf_builder)).await?;

        // 4. 群成员信息（重点：应对组成员的 Hard Delete，对于有变动的群，直接下发全量现存成员列表）
        // 这样客户端如果发现本地的某些人不在这份全量名单里，即可直接在本地删除。
        let mut group_members = vec![];
        if !changed_group_ids.is_empty() {
            let member_builder = sqlxplus::QueryBuilder::new("SELECT * FROM `group_member`")
                .and_in("group_id", changed_group_ids);
            group_members = GroupMember::find_all(pool, Some(member_builder)).await?;
        }

        // 5. 抓取所需的用户资料 (BFF)
        let mut profile_uids = std::collections::HashSet::new();
        for f in &friends {
            if let Some(f_uid) = f.friend_uid { profile_uids.insert(f_uid); }
        }
        for rf in &room_friends {
            if let Some(u1) = rf.uid1 { if u1 != uid { profile_uids.insert(u1); } }
            if let Some(u2) = rf.uid2 { if u2 != uid { profile_uids.insert(u2); } }
        }
        for gm in &group_members {
            if let Some(u) = gm.uid { if u != uid { profile_uids.insert(u); } }
        }

        // 把自己也加入资料同步列表，以便本地能渲染自己的头像和昵称
        profile_uids.insert(uid);

        let mut user_profiles = vec![];
        if !profile_uids.is_empty() {
            let uids_vec: Vec<i64> = profile_uids.into_iter().collect();
            if let Ok(user_map) = IdentityClient::batch_get_user_info(uids_vec).await {
                for (_, user_info) in user_map {
                    user_profiles.push(user_info);
                }
            }
        }

        Ok(SyncResponse {
            friends,
            contacts,
            rooms,
            room_friends,
            room_groups,
            group_members,
            user_profiles,
        })
    }
}
