use sqlxplus::Crud;

use crate::error::ImError;

use super::model::Contact;

/// 会话 Repository
pub struct ContactRepo;

impl ContactRepo {
    /// 查询用户的会话列表（按 active_time 降序，排除已删除的）
    pub async fn find_by_uid(
        pool: &sqlx::Pool<sqlx::MySql>,
        uid: i64,
    ) -> Result<Vec<Contact>, ImError> {
        let builder = sqlxplus::QueryBuilder::new("SELECT * FROM `contact`")
            .and_eq("uid", uid)
            .and_eq("is_deleted", 0i16)
            .order_by("is_top", false) // 置顶优先
            .order_by("active_time", false); // 最近活跃优先
        let list = Contact::find_all(pool, Some(builder)).await?;
        Ok(list)
    }

    /// 查询用户在某个房间的会话
    pub async fn find_by_uid_room(
        pool: &sqlx::Pool<sqlx::MySql>,
        uid: i64,
        room_id: i64,
    ) -> Result<Option<Contact>, ImError> {
        let builder = sqlxplus::QueryBuilder::new("SELECT * FROM `contact`")
            .and_eq("uid", uid)
            .and_eq("room_id", room_id);
        let list = Contact::find_all(pool, Some(builder)).await?;
        Ok(list.into_iter().next())
    }

    /// 更新最新已读消息位点并精准扣减视角可见的增量未读数
    pub async fn update_read_offset(
        pool: &sqlx::Pool<sqlx::MySql>,
        uid: i64,
        room_id: i64,
        read_msg_id: i64,
        diff_count: i64,
    ) -> Result<(), ImError> {
        let sql = r#"
            UPDATE `contact`
            SET `read_msg_id` = GREATEST(COALESCE(`read_msg_id`, 0), ?),
                `unread_count` = CASE
                    WHEN ? >= COALESCE(`last_msg_id`, 0) THEN 0
                    ELSE GREATEST(0, CAST(`unread_count` AS SIGNED) - ?)
                END,
                `updated_at` = NOW()
            WHERE `uid` = ? AND `room_id` = ?
        "#;
        let result = sqlx::query(sql)
            .bind(read_msg_id)
            .bind(read_msg_id)
            .bind(diff_count)
            .bind(uid)
            .bind(room_id)
            .execute(pool)
            .await?;
        tracing::info!("update_read_offset: uid={}, room_id={}, read_msg_id={}, diff_count={}, rows_affected={}",
            uid, room_id, read_msg_id, diff_count, result.rows_affected());
        Ok(())
    }

    /// 软删除会话
    pub async fn soft_delete(
        pool: &sqlx::Pool<sqlx::MySql>,
        uid: i64,
        room_id: i64,
    ) -> Result<(), ImError> {
        sqlx::query("UPDATE `contact` SET `is_deleted` = 1, `clear_msg_id` = COALESCE(`last_msg_id`, 0), `updated_at` = NOW() WHERE `uid` = ? AND `room_id` = ?")
            .bind(uid)
            .bind(room_id)
            .execute(pool)
            .await?;
        Ok(())
    }

    /// 标记未读（简单强制将未读数设置为1，触发微信的红点特效）
    pub async fn set_unread(
        pool: &sqlx::Pool<sqlx::MySql>,
        uid: i64,
        room_id: i64,
    ) -> Result<(), ImError> {
        sqlx::query("UPDATE `contact` SET `unread_count` = 1, `updated_at` = NOW() WHERE `uid` = ? AND `room_id` = ?")
            .bind(uid)
            .bind(room_id)
            .execute(pool)
            .await?;
        Ok(())
    }

    /// 恢复已删除的会话
    pub async fn restore(
        pool: &sqlx::Pool<sqlx::MySql>,
        uid: i64,
        room_id: i64,
    ) -> Result<(), ImError> {
        sqlx::query("UPDATE `contact` SET `is_deleted` = 0, `active_time` = NOW(), `updated_at` = NOW() WHERE `uid` = ? AND `room_id` = ?")
            .bind(uid)
            .bind(room_id)
            .execute(pool)
            .await?;
        Ok(())
    }

    /// 查询房间内所有用户 ID（高效：只查 uid 列，排除已删除）
    pub async fn find_uids_by_room_id(
        pool: &sqlx::Pool<sqlx::MySql>,
        room_id: i64,
    ) -> Result<Vec<i64>, ImError> {
        let rows: Vec<(i64,)> =
            sqlx::query_as("SELECT `uid` FROM `contact` WHERE `room_id` = ? AND `is_deleted` = 0")
                .bind(room_id)
                .fetch_all(pool)
                .await?;
        Ok(rows.into_iter().map(|(uid,)| uid).collect())
    }

    /// 批量查询指定用户的在某个房间的会话
    pub async fn find_by_uids_room(
        pool: &sqlx::Pool<sqlx::MySql>,
        uids: &[i64],
        room_id: i64,
    ) -> Result<Vec<Contact>, ImError> {
        if uids.is_empty() {
            return Ok(vec![]);
        }
        let builder = sqlxplus::QueryBuilder::new("SELECT * FROM `contact`")
            .and_in("uid", uids.to_vec())
            .and_eq("room_id", room_id);
        let list = Contact::find_all(pool, Some(builder)).await?;
        Ok(list)
    }
}
