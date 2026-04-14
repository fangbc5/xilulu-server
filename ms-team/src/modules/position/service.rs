use super::model::dto::{CreatePositionRequest, ListPositionsQuery, UpdatePositionRequest};
use super::model::entity::Position;
use super::repository::PositionRepo;
use crate::error::{OrganizationError, Result};
use crate::modules::employee::EmployeePositionRepo;
use crate::modules::organization::Organization;
use chrono::Utc;
use sqlxplus::Crud;
use sqlxplus::DbPool;
use sqlxplus::QueryBuilder;
use std::sync::Arc;

/// 岗位 Service
pub struct PositionService {
    db_pool: Arc<DbPool>,
}

impl PositionService {
    /// 创建新的 PositionService
    pub fn new(db_pool: Arc<DbPool>) -> Self {
        Self { db_pool }
    }

    /// 分页查询岗位
    pub async fn find_page(
        &self,
        tenant_id: i64,
        req: ListPositionsQuery,
    ) -> Result<(Vec<Position>, i64)> {
        let mut builder =
            QueryBuilder::new("SELECT * FROM position").and_eq("tenant_id", tenant_id);

        if let Some(org_id) = req.org_id {
            builder = builder.and_eq("org_id", org_id);
        }

        if let Some(category) = req.category {
            builder = builder.and_eq("category", category);
        }

        if let Some(ref keyword) = req.keyword {
            builder = builder.and_group(|mut g| {
                g = g.or_like("name", format!("%{}%", keyword));
                g = g.or_like("code", format!("%{}%", keyword));
                g
            });
        }

        if let Some(status) = req.status {
            builder = builder.and_eq("status", status);
        }

        // Default sort by sort_order, then created_at
        builder = builder
            .order_by("sort_order", false)
            .order_by("created_at", true);

        let page_num = req.page.cursor.unwrap_or(1);
        let result = Position::paginate(
            self.db_pool.mysql_pool(),
            builder,
            page_num,
            req.page.page_size,
        )
        .await
        .map_err(|e| OrganizationError::DatabaseError(e.to_string()))?;

        Ok((result.items, result.total as i64))
    }

    /// 创建岗位
    pub async fn create(
        &self,
        tenant_id: i64,
        req: CreatePositionRequest,
        created_by: Option<i64>,
    ) -> Result<i64> {
        // 检查编码是否已存在
        if let Some(_) =
            PositionRepo::find_by_org_and_code(self.db_pool.mysql_pool(), req.org_id, &req.code)
                .await?
        {
            return Err(OrganizationError::PositionExists(req.code).into());
        }

        // ✏️ 检查组织是否存在
        if let None = Organization::find_by_id(self.db_pool.mysql_pool(), req.org_id)
            .await
            .map_err(|e| OrganizationError::DatabaseError(e.to_string()))?
        {
            return Err(OrganizationError::OrganizationNotFound.into());
        }

        let now = Utc::now();
        let mut pos = Position::default();
        pos.tenant_id = tenant_id;
        pos.org_id = req.org_id;
        pos.code = req.code;
        pos.name = req.name;
        pos.category = req.category;
        pos.level = req.level;
        pos.description = req.description;
        pos.requirements = req.requirements;
        pos.sort_order = req.sort_order.or(Some(0));
        pos.status = Some(1);
        pos.created_by = created_by;
        pos.created_at = Some(now);
        pos.updated_at = Some(now);

        let id = pos
            .insert(self.db_pool.mysql_pool())
            .await
            .map_err(|e| OrganizationError::DatabaseError(e.to_string()))?;

        Ok(id)
    }

    /// 获取岗位详情
    pub async fn get_by_id(&self, id: i64) -> Result<Position> {
        let pos = Position::find_by_id(self.db_pool.mysql_pool(), id)
            .await
            .map_err(|e| OrganizationError::DatabaseError(e.to_string()))?;
        pos.ok_or_else(|| OrganizationError::PositionNotFound.into())
    }

    /// 获取组织下的所有岗位
    pub async fn list_by_org(&self, org_id: i64) -> Result<Vec<Position>> {
        PositionRepo::find_by_org_id(self.db_pool.mysql_pool(), org_id).await
    }

    /// 更新岗位
    pub async fn update(
        &self,
        id: i64,
        req: UpdatePositionRequest,
        updated_by: Option<i64>,
    ) -> Result<()> {
        let existing = self.get_by_id(id).await?;

        let mut pos = Position::default();
        pos.id = Some(id);
        pos.name = req.name.unwrap_or(existing.name);
        pos.category = req.category.or(existing.category);
        pos.level = req.level.or(existing.level);
        pos.description = req.description.or(existing.description);
        pos.requirements = req.requirements.or(existing.requirements);
        pos.sort_order = req.sort_order.or(existing.sort_order);
        pos.status = req.status.or(existing.status);
        pos.updated_by = updated_by;
        pos.updated_at = Some(Utc::now());

        pos.update(self.db_pool.mysql_pool())
            .await
            .map_err(|e| OrganizationError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// 删除岗位
    pub async fn delete(&self, id: i64) -> Result<()> {
        let _ = self.get_by_id(id).await?;

        // 检查是否有员工在该岗位
        if EmployeePositionRepo::has_employees(self.db_pool.mysql_pool(), id).await? {
            return Err(OrganizationError::PositionHasEmployees.into());
        }

        Position::delete_by_id(self.db_pool.mysql_pool(), id)
            .await
            .map_err(|e| OrganizationError::DatabaseError(e.to_string()))?;

        Ok(())
    }
}
