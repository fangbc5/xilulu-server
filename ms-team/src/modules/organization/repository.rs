use super::model::entity::Organization;
use crate::error::{OrganizationError, Result};
use sqlx::{MySql, Pool};
use sqlxplus::Crud;

/// 组织 Repository
pub struct OrganizationRepo;

impl OrganizationRepo {
    /// 根据租户ID和编码查找组织
    pub async fn find_by_tenant_and_code(
        pool: &Pool<MySql>,
        tenant_id: i64,
        code: &str,
    ) -> Result<Option<Organization>> {
        let builder = sqlxplus::QueryBuilder::new("SELECT * FROM organization")
            .and_eq("tenant_id", tenant_id)
            .and_eq("code", code);

        let org = Organization::find_one(pool, builder)
            .await
            .map_err(|e| OrganizationError::DatabaseError(e.to_string()))?;
        Ok(org)
    }

    /// 根据租户ID查找所有组织
    pub async fn find_by_tenant_id(
        pool: &Pool<MySql>,
        tenant_id: i64,
    ) -> Result<Vec<Organization>> {
        let builder = sqlxplus::QueryBuilder::new("SELECT * FROM organization")
            .and_eq("tenant_id", tenant_id)
            .order_by("sort_order", true)
            .order_by("id", true);

        let orgs = Organization::find_all(pool, Some(builder))
            .await
            .map_err(|e| OrganizationError::DatabaseError(e.to_string()))?;
        Ok(orgs)
    }

    /// 检查是否存在子组织
    pub async fn has_children(pool: &Pool<MySql>, org_id: i64) -> Result<bool> {
        let builder =
            sqlxplus::QueryBuilder::new("SELECT * FROM organization").and_eq("parent_id", org_id);

        let count = Organization::count(pool, builder)
            .await
            .map_err(|e| OrganizationError::DatabaseError(e.to_string()))?;
        Ok(count > 0)
    }

    /// 根据租户ID获取组织数量
    pub async fn count_by_tenant_id(pool: &Pool<MySql>, tenant_id: i64) -> Result<i64> {
        let builder = sqlxplus::QueryBuilder::new("SELECT * FROM organization")
            .and_eq("tenant_id", tenant_id);

        let count = Organization::count(pool, builder)
            .await
            .map_err(|e| OrganizationError::DatabaseError(e.to_string()))?;
        Ok(count as i64)
    }
}
