// Plan 模块 Service 层
// 负责套餐相关的业务逻辑

use crate::modules::plan::model::{entity::*, BillingCycle, SubscriptionStatus};
use crate::modules::plan::{repository::*, PlanInfo};
use crate::modules::tenant::Tenant;
use crate::{error::IdentityError, modules::plan::TenantSubscriptionInfo};
use anyhow::Result;
use chrono::{Datelike, NaiveDate, Utc};
use sqlxplus::{Crud, DbPool};
use std::{collections::HashMap, sync::Arc};

/// 套餐 Service
pub struct PlanService {
    db_pool: Arc<DbPool>,
}

impl PlanService {
    /// 创建新的 PlanService
    pub fn new(db_pool: Arc<DbPool>) -> Self {
        Self { db_pool }
    }

    /// 创建套餐
    pub async fn create_plan(
        &self,
        name: &str,
        r#type: &str,
        price: &str,
        billing_cycle: &str,
        description: Option<&str>,
        is_active: Option<bool>,
        sort_order: Option<i32>,
        create_by: Option<i64>,
    ) -> Result<i64> {
        // 检查套餐名称是否已存在
        if PlanRepo::exists_by_name(self.db_pool.mysql_pool(), name).await? {
            return Err(IdentityError::PlanExists.into());
        }

        // 创建套餐实体
        let mut plan = Plan::default();
        plan.name = name.to_string();
        plan.r#type = r#type.to_string();
        plan.price = Some(price.to_string());
        plan.billing_cycle = billing_cycle.to_string();
        plan.description = description.map(|s| s.to_string());
        plan.is_active = is_active.or(Some(true));
        plan.sort_order = sort_order.or(Some(0));
        plan.created_by = create_by;
        plan.created_at = Some(Utc::now());
        plan.updated_at = Some(Utc::now());

        // 保存套餐
        let plan_id = plan
            .insert(self.db_pool.mysql_pool())
            .await
            .map_err(|e| IdentityError::DatabaseError(e.to_string()))?;

        Ok(plan_id)
    }

    /// 更新套餐
    pub async fn update_plan(
        &self,
        plan_id: i64,
        name: Option<&str>,
        r#type: Option<&str>,
        price: Option<&str>,
        billing_cycle: Option<&str>,
        description: Option<&str>,
        is_active: Option<bool>,
        sort_order: Option<i32>,
        update_by: Option<i64>,
    ) -> Result<()> {
        // 如果更新名称，检查是否已存在
        if let Some(name_str) = name {
            let existing_plan = Plan::find_by_id(self.db_pool.mysql_pool(), plan_id)
                .await
                .map_err(|e| IdentityError::DatabaseError(e.to_string()))?
                .ok_or_else(|| anyhow::Error::from(IdentityError::PlanNotFound))?;
            if name_str != existing_plan.name {
                if PlanRepo::exists_by_name(self.db_pool.mysql_pool(), name_str).await? {
                    return Err(IdentityError::PlanExists.into());
                }
            }
        }

        // 获取现有套餐
        let mut existing_plan = Plan::find_by_id(self.db_pool.mysql_pool(), plan_id)
            .await
            .map_err(|e| IdentityError::DatabaseError(e.to_string()))?
            .ok_or_else(|| anyhow::Error::from(IdentityError::PlanNotFound))?;

        // 更新字段
        if let Some(name_str) = name {
            existing_plan.name = name_str.to_string();
        }
        if let Some(type_str) = r#type {
            existing_plan.r#type = type_str.to_string();
        }
        if let Some(price_val) = price {
            existing_plan.price = Some(price_val.to_string());
        }
        if let Some(cycle_str) = billing_cycle {
            existing_plan.billing_cycle = cycle_str.to_string();
        }
        if let Some(desc) = description {
            existing_plan.description = Some(desc.to_string());
        }
        if let Some(active) = is_active {
            existing_plan.is_active = Some(active);
        }
        if let Some(order) = sort_order {
            existing_plan.sort_order = Some(order);
        }
        existing_plan.updated_by = update_by;
        existing_plan.updated_at = Some(Utc::now());

