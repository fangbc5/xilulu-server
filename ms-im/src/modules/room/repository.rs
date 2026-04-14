use sqlxplus::Crud;

use crate::error::ImError;

use super::model::{GroupMember, Room, RoomFriend, RoomGroup};

/// 房间 Repository
pub struct RoomRepo;

impl RoomRepo {
    /// 更新房间最后活跃时间和最新消息ID
    pub async fn update_active(pool: &sqlx::Pool<sqlx::MySql>, room_id: i64, msg_id: Option<i64>) -> Result<(), ImError> {
        sqlx::query("UPDATE `room` SET `active_time` = NOW(), `last_msg_id` = COALESCE(?, `last_msg_id`), `updated_at` = NOW() WHERE `id` = ?")
            .bind(msg_id)
            .bind(room_id)
            .execute(pool)
            .await?;
        Ok(())
    }

    /// 批量查询房间
    pub async fn find_by_ids(pool: &sqlx::Pool<sqlx::MySql>, ids: &[i64]) -> Result<Vec<Room>, ImError> {
        if ids.is_empty() {
            return Ok(vec![]);
        }
        let builder = sqlxplus::QueryBuilder::new("SELECT * FROM `room`")
            .and_in("id", ids.to_vec());
        let list = Room::find_all(pool, Some(builder)).await?;
        Ok(list)
    }
}

/// 单聊房间 Repository
pub struct RoomFriendRepo;

impl RoomFriendRepo {
    /// 根据 room_key 查询（uid1_uid2，uid1 < uid2）
    pub async fn find_by_room_key(pool: &sqlx::Pool<sqlx::MySql>, room_key: &str) -> Result<Option<RoomFriend>, ImError> {
        let rf = sqlx::query_as::<_, RoomFriend>("SELECT * FROM `room_friend` WHERE room_key = ?")
            .bind(room_key)
            .fetch_optional(pool)
            .await?;
        Ok(rf)
    }

    /// 根据 room_id 查询
    pub async fn find_by_room_id(pool: &sqlx::Pool<sqlx::MySql>, room_id: i64) -> Result<Option<RoomFriend>, ImError> {
        let rf = sqlx::query_as::<_, RoomFriend>("SELECT * FROM `room_friend` WHERE room_id = ?")
            .bind(room_id)
            .fetch_optional(pool)
            .await?;
        Ok(rf)
    }

    /// 批量查询（按 room_id 列表）
    pub async fn find_by_room_ids(pool: &sqlx::Pool<sqlx::MySql>, room_ids: &[i64]) -> Result<Vec<RoomFriend>, ImError> {
        if room_ids.is_empty() {
            return Ok(vec![]);
        }
        let builder = sqlxplus::QueryBuilder::new("SELECT * FROM `room_friend`")
            .and_in("room_id", room_ids.to_vec());
        let list = RoomFriend::find_all(pool, Some(builder)).await?;
        Ok(list)
    }
}

/// 群聊房间 Repository
pub struct RoomGroupRepo;

impl RoomGroupRepo {
    /// 根据 room_id 查询群信息
    pub async fn find_by_room_id(pool: &sqlx::Pool<sqlx::MySql>, room_id: i64) -> Result<Option<RoomGroup>, ImError> {
        let builder = sqlxplus::QueryBuilder::new("SELECT * FROM `room_group`")
            .and_eq("room_id", room_id);
        let rg = RoomGroup::find_one(pool, builder).await?;
        Ok(rg)
    }

    /// 批量查询（按 room_id 列表）
    pub async fn find_by_room_ids(pool: &sqlx::Pool<sqlx::MySql>, room_ids: &[i64]) -> Result<Vec<RoomGroup>, ImError> {
        if room_ids.is_empty() {
            return Ok(vec![]);
        }
        let builder = sqlxplus::QueryBuilder::new("SELECT * FROM `room_group`")
            .and_in("room_id", room_ids.to_vec());
        let list = RoomGroup::find_all(pool, Some(builder)).await?;
        Ok(list)
    }
}

/// 群成员 Repository
pub struct GroupMemberRepo;

impl GroupMemberRepo {
    /// 查询群的所有成员
    pub async fn find_by_group_id(pool: &sqlx::Pool<sqlx::MySql>, group_id: i64) -> Result<Vec<GroupMember>, ImError> {
        let builder = sqlxplus::QueryBuilder::new("SELECT * FROM `group_member`")
            .and_eq("group_id", group_id);
        let list = GroupMember::find_all(pool, Some(builder)).await?;
        Ok(list)
    }

    /// 查询用户在某群的成员信息
    pub async fn find_member(pool: &sqlx::Pool<sqlx::MySql>, group_id: i64, uid: i64) -> Result<Option<GroupMember>, ImError> {
        let builder = sqlxplus::QueryBuilder::new("SELECT * FROM `group_member`")
            .and_eq("group_id", group_id)
            .and_eq("uid", uid);
        let list = GroupMember::find_all(pool, Some(builder)).await?;
        Ok(list.into_iter().next())
    }

    /// 删除群成员
    pub async fn delete(pool: &sqlx::Pool<sqlx::MySql>, group_id: i64, uid: i64) -> Result<(), ImError> {
        sqlx::query("DELETE FROM `group_member` WHERE `group_id` = ? AND `uid` = ?")
            .bind(group_id)
            .bind(uid)
            .execute(pool)
            .await?;
        Ok(())
    }

    /// 查询用户加入的所有群
    pub async fn find_groups_by_uid(pool: &sqlx::Pool<sqlx::MySql>, uid: i64) -> Result<Vec<GroupMember>, ImError> {
        let builder = sqlxplus::QueryBuilder::new("SELECT * FROM `group_member`")
            .and_eq("uid", uid);
        let list = GroupMember::find_all(pool, Some(builder)).await?;
        Ok(list)
    }
}

