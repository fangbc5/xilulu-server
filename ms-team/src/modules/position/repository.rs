use super::model::entity::Position;
use crate::error::{OrganizationError, Result};
use sqlx::{MySql, Pool};
use sqlxplus::Crud;

/// 岗位 Repository
pub struct PositionRepo;

impl PositionRepo {
    /// 根据组织ID和编码查找岗位
    pub async fn find_by_org_and_code(
        pool: &Pool<MySql>,
        org_id: i64,
        code: &str,
    ) -> Result<Option<Position>> {
        let builder = sqlxplus::QueryBuilder::new("SELECT * FROM position")
            .and_eq("org_id", org_id)
            .and_eq("code", code);

        let pos = Position::find_one(pool, builder)
            .await
            .map_err(|e| OrganizationError::DatabaseError(e.to_string()))?;
        Ok(pos)
    }

    /// 根据组织ID查找所有岗位
    pub async fn find_by_org_id(pool: &Pool<MySql>, org_id: i64) -> Result<Vec<Position>> {
        let builder = sqlxplus::QueryBuilder::new("SELECT * FROM position")
            .and_eq("org_id", org_id)
            .order_by("sort_order", true)
            .order_by("id", true);

        let positions = Position::find_all(pool, Some(builder))
            .await
            .map_err(|e| OrganizationError::DatabaseError(e.to_string()))?;
        Ok(positions)
    }

    /// 获取岗位数量
    pub async fn count_by_org_id(pool: &Pool<MySql>, org_id: i64) -> Result<i64> {
        let builder =
            sqlxplus::QueryBuilder::new("SELECT * FROM position").and_eq("org_id", org_id);

        let count = Position::count(pool, builder)
            .await
            .map_err(|e| OrganizationError::DatabaseError(e.to_string()))?;
        Ok(count as i64)
    }
}
