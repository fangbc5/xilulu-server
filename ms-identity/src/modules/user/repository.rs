// 用户模块 Repository 层
// 负责用户相关的数据访问操作

use crate::error::IdentityError;
// 使用模块重新导出的类型
use crate::modules::user::{TenantUserRel, User, UserRole};
use anyhow::Result;
use sqlxplus::Crud;

/// 用户相关的数据访问操作
pub struct UserRepo;

impl UserRepo {
    /// 根据用户名查找用户
    pub async fn find_by_username(pool: &sqlx::Pool<sqlx::MySql>, username: &str) -> Result<User> {
        // sqlxplus 会自动处理软删除字段和 limit(1)
        let builder =
            sqlxplus::QueryBuilder::new("SELECT * FROM `user`").and_eq("username", username);

        let user = User::find_one(pool, builder)
            .await
            .map_err(|e| IdentityError::DatabaseError(e.to_string()))?;

        user.ok_or_else(|| anyhow::Error::from(IdentityError::UserNotFound))
    }

    /// 根据邮箱查找用户
    pub async fn find_by_email(pool: &sqlx::Pool<sqlx::MySql>, email: &str) -> Result<User> {
        // sqlxplus 会自动处理软删除字段和 limit(1)
        let builder = sqlxplus::QueryBuilder::new("SELECT * FROM `user`").and_eq("email", email);

        let user = User::find_one(pool, builder)
            .await
            .map_err(|e| IdentityError::DatabaseError(e.to_string()))?;

        user.ok_or_else(|| anyhow::Error::from(IdentityError::UserNotFound))
    }

    /// 根据手机号查找用户
    pub async fn find_by_mobile(pool: &sqlx::Pool<sqlx::MySql>, mobile: &str) -> Result<User> {
        // sqlxplus 会自动处理软删除字段和 limit(1)
        let builder = sqlxplus::QueryBuilder::new("SELECT * FROM `user`").and_eq("mobile", mobile);

        let user = User::find_one(pool, builder)
            .await
            .map_err(|e| IdentityError::DatabaseError(e.to_string()))?;

        user.ok_or_else(|| anyhow::Error::from(IdentityError::UserNotFound))
    }

    /// 检查用户名是否存在
    pub async fn exists_by_username(
        pool: &sqlx::Pool<sqlx::MySql>,
        username: &str,
    ) -> Result<bool> {
        // sqlxplus 会自动处理软删除字段
        let builder = sqlxplus::QueryBuilder::new("").and_eq("username", username);
        let count = User::count(pool, builder)
            .await
            .map_err(|e| IdentityError::DatabaseError(e.to_string()))?;

        Ok(count > 0)
    }

    /// 检查邮箱是否存在
    pub async fn exists_by_email(pool: &sqlx::Pool<sqlx::MySql>, email: &str) -> Result<bool> {
        // sqlxplus 会自动处理软删除字段
        let builder = sqlxplus::QueryBuilder::new("").and_eq("email", email);
        let count = User::count(pool, builder)
            .await
            .map_err(|e| IdentityError::DatabaseError(e.to_string()))?;

        Ok(count > 0)
    }

    /// 检查手机号是否存在
    pub async fn exists_by_mobile(pool: &sqlx::Pool<sqlx::MySql>, mobile: &str) -> Result<bool> {
        // sqlxplus 会自动处理软删除字段
        let builder = sqlxplus::QueryBuilder::new("").and_eq("mobile", mobile);
        let count = User::count(pool, builder)
            .await
            .map_err(|e| IdentityError::DatabaseError(e.to_string()))?;

        Ok(count > 0)
    }
}

/// 用户租户关系相关的数据访问操作
pub struct UserTenantRelRepo;

impl UserTenantRelRepo {
    /// 根据用户 ID 查找所有租户关系
    pub async fn find_by_user_id(
        pool: &sqlx::Pool<sqlx::MySql>,
        user_id: i64,
    ) -> Result<Vec<TenantUserRel>> {
        let builder = sqlxplus::QueryBuilder::new("SELECT * FROM `tenant_user_rel`")
            .and_eq("user_id", user_id);

        let rels = TenantUserRel::find_all(pool, Some(builder))
            .await
            .map_err(|e| IdentityError::DatabaseError(e.to_string()))?;

        Ok(rels)
    }

    /// 根据租户 ID 查找所有用户关系
    pub async fn find_by_tenant_id(
        pool: &sqlx::Pool<sqlx::MySql>,
        tenant_id: i64,
    ) -> Result<Vec<TenantUserRel>> {
        let builder = sqlxplus::QueryBuilder::new("SELECT * FROM `tenant_user_rel`")
            .and_eq("tenant_id", tenant_id);

        let rels = TenantUserRel::find_all(pool, Some(builder))
            .await
            .map_err(|e| IdentityError::DatabaseError(e.to_string()))?;

        Ok(rels)
    }

    /// 查找用户和租户的关系
    pub async fn find_by_user_and_tenant(
        pool: &sqlx::Pool<sqlx::MySql>,
        user_id: i64,
        tenant_id: i64,
    ) -> Result<Option<TenantUserRel>> {
        // sqlxplus 会自动处理 limit(1)
        let builder = sqlxplus::QueryBuilder::new("SELECT * FROM `tenant_user_rel`")
            .and_eq("user_id", user_id)
            .and_eq("tenant_id", tenant_id);

        let rel = TenantUserRel::find_one(pool, builder)
            .await
            .map_err(|e| IdentityError::DatabaseError(e.to_string()))?;

        Ok(rel)
    }
}

/// 用户角色相关的数据访问操作
pub struct UserRoleRepo;

impl UserRoleRepo {
    /// 根据用户 ID 和租户 ID 查找所有角色
    pub async fn find_by_user_and_tenant(
        pool: &sqlx::Pool<sqlx::MySql>,
        user_id: i64,
        tenant_id: i64,
    ) -> Result<Vec<UserRole>> {
        let builder = sqlxplus::QueryBuilder::new("SELECT * FROM `user_role`")
            .and_eq("user_id", user_id)
            .and_eq("tenant_id", tenant_id);

        let roles = UserRole::find_all(pool, Some(builder))
            .await
            .map_err(|e| IdentityError::DatabaseError(e.to_string()))?;

        Ok(roles)
    }

    /// 查找用户角色关系
    pub async fn find_by_user_role_and_tenant(
        pool: &sqlx::Pool<sqlx::MySql>,
        user_id: i64,
        role_id: i64,
        tenant_id: i64,
    ) -> Result<Option<UserRole>> {
        let builder = sqlxplus::QueryBuilder::new("SELECT * FROM `user_role`")
            .and_eq("user_id", user_id)
            .and_eq("role_id", role_id)
            .and_eq("tenant_id", tenant_id);

        let role = UserRole::find_one(pool, builder)
            .await
            .map_err(|e| IdentityError::DatabaseError(e.to_string()))?;

        Ok(role)
    }
}
