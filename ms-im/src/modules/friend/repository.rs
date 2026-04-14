use sqlxplus::Crud;

use crate::error::ImError;

use super::model::{UserApply, UserFriend};

/// 好友关系 Repository
pub struct FriendRepo;

impl FriendRepo {
    /// 检查是否已是好友
    pub async fn is_friend(pool: &sqlx::Pool<sqlx::MySql>, uid: i64, friend_uid: i64) -> Result<bool, ImError> {
        let builder = sqlxplus::QueryBuilder::new("SELECT * FROM `user_friend`")
            .and_eq("uid", uid)
            .and_eq("friend_uid", friend_uid)
            .and_eq("status", 1i16);
        let list = UserFriend::find_all(pool, Some(builder)).await?;
        Ok(!list.is_empty())
    }

    /// 删除好友（逻辑删除，status=2）
    pub async fn delete_friend(pool: &sqlx::Pool<sqlx::MySql>, uid: i64, friend_uid: i64) -> Result<(), ImError> {
        sqlx::query("UPDATE `user_friend` SET `status` = 2 WHERE `uid` = ? AND `friend_uid` = ?")
            .bind(uid)
            .bind(friend_uid)
            .execute(pool)
            .await?;
        Ok(())
    }
}

/// 好友申请 Repository
pub struct ApplyRepo;

impl ApplyRepo {
    /// 查询我的所有申请记录（收到的 + 发送的）
    pub async fn find_all_applies(pool: &sqlx::Pool<sqlx::MySql>, user_id: i64) -> Result<Vec<UserApply>, ImError> {
        let builder = sqlxplus::QueryBuilder::new("SELECT * FROM `user_apply`")
            .and_group(|mut b| {
                b = b.or_eq("target_id", user_id);
                b = b.or_eq("uid", user_id);
                b
            })
            .order_by("created_at", false);
        let list = UserApply::find_all(pool, Some(builder)).await?;
        Ok(list)
    }

    /// 查询待审批的申请
    pub async fn find_pending(pool: &sqlx::Pool<sqlx::MySql>, uid: i64, target_id: i64) -> Result<Option<UserApply>, ImError> {
        let builder = sqlxplus::QueryBuilder::new("SELECT * FROM `user_apply`")
            .and_eq("uid", uid)
            .and_eq("target_id", target_id)
            .and_eq("status", 0i16);
        let list = UserApply::find_all(pool, Some(builder)).await?;
        Ok(list.into_iter().next())
    }

    /// 更新申请状态
    pub async fn update_status(pool: &sqlx::Pool<sqlx::MySql>, id: i64, status: i16) -> Result<(), ImError> {
        let apply = UserApply {
            id: Some(id),
            status: Some(status),
            ..Default::default()
        };
        apply.update(pool).await?;
        Ok(())
    }
}
