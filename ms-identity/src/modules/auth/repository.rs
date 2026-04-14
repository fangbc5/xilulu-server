// 权限模块 Repository 层
// 负责权限相关的数据访问操作

use crate::error::IdentityError;
// 使用模块重新导出的类型（通过 auth 模块的重新导出）
use crate::modules::auth::{Resource, Role, RoleResourceRel};
use anyhow::Result;
use sqlxplus::Crud;

/// 角色 Repository
pub struct RoleRepo;

impl RoleRepo {
    /// 根据租户 ID 和角色代码查找角色
    pub async fn find_by_code(
        pool: &sqlx::Pool<sqlx::MySql>,
        code: &str,
        tenant_id: i64,
    ) -> Result<Role> {
        // sqlxplus 会自动处理软删除字段和 limit(1)
        let builder = sqlxplus::QueryBuilder::new("SELECT * FROM `role`")
            .and_eq("code", code)
            .and_eq("tenant_id", tenant_id);

        let role = Role::find_one(pool, builder)
            .await
            .map_err(|e| IdentityError::DatabaseError(e.to_string()))?;

        role.ok_or_else(|| anyhow::Error::from(IdentityError::RoleNotFound))
    }

    /// 根据租户 ID 查找所有角色
    pub async fn find_by_tenant_id(
        pool: &sqlx::Pool<sqlx::MySql>,
        tenant_id: i64,
    ) -> Result<Vec<Role>> {
        // sqlxplus 会自动处理软删除字段
        let builder =
            sqlxplus::QueryBuilder::new("SELECT * FROM `role`").and_eq("tenant_id", tenant_id);

        let roles = Role::find_all(pool, Some(builder))
            .await
            .map_err(|e| IdentityError::DatabaseError(e.to_string()))?;

        Ok(roles)
    }
}

/// 资源 Repository
pub struct ResourceRepo;

impl ResourceRepo {
    /// 根据应用 ID 查找所有资源
    pub async fn find_by_application_id(
        pool: &sqlx::Pool<sqlx::MySql>,
        app_id: i64,
    ) -> Result<Vec<Resource>> {
        // sqlxplus 会自动处理软删除字段
        let builder = sqlxplus::QueryBuilder::new("SELECT * FROM `resource`")
            .and_eq("application_id", app_id);

        let resources = Resource::find_all(pool, Some(builder))
            .await
            .map_err(|e| IdentityError::DatabaseError(e.to_string()))?;

        Ok(resources)
    }


}

/// 角色资源关系 Repository
pub struct RoleResourceRelRepo;

impl RoleResourceRelRepo {
    /// 根据角色 ID 查找所有资源关系
    pub async fn find_by_role_id(
        pool: &sqlx::Pool<sqlx::MySql>,
        role_id: i64,
    ) -> Result<Vec<RoleResourceRel>> {
        // sqlxplus 会自动处理软删除字段
        let builder = sqlxplus::QueryBuilder::new("SELECT * FROM `role_resource_rel`")
            .and_eq("role_id", role_id);

        let rels = RoleResourceRel::find_all(pool, Some(builder))
            .await
            .map_err(|e| IdentityError::DatabaseError(e.to_string()))?;

        Ok(rels)
    }



    /// 查找角色和资源的关系
    pub async fn find_by_role_and_resource(
        pool: &sqlx::Pool<sqlx::MySql>,
        role_id: i64,
        resource_id: i64,
    ) -> Result<Option<RoleResourceRel>> {
        // sqlxplus 会自动处理软删除字段和 limit(1)
        let builder = sqlxplus::QueryBuilder::new("SELECT * FROM `role_resource_rel`")
            .and_eq("role_id", role_id)
            .and_eq("resource_id", resource_id);

        let rel = RoleResourceRel::find_one(pool, builder)
            .await
            .map_err(|e| IdentityError::DatabaseError(e.to_string()))?;

        Ok(rel)
    }
}


