// Plan 模块 DTO

use crate::modules::plan::model::entity::*;
use serde::{Deserialize, Serialize};

/// 套餐列表查询请求
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct ListPlansRequest {
    #[serde(flatten)]
    pub page: fbc_starter::base::CursorPageBaseReq,
    /// 搜索关键词（套餐名称、类型、描述）
    pub search_key: Option<String>,
    /// 排除已订阅的租户ID（如果提供，则过滤掉该租户已订阅的套餐）
    pub exclude_subscribed_tenant_id: Option<i64>,
}

/// 创建套餐请求
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreatePlanRequest {
    pub name: String,
    pub r#type: String,
    pub price: String,
    pub billing_cycle: String,
    pub description: Option<String>,
    pub is_active: Option<bool>,
    pub sort_order: Option<i32>,
}

/// 创建套餐响应
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct CreatePlanResponse {
    pub plan_id: i64,
}

/// 更新套餐请求
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdatePlanRequest {
    pub name: Option<String>,
    pub r#type: Option<String>,
    pub price: Option<String>,
    pub billing_cycle: Option<String>,
    pub description: Option<String>,
    pub is_active: Option<bool>,
    pub sort_order: Option<i32>,
}

/// 套餐信息响应
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct PlanInfo {
    pub id: Option<i64>,
    pub name: String,
    pub r#type: String,
    pub price: Option<String>,
    pub billing_cycle: String,
    pub description: Option<String>,
    pub is_active: Option<bool>,
    pub sort_order: Option<i32>,
}

impl From<Plan> for PlanInfo {
    fn from(plan: Plan) -> Self {
        Self {
            id: plan.id,
            name: plan.name,
            r#type: plan.r#type,
            price: plan.price,
            billing_cycle: plan.billing_cycle,
            description: plan.description,
            is_active: plan.is_active,
            sort_order: plan.sort_order,
        }
    }
}

/// 创建套餐权益请求
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreatePlanEntitlementRequest {
    pub plan_id: i64,
    pub entitlement_key: String,
    pub entitlement_value: String,
    pub value_type: String,
    pub description: Option<String>,
}

/// 更新套餐权益请求
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdatePlanEntitlementRequest {
    pub entitlement_key: Option<String>,
    pub entitlement_value: Option<String>,
    pub value_type: Option<String>,
    pub description: Option<String>,
}

/// 套餐权益信息响应
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct PlanEntitlementInfo {
    pub id: Option<i64>,
    pub plan_id: i64,
    pub entitlement_key: String,
    pub entitlement_value: String,
    pub value_type: String,
    pub description: Option<String>,
}

impl From<PlanEntitlement> for PlanEntitlementInfo {
    fn from(entitlement: PlanEntitlement) -> Self {
        Self {
            id: entitlement.id,
            plan_id: entitlement.plan_id,
            entitlement_key: entitlement.entitlement_key,
            entitlement_value: entitlement.entitlement_value,
            value_type: entitlement.value_type,
            description: entitlement.description,
        }
    }
}

/// 创建租户订阅请求
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateTenantSubscriptionRequest {
    pub tenant_id: i64,
    pub plan_id: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_at: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expire_at: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auto_renew: Option<bool>,
}

/// 创建租户订阅响应
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct CreateTenantSubscriptionResponse {
    pub subscription_id: i64,
}

/// 更新租户订阅请求
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateTenantSubscriptionRequest {
    pub plan_id: i64,
    pub status: Option<String>,
    pub start_at: Option<chrono::DateTime<chrono::Utc>>,
    pub expire_at: Option<chrono::DateTime<chrono::Utc>>,
    pub auto_renew: Option<bool>,
}

/// 租户订阅信息响应
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct TenantSubscriptionInfo {
    pub id: Option<i64>,
    pub tenant_id: i64,
    pub plan_id: i64,
    pub status: String,
    pub start_at: chrono::DateTime<chrono::Utc>,
    pub expire_at: chrono::DateTime<chrono::Utc>,
    pub auto_renew: Option<bool>,
    /// 套餐详细信息
    pub plan: Option<PlanInfo>,
}

impl From<TenantSubscription> for TenantSubscriptionInfo {
    fn from(subscription: TenantSubscription) -> Self {
        Self {
            id: subscription.id,
            tenant_id: subscription.tenant_id,
            plan_id: subscription.plan_id,
            status: subscription.status,
            start_at: subscription.start_at,
            expire_at: subscription.expire_at,
            auto_renew: subscription.auto_renew,
            plan: None,
        }
    }
}

/// 租户用量信息响应
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct TenantUsageInfo {
    pub id: Option<i64>,
    pub tenant_id: i64,
    pub plan_id: i64,
    pub entitlement_key: String,
    pub cycle_type: String,
    pub cycle_start: chrono::NaiveDate,
    pub cycle_end: chrono::NaiveDate,
    pub used_value: Option<i64>,
}

impl From<TenantUsage> for TenantUsageInfo {
    fn from(usage: TenantUsage) -> Self {
        Self {
            id: usage.id,
            tenant_id: usage.tenant_id,
            plan_id: usage.plan_id,
            entitlement_key: usage.entitlement_key,
            cycle_type: usage.cycle_type,
            cycle_start: usage.cycle_start,
            cycle_end: usage.cycle_end,
            used_value: usage.used_value,
        }
    }
}

/// 记录用量请求
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct RecordUsageRequest {
    pub tenant_id: i64,
    pub entitlement_key: String,
    pub delta: i64,
    pub source: String,
    pub ref_id: Option<String>,
}

/// 租户用量日志信息响应
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct TenantUsageLogInfo {
    pub id: Option<i64>,
    pub tenant_id: i64,
    pub entitlement_key: String,
    pub delta: i64,
    pub source: String,
    pub ref_id: Option<String>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl From<TenantUsageLog> for TenantUsageLogInfo {
    fn from(log: TenantUsageLog) -> Self {
        Self {
            id: log.id,
            tenant_id: log.tenant_id,
            entitlement_key: log.entitlement_key,
            delta: log.delta,
            source: log.source,
            ref_id: log.ref_id,
            created_at: log.created_at,
        }
    }
}