        existing_plan
            .update(self.db_pool.mysql_pool())
            .await
            .map_err(|e| IdentityError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// 删除套餐（逻辑删除）
    pub async fn delete_plan(&self, plan_id: i64) -> Result<()> {
        Plan::delete_by_id(self.db_pool.mysql_pool(), plan_id)
            .await
            .map_err(|e| IdentityError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    /// 获取套餐信息
    pub async fn get_plan_info(&self, plan_id: i64) -> Result<Plan> {
        let plan = Plan::find_by_id(self.db_pool.mysql_pool(), plan_id)
            .await
            .map_err(|e| IdentityError::DatabaseError(e.to_string()))?;
        plan.ok_or_else(|| anyhow::Error::from(IdentityError::PlanNotFound))
    }

    /// 获取套餐信息列表
    pub async fn get_plans_by_ids(&self, plan_ids: Vec<i64>) -> Result<Vec<Plan>> {
        Plan::find_by_ids(self.db_pool.mysql_pool(), plan_ids)
            .await
            .map_err(|e| IdentityError::DatabaseError(e.to_string()).into())
    }

    /// 分页查询所有套餐（不过滤激活状态）
    pub async fn list_plans(
        &self,
        page: u32,
        page_size: u32,
        search_key: Option<&str>,
        exclude_subscribed_tenant_id: Option<i64>,
    ) -> Result<(Vec<Plan>, i64)> {
        use sqlxplus::QueryBuilder;

        // 构建 QueryBuilder（sqlxplus 会自动处理软删除字段 is_del）
        let mut builder = QueryBuilder::new("SELECT * FROM `plan`");

        // 排除已订阅的套餐
        if let Some(exclude_subscribed_tenant_id) = exclude_subscribed_tenant_id {
            if let Some(subscribed_plan) = TenantSubscriptionRepo::find_active_by_tenant_id(
                self.db_pool.mysql_pool(),
                exclude_subscribed_tenant_id,
            )
            .await?
            {
                builder = builder.and_ne("id", subscribed_plan.plan_id);
            }
        }

        // 搜索条件
        if let Some(key) = search_key {
            builder = builder.and_group(|mut builder_and| {
                builder_and = builder_and.or_like("name", key);
                builder_and = builder_and.or_like("type", key);
                builder_and = builder_and.or_like("description", key);
                builder_and
            });
        }

        // 添加排序
        builder = builder.order_by("sort_order", true);

        let result = Plan::paginate(self.db_pool.mysql_pool(), builder, page, page_size)
            .await
            .map_err(|e| IdentityError::DatabaseError(e.to_string()))?;

        let plans = result.items;
        let total = result.total;

        Ok((plans, total))
    }
}

/// 套餐权益 Service
pub struct PlanEntitlementService {
    db_pool: Arc<DbPool>,
}

impl PlanEntitlementService {
    /// 创建新的 PlanEntitlementService
    pub fn new(db_pool: Arc<DbPool>) -> Self {
        Self { db_pool }
    }

    /// 创建套餐权益
    pub async fn create_plan_entitlement(
        &self,
        plan_id: i64,
        entitlement_key: &str,
        entitlement_value: &str,
        value_type: &str,
        description: Option<&str>,
        create_by: Option<i64>,
    ) -> Result<i64> {
        // 验证套餐存在
        Plan::find_by_id(self.db_pool.mysql_pool(), plan_id)
            .await
            .map_err(|e| IdentityError::DatabaseError(e.to_string()))?
            .ok_or_else(|| anyhow::Error::from(IdentityError::PlanNotFound))?;

        // 创建权益实体
        let mut entitlement = PlanEntitlement::default();
        entitlement.plan_id = plan_id;
        entitlement.entitlement_key = entitlement_key.to_string();
        entitlement.entitlement_value = entitlement_value.to_string();
        entitlement.value_type = value_type.to_string();
        entitlement.description = description.map(|s| s.to_string());
        entitlement.created_by = create_by;
        entitlement.created_at = Some(Utc::now());
        entitlement.updated_at = Some(Utc::now());

        // 保存权益
        let entitlement_id = entitlement
            .insert(self.db_pool.mysql_pool())
            .await
            .map_err(|e| IdentityError::DatabaseError(e.to_string()))?;

        Ok(entitlement_id)
    }

    /// 获取套餐的所有权益
    pub async fn get_plan_entitlements(&self, plan_id: i64) -> Result<Vec<PlanEntitlement>> {
        PlanEntitlementRepo::find_by_plan_id(self.db_pool.mysql_pool(), plan_id).await
    }

    /// 更新套餐权益
    pub async fn update_plan_entitlement(
        &self,
        entitlement_id: i64,
        entitlement_key: Option<&str>,
        entitlement_value: Option<&str>,
        value_type: Option<&str>,
        description: Option<&str>,
        update_by: Option<i64>,
    ) -> Result<()> {
        // 查找权益
        let mut entitlement =
            PlanEntitlement::find_by_id(self.db_pool.mysql_pool(), entitlement_id)
                .await
                .map_err(|e| IdentityError::DatabaseError(e.to_string()))?
                .ok_or_else(|| anyhow::Error::from(IdentityError::PlanEntitlementNotFound))?;

        // 更新字段
        if let Some(key) = entitlement_key {
            entitlement.entitlement_key = key.to_string();
        }
        if let Some(value) = entitlement_value {
            entitlement.entitlement_value = value.to_string();
        }
        if let Some(vt) = value_type {
            entitlement.value_type = vt.to_string();
        }
        if let Some(desc) = description {
            entitlement.description = Some(desc.to_string());
        }
        entitlement.updated_by = update_by;
        entitlement.updated_at = Some(Utc::now());

        // 保存更新
        entitlement
            .update(self.db_pool.mysql_pool())
            .await
            .map_err(|e| IdentityError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// 删除套餐权益
    pub async fn delete_plan_entitlement(&self, entitlement_id: i64) -> Result<()> {
        PlanEntitlement::delete_by_id(self.db_pool.mysql_pool(), entitlement_id)
            .await
            .map_err(|e| IdentityError::DatabaseError(e.to_string()))?;
        Ok(())
    }
}

/// 租户订阅 Service
pub struct TenantSubscriptionService {
    db_pool: Arc<DbPool>,
    plan_service: Arc<PlanService>,
}

impl TenantSubscriptionService {
    /// 创建新的 TenantSubscriptionService
    pub fn new(db_pool: Arc<DbPool>, plan_service: Arc<PlanService>) -> Self {
        Self {
            db_pool,
            plan_service,
        }
    }

    /// 创建租户订阅
    pub async fn create_subscription(
        &self,
        tenant_id: i64,
        plan_id: i64,
        start_at: Option<chrono::DateTime<chrono::Utc>>,
        expire_at: Option<chrono::DateTime<chrono::Utc>>,
        auto_renew: Option<bool>,
        create_by: Option<i64>,
    ) -> Result<i64> {
        // 验证租户存在
        Tenant::find_by_id(self.db_pool.mysql_pool(), tenant_id)
            .await
            .map_err(|e| IdentityError::DatabaseError(e.to_string()))?
            .ok_or_else(|| anyhow::Error::from(IdentityError::TenantNotFound))?;

        // 验证套餐存在并获取套餐信息（确保套餐未删除）
        let builder = sqlxplus::QueryBuilder::new("SELECT * FROM `plan`").and_eq("id", plan_id);
        let plan = Plan::find_one(self.db_pool.mysql_pool(), builder)
            .await
            .map_err(|e| IdentityError::DatabaseError(e.to_string()))?
            .ok_or_else(|| anyhow::Error::from(IdentityError::PlanNotFound))?;

        // 根据套餐的计费周期类型自动生成开始时间和到期时间
        let now = Utc::now();
        let cycle = BillingCycle::from(plan.billing_cycle.as_str());

        // 开始时间：
        // - 一次性套餐：如果前端传了 start_at，则使用前端时间，否则使用当前时间
        // - 其他类型：使用当前时间或前端传入的时间（保持兼容性）
        let start = match cycle {
            BillingCycle::OneTime => start_at.unwrap_or(now),
            _ => start_at.unwrap_or(now),
        };

        // 到期时间：
        // - one_time：如果前端传了 expire_at，则使用前端时间，否则默认 1 年
        // - forever：过期时间设置为 MySQL TIMESTAMP 类型最大值（2038-01-19 03:14:07）
        // - 其他：按计费周期自动计算，若前端传入 expire_at 则优先使用
        let expire = match cycle {
            BillingCycle::OneTime => expire_at.unwrap_or(start + chrono::Duration::days(365)),
            BillingCycle::Forever => {
                // MySQL TIMESTAMP 类型的最大值是 2038-01-19 03:14:07
                let max_naive = NaiveDate::from_ymd_opt(2038, 1, 19)
                    .and_then(|d| d.and_hms_opt(3, 14, 7))
                    .expect("构造最大时间戳失败");
                chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(max_naive, Utc)
            }
            BillingCycle::Monthly => expire_at.unwrap_or(start + chrono::Duration::days(30)),
            BillingCycle::Quarterly => expire_at.unwrap_or(start + chrono::Duration::days(90)),
            BillingCycle::Yearly => expire_at.unwrap_or(start + chrono::Duration::days(365)),
        };

        // 在事务中执行所有数据库操作
        let subscription_id = sqlxplus::with_transaction(self.db_pool.as_ref(), |tx| {
            let tenant_id = tenant_id;
            let plan_id = plan_id;
            let create_by = create_by;
            let start = start;
            let expire = expire;
            let auto_renew = auto_renew;

            Box::pin(async move {
                // 查找该租户现有的 active 订阅，如果存在则将其标记为 expired
                // 确保一个租户只有一个 active 订阅
                let builder = sqlxplus::QueryBuilder::new("SELECT * FROM `tenant_subscription`")
                    .and_eq("tenant_id", tenant_id)
                    .and_eq("status", "active");
                if let Some(mut active_subscription) =
                    TenantSubscription::find_one(tx.as_mysql_executor(), builder).await?
                {
                    // 将现有的 active 订阅标记为 expired
                    active_subscription.status = SubscriptionStatus::Expired.as_str().to_string();
                    active_subscription.updated_at = Some(Utc::now());
                    active_subscription.updated_by = create_by;
                    active_subscription.update(tx.as_mysql_executor()).await?;
                }

                // 创建订阅实体
                let mut subscription = TenantSubscription::default();
                subscription.tenant_id = tenant_id;
                subscription.plan_id = plan_id;
                subscription.status = SubscriptionStatus::Active.as_str().to_string();
                subscription.start_at = start;
                subscription.expire_at = expire;
                subscription.auto_renew = auto_renew.or(Some(false));
                subscription.created_by = create_by;
                subscription.created_at = Some(Utc::now());
                subscription.updated_at = Some(Utc::now());

                // 保存订阅
                let subscription_id = subscription.insert(tx.as_mysql_executor()).await?;

                // 更新租户表的套餐ID和到期时间
                let mut tenant = Tenant::find_by_id(tx.as_mysql_executor(), tenant_id)
                    .await?
                    .ok_or_else(|| sqlxplus::SqlxPlusError::from(sqlx::Error::RowNotFound))?;

                tenant.package_id = plan_id;
                tenant.expire_time = expire;
                tenant.update_by = create_by;
                tenant.update_time = Some(Utc::now());

                tenant.update(tx.as_mysql_executor()).await?;

                Ok::<i64, sqlxplus::SqlxPlusError>(subscription_id)
            })
        })
        .await
        .map_err(|e| IdentityError::DatabaseError(e.to_string()))?;

        Ok(subscription_id)
    }

    /// 更新租户订阅
    pub async fn update_subscription(
        &self,
        tenant_id: i64,
        plan_id: i64,
        status: Option<&str>,
        start_at: Option<chrono::DateTime<chrono::Utc>>,
        expire_at: Option<chrono::DateTime<chrono::Utc>>,
        auto_renew: Option<bool>,
        update_by: Option<i64>,
    ) -> Result<()> {
        // 获取现有订阅（获取激活的订阅）
        let mut subscription = TenantSubscriptionRepo::find_by_tenant_id_and_plan_id(
            self.db_pool.mysql_pool(),
            tenant_id,
            plan_id,
        )
        .await?
        .ok_or_else(|| anyhow::Error::from(IdentityError::TenantSubscriptionNotFound))?;

        if let Some(status_str) = status {
            subscription.status = status_str.to_string();
        }
        if let Some(start) = start_at {
            subscription.start_at = start;
        }
        if let Some(expire) = expire_at {
            subscription.expire_at = expire;
        }
        if let Some(renew) = auto_renew {
            subscription.auto_renew = Some(renew);
        }
        subscription.updated_by = update_by;
        subscription.updated_at = Some(Utc::now());

        subscription
            .update(self.db_pool.mysql_pool())
            .await
            .map_err(|e| IdentityError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// 获取租户所有订阅信息（包含套餐详细信息）
    pub async fn get_subscriptions_with_plan(
        &self,
        tenant_id: i64,
    ) -> Result<Vec<TenantSubscriptionInfo>> {
        let subscriptions =
            TenantSubscriptionRepo::find_all_by_tenant_id(self.db_pool.mysql_pool(), tenant_id)
                .await?;

        if subscriptions.is_empty() {
            return Ok(Vec::new());
        }

        // 批量获取所有套餐信息
        let plan_ids: Vec<i64> = subscriptions.iter().map(|s| s.plan_id).collect();
        let plans = self.plan_service.get_plans_by_ids(plan_ids).await?;
        let plans_map: HashMap<i64, Plan> = plans
            .into_iter()
            .filter_map(|plan| plan.id.map(|id| (id, plan)))
            .collect();

        // 构建结果：通过 plan_id 匹配套餐，转换为 TenantSubscriptionInfo
        let mut result = Vec::new();
        for subscription in subscriptions {
            let plan_id = subscription.plan_id;
            let mut info = TenantSubscriptionInfo::from(subscription);
            // 如果套餐信息存在，添加到响应中
            if let Some(plan) = plans_map.get(&plan_id).cloned() {
                info.plan = Some(PlanInfo::from(plan));
            }
            result.push(info);
        }
        Ok(result)
    }

    /// 获取租户当前激活的订阅信息（包含套餐详细信息）
    pub async fn get_active_subscription_with_plan(
        &self,
        tenant_id: i64,
    ) -> Result<Option<TenantSubscriptionInfo>> {
        let subscription = match TenantSubscriptionRepo::find_active_by_tenant_id(
            self.db_pool.mysql_pool(),
            tenant_id,
        )
        .await?
        {
            Some(s) => s,
            None => return Ok(None), // 如果没有激活的订阅，返回 None
        };

        let plan_id = subscription.plan_id;
        let mut info = TenantSubscriptionInfo::from(subscription);

        // 根据订阅中的 plan_id 获取套餐信息
        if let Ok(plan) = self.plan_service.get_plan_info(plan_id).await {
            info.plan = Some(PlanInfo::from(plan));
        }

        Ok(Some(info))
    }

    /// 取消订阅（退订）
    /// 将当前 active 的订阅标记为 canceled
    pub async fn cancel_subscription(&self, tenant_id: i64, update_by: Option<i64>) -> Result<()> {
        // 查找该租户当前的 active 订阅
        let mut active_subscription =
            TenantSubscriptionRepo::find_active_by_tenant_id(self.db_pool.mysql_pool(), tenant_id)
                .await
                .map_err(|e| IdentityError::DatabaseError(e.to_string()))?
                .ok_or_else(|| anyhow::Error::from(IdentityError::TenantSubscriptionNotFound))?;

        // 将 active 订阅标记为 canceled
        active_subscription.status = SubscriptionStatus::Canceled.as_str().to_string();
        active_subscription.updated_at = Some(Utc::now());
        active_subscription.updated_by = update_by;

        active_subscription
            .update(self.db_pool.mysql_pool())
            .await
            .map_err(|e| IdentityError::DatabaseError(e.to_string()))?;

        Ok(())
    }
}

/// 租户用量 Service
pub struct TenantUsageService {
    db_pool: Arc<DbPool>,
}

impl TenantUsageService {
    /// 创建新的 TenantUsageService
    pub fn new(db_pool: Arc<DbPool>) -> Self {
        Self { db_pool }
    }

    /// 记录用量（增加用量统计）
    pub async fn record_usage(
        &self,
        tenant_id: i64,
        entitlement_key: &str,
        delta: i64,
        source: &str,
        ref_id: Option<&str>,
    ) -> Result<()> {
        // 获取租户订阅
        let subscription =
            TenantSubscriptionRepo::find_active_by_tenant_id(self.db_pool.mysql_pool(), tenant_id)
                .await?
                .ok_or_else(|| anyhow::Error::from(IdentityError::TenantSubscriptionNotFound))?;

        // 获取当前周期
        let now = Utc::now().date_naive();
        let (cycle_type, cycle_start, cycle_end) = Self::get_current_cycle(&subscription, now)?;

        // 记录用量日志
        let mut log = TenantUsageLog::default();
        log.tenant_id = tenant_id;
        log.entitlement_key = entitlement_key.to_string();
        log.delta = delta;
        log.source = source.to_string();
        log.ref_id = ref_id.map(|s| s.to_string());
        log.created_at = Some(Utc::now());

        log.insert(self.db_pool.mysql_pool())
            .await
            .map_err(|e| IdentityError::DatabaseError(e.to_string()))?;

        // 更新或创建用量统计
        match TenantUsageRepo::find_by_tenant_and_cycle(
            self.db_pool.mysql_pool(),
            tenant_id,
            entitlement_key,
            cycle_start,
        )
        .await
        {
            Ok(mut usage) => {
                // 更新现有用量
                usage.used_value = Some(usage.used_value.unwrap_or(0) + delta);
                usage.updated_at = Some(Utc::now());
                usage
                    .update(self.db_pool.mysql_pool())
                    .await
                    .map_err(|e| IdentityError::DatabaseError(e.to_string()))?;
            }
            Err(_) => {
                // 创建新用量记录
                let mut usage = TenantUsage::default();
                usage.tenant_id = tenant_id;
                usage.plan_id = subscription.plan_id;
                usage.entitlement_key = entitlement_key.to_string();
                usage.cycle_type = cycle_type;
                usage.cycle_start = cycle_start;
                usage.cycle_end = cycle_end;
                usage.used_value = Some(delta);
                usage.created_at = Some(Utc::now());
                usage.updated_at = Some(Utc::now());

                usage
                    .insert(self.db_pool.mysql_pool())
                    .await
                    .map_err(|e| IdentityError::DatabaseError(e.to_string()))?;
            }
        }

        Ok(())
    }

    /// 获取租户用量
    pub async fn get_tenant_usage(&self, tenant_id: i64) -> Result<Vec<TenantUsage>> {
        TenantUsageRepo::find_by_tenant_id(self.db_pool.mysql_pool(), tenant_id).await
    }

    /// 获取租户用量日志
    pub async fn get_usage_logs(
        &self,
        tenant_id: i64,
        entitlement_key: Option<&str>,
        limit: Option<u64>,
    ) -> Result<Vec<TenantUsageLog>> {
        if let Some(key) = entitlement_key {
            TenantUsageLogRepo::find_by_tenant_and_key(
                self.db_pool.mysql_pool(),
                tenant_id,
                key,
                limit,
            )
            .await
        } else {
            TenantUsageLogRepo::find_by_tenant_id(self.db_pool.mysql_pool(), tenant_id, limit).await
        }
    }

    /// 获取当前周期（根据订阅的计费周期）
    fn get_current_cycle(
        _subscription: &TenantSubscription,
        now: NaiveDate,
    ) -> Result<(String, NaiveDate, NaiveDate)> {
        // 获取套餐信息以确定计费周期
        // 这里简化处理，假设从订阅开始时间计算周期
        let year = now.year();
        let month = now.month();

        // 简化：按月计算周期
        let cycle_start = NaiveDate::from_ymd_opt(year, month, 1)
            .ok_or_else(|| IdentityError::BusinessError("无效的日期".to_string()))?;
        let cycle_end = if month == 12 {
            NaiveDate::from_ymd_opt(year + 1, 1, 1)
                .ok_or_else(|| IdentityError::BusinessError("无效的日期".to_string()))?
                .pred_opt()
                .ok_or_else(|| IdentityError::BusinessError("无效的日期".to_string()))?
        } else {
            NaiveDate::from_ymd_opt(year, month + 1, 1)
                .ok_or_else(|| IdentityError::BusinessError("无效的日期".to_string()))?
                .pred_opt()
                .ok_or_else(|| IdentityError::BusinessError("无效的日期".to_string()))?
        };

        Ok(("monthly".to_string(), cycle_start, cycle_end))
    }
}
