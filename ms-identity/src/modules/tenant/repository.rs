// 租户模块 Repository 层
// 负责租户相关的数据访问操作

use crate::error::IdentityError;
// 使用模块重新导出的类型
use crate::modules::tenant::{Tenant, TenantApplicationRel};
use anyhow::Result;
use sqlxplus::Crud;

/// 租户 Repository
pub struct TenantRepo;

impl TenantRepo {

    /// 检查租户名称是否存在
    pub async fn exists_by_name(pool: &sqlx::Pool<sqlx::MySql>, name: &str) -> Result<bool> {
        // sqlxplus 会自动处理软删除字段
        let builder = sqlxplus::QueryBuilder::new("").and_eq("name", name);
        let count = Tenant::count(pool, builder)
            .await
            .map_err(|e| IdentityError::DatabaseError(e.to_string()))?;

        Ok(count > 0)
    }
}

/// 租户应用关系 Repository
pub struct TenantApplicationRelRepo;

impl TenantApplicationRelRepo {
    /// 根据租户 ID 查找所有应用关系
    pub async fn find_by_tenant_id(
        pool: &sqlx::Pool<sqlx::MySql>,
        tenant_id: i64,
    ) -> Result<Vec<TenantApplicationRel>> {
        let builder = sqlxplus::QueryBuilder::new("SELECT * FROM `tenant_application_rel`")
            .and_eq("tenant_id", tenant_id);

        let rels = TenantApplicationRel::find_all(pool, Some(builder))
            .await
            .map_err(|e| IdentityError::DatabaseError(e.to_string()))?;

        Ok(rels)
    }



    /// 查找租户和应用的关系
    pub async fn find_by_tenant_and_application(
        pool: &sqlx::Pool<sqlx::MySql>,
        tenant_id: i64,
        app_id: i64,
    ) -> Result<Option<TenantApplicationRel>> {
        // sqlxplus 会自动处理 limit(1)
        let builder = sqlxplus::QueryBuilder::new("SELECT * FROM `tenant_application_rel`")
            .and_eq("tenant_id", tenant_id)
            .and_eq("application_id", app_id);

        let rel = TenantApplicationRel::find_one(pool, builder)
            .await
            .map_err(|e| IdentityError::DatabaseError(e.to_string()))?;

        Ok(rel)
    }
}
