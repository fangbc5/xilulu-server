// Plan 模块 Repository 层
// 负责套餐相关的数据访问操作

use crate::error::IdentityError;
use crate::modules::plan::model::entity::*;
use anyhow::Result;
use sqlxplus::Crud;

/// 套餐 Repository
pub struct PlanRepo;

impl PlanRepo {
    /// 根据名称查找套餐
    pub async fn find_by_name(pool: &sqlx::Pool<sqlx::MySql>, name: &str) -> Result<Plan> {
        let builder = sqlxplus::QueryBuilder::new("SELECT * FROM `plan`").and_eq("name", name);

        let plan = Plan::find_one(pool, builder)
            .await
            .map_err(|e| IdentityError::DatabaseError(e.to_string()))?;

        plan.ok_or_else(|| anyhow::Error::from(IdentityError::PlanNotFound))
    }

    /// 检查套餐名称是否存在
    pub async fn exists_by_name(pool: &sqlx::Pool<sqlx::MySql>, name: &str) -> Result<bool> {
        let builder = sqlxplus::QueryBuilder::new("").and_eq("name", name);
        let count = Plan::count(pool, builder)
            .await
            .map_err(|e| IdentityError::DatabaseError(e.to_string()))?;

        Ok(count > 0)
    }
}

/// 套餐权益 Repository
pub struct PlanEntitlementRepo;

impl PlanEntitlementRepo {
    /// 根据套餐 ID 查找所有权益
    pub async fn find_by_plan_id(
        pool: &sqlx::Pool<sqlx::MySql>,
        plan_id: i64,
    ) -> Result<Vec<PlanEntitlement>> {
        let builder = sqlxplus::QueryBuilder::new("SELECT * FROM `plan_entitlement`")
            .and_eq("plan_id", plan_id);

        PlanEntitlement::find_all(pool, Some(builder))
            .await
            .map_err(|e| anyhow::Error::from(IdentityError::DatabaseError(e.to_string())))
    }
}

/// 租户订阅 Repository
pub struct TenantSubscriptionRepo;

impl TenantSubscriptionRepo {
    /// 根据租户 ID 查找所有订阅
    pub async fn find_all_by_tenant_id(
        pool: &sqlx::Pool<sqlx::MySql>,
        tenant_id: i64,
    ) -> Result<Vec<TenantSubscription>> {
        let builder = sqlxplus::QueryBuilder::new("SELECT * FROM `tenant_subscription`")
            .and_eq("tenant_id", tenant_id)
            .order_by("created_at", false);
        Ok(TenantSubscription::find_all(pool, Some(builder)).await?)
    }

    /// 根据租户 ID 查找单个激活的订阅（兼容旧代码）
    pub async fn find_active_by_tenant_id(
        pool: &sqlx::Pool<sqlx::MySql>,
        tenant_id: i64,
    ) -> Result<Option<TenantSubscription>> {
        let builder = sqlxplus::QueryBuilder::new("SELECT * FROM `tenant_subscription`")
            .and_eq("tenant_id", tenant_id)
            .and_eq("status", "active");
        Ok(TenantSubscription::find_one(pool, builder).await?)
    }

    pub async fn find_by_tenant_id_and_plan_id(
        pool: &sqlx::Pool<sqlx::MySql>,
        tenant_id: i64,
        plan_id: i64,
    ) -> Result<Option<TenantSubscription>> {
        let builder = sqlxplus::QueryBuilder::new("SELECT * FROM `tenant_subscription`")
            .and_eq("tenant_id", tenant_id)
            .and_eq("plan_id", plan_id);
        Ok(TenantSubscription::find_one(pool, builder).await?)
    }
}

/// 租户用量 Repository
pub struct TenantUsageRepo;

impl TenantUsageRepo {
    /// 根据租户 ID、权益 key 和周期查找用量
    pub async fn find_by_tenant_and_cycle(
        pool: &sqlx::Pool<sqlx::MySql>,
        tenant_id: i64,
        entitlement_key: &str,
        cycle_start: chrono::NaiveDate,
    ) -> Result<TenantUsage> {
        // 将 NaiveDate 转换为字符串格式用于查询
        let cycle_start_str = cycle_start.format("%Y-%m-%d").to_string();
        let builder = sqlxplus::QueryBuilder::new("SELECT * FROM `tenant_usage`")
            .and_eq("tenant_id", tenant_id)
            .and_eq("entitlement_key", entitlement_key)
            .and_eq("cycle_start", cycle_start_str);

        let usage = TenantUsage::find_one(pool, builder)
            .await
            .map_err(|e| IdentityError::DatabaseError(e.to_string()))?;

        usage.ok_or_else(|| anyhow::Error::from(IdentityError::TenantUsageNotFound))
    }

    /// 根据租户 ID 查找所有用量
    pub async fn find_by_tenant_id(
        pool: &sqlx::Pool<sqlx::MySql>,
        tenant_id: i64,
    ) -> Result<Vec<TenantUsage>> {
        let builder = sqlxplus::QueryBuilder::new("SELECT * FROM `tenant_usage`")
            .and_eq("tenant_id", tenant_id);

        let mut usages = TenantUsage::find_all(pool, Some(builder))
            .await
            .map_err(|e| anyhow::Error::from(IdentityError::DatabaseError(e.to_string())))?;

        // 手动排序
        usages.sort_by(|a, b| b.cycle_start.cmp(&a.cycle_start));

        Ok(usages)
    }
}

/// 租户用量日志 Repository
pub struct TenantUsageLogRepo;

impl TenantUsageLogRepo {
    /// 根据租户 ID 查找用量日志
    pub async fn find_by_tenant_id(
        pool: &sqlx::Pool<sqlx::MySql>,
        tenant_id: i64,
        limit: Option<u64>,
    ) -> Result<Vec<TenantUsageLog>> {
        let builder = sqlxplus::QueryBuilder::new("SELECT * FROM `tenant_usage_log`")
            .and_eq("tenant_id", tenant_id);

        let mut logs = TenantUsageLog::find_all(pool, Some(builder))
            .await
            .map_err(|e| anyhow::Error::from(IdentityError::DatabaseError(e.to_string())))?;

        // 手动排序
        logs.sort_by(|a, b| {
            b.created_at
                .unwrap_or_default()
                .cmp(&a.created_at.unwrap_or_default())
        });

        // 应用 limit
        if let Some(limit_val) = limit {
            logs.truncate(limit_val as usize);
        }

        Ok(logs)
    }

    /// 根据租户 ID 和权益 key 查找用量日志
    pub async fn find_by_tenant_and_key(
        pool: &sqlx::Pool<sqlx::MySql>,
        tenant_id: i64,
        entitlement_key: &str,
        limit: Option<u64>,
    ) -> Result<Vec<TenantUsageLog>> {
        let builder = sqlxplus::QueryBuilder::new("SELECT * FROM `tenant_usage_log`")
            .and_eq("tenant_id", tenant_id)
            .and_eq("entitlement_key", entitlement_key);

        let mut logs = TenantUsageLog::find_all(pool, Some(builder))
            .await
            .map_err(|e| anyhow::Error::from(IdentityError::DatabaseError(e.to_string())))?;

        // 手动排序
        logs.sort_by(|a, b| {
            b.created_at
                .unwrap_or_default()
                .cmp(&a.created_at.unwrap_or_default())
        });

        // 应用 limit
        if let Some(limit_val) = limit {
            logs.truncate(limit_val as usize);
        }

        Ok(logs)
    }
}
