// Plan 模块 HTTP 处理器

use crate::context::RequestContext;
use crate::error::{IdentityError, to_err};
use crate::modules::plan::*;
use crate::state::AppState;
use axum::{
    extract::{Path, Query, State},
    response::Json,
};
use fbc_starter::{base::CursorPageBaseResp, R};
use serde::Deserialize;
use std::sync::Arc;

/// 获取套餐信息
pub async fn get_plan(
    State(app_state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<Json<R<PlanInfo>>, IdentityError> {
    let plan = app_state.plan_service.get_plan_info(id).await.map_err(to_err)?;
    Ok(Json(R::ok_with_data(PlanInfo::from(plan))))
}

/// 创建套餐
pub async fn create_plan(
    State(app_state): State<Arc<AppState>>,
    context: RequestContext,
    axum::Json(req): axum::Json<CreatePlanRequest>,
) -> Result<Json<R<CreatePlanResponse>>, IdentityError> {
    let plan_id = app_state
        .plan_service
        .create_plan(
            &req.name,
            &req.r#type,
            &req.price,
            &req.billing_cycle,
            req.description.as_deref(),
            req.is_active,
            req.sort_order,
            Some(context.user_id),
        )
        .await
        .map_err(to_err)?;
    Ok(Json(R::ok_with_data(CreatePlanResponse { plan_id })))
}

/// 更新套餐
pub async fn update_plan(
    State(app_state): State<Arc<AppState>>,
    context: RequestContext,
    Path(id): Path<i64>,
    axum::Json(req): axum::Json<UpdatePlanRequest>,
) -> Result<Json<R<()>>, IdentityError> {
    app_state
        .plan_service
        .update_plan(
            id,
            req.name.as_deref(),
            req.r#type.as_deref(),
            req.price.as_deref(),
            req.billing_cycle.as_deref(),
            req.description.as_deref(),
            req.is_active,
            req.sort_order,
            Some(context.user_id),
        )
        .await
        .map_err(to_err)?;
    Ok(Json(R::ok()))
}

/// 删除套餐
pub async fn delete_plan(
    State(app_state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<Json<R<()>>, IdentityError> {
    app_state.plan_service.delete_plan(id).await.map_err(to_err)?;
    Ok(Json(R::ok()))
}

/// 分页查询所有套餐（不过滤激活状态）
pub async fn list_plans(
    State(app_state): State<Arc<AppState>>,
    Query(req): Query<ListPlansRequest>,
) -> Result<Json<R<CursorPageBaseResp<PlanInfo>>>, IdentityError> {
    let page = req.page.cursor.unwrap_or(1);
    let page_size = req.page.page_size;

    let (plans, total) = app_state
        .plan_service
        .list_plans(
            page,
            page_size,
            req.search_key.as_deref(),
            req.exclude_subscribed_tenant_id,
        )
        .await
        .map_err(to_err)?;

    let list: Vec<PlanInfo> = plans.into_iter().map(PlanInfo::from).collect();
    let next_cursor = if list.len() as u32 >= page_size {
        Some((page as u32) * page_size)
    } else {
        None
    };
    let has_next = next_cursor.is_some() && ((page as i64) * (page_size as i64)) < total;

    Ok(Json(R::ok_with_data(CursorPageBaseResp::init(
        next_cursor,
        has_next,
        list,
        total,
    ))))
}

/// 创建套餐权益
pub async fn create_plan_entitlement(
    State(app_state): State<Arc<AppState>>,
    context: RequestContext,
    axum::Json(req): axum::Json<CreatePlanEntitlementRequest>,
) -> Result<Json<R<CreatePlanResponse>>, IdentityError> {
    let entitlement_id = app_state
        .plan_entitlement_service
        .create_plan_entitlement(
            req.plan_id,
            &req.entitlement_key,
            &req.entitlement_value,
            &req.value_type,
            req.description.as_deref(),
            Some(context.user_id),
        )
        .await
        .map_err(to_err)?;
    Ok(Json(R::ok_with_data(CreatePlanResponse {
        plan_id: entitlement_id,
    })))
}

/// 获取套餐的所有权益
pub async fn get_plan_entitlements(
    State(app_state): State<Arc<AppState>>,
    Path(plan_id): Path<i64>,
) -> Result<Json<R<Vec<PlanEntitlementInfo>>>, IdentityError> {
    let entitlements = app_state
        .plan_entitlement_service
        .get_plan_entitlements(plan_id)
        .await
        .map_err(to_err)?;
    let infos = entitlements
        .into_iter()
        .map(PlanEntitlementInfo::from)
        .collect();
    Ok(Json(R::ok_with_data(infos)))
}

/// 更新套餐权益
pub async fn update_plan_entitlement(
    State(app_state): State<Arc<AppState>>,
    context: RequestContext,
    Path(id): Path<i64>,
    axum::Json(req): axum::Json<UpdatePlanEntitlementRequest>,
) -> Result<Json<R<()>>, IdentityError> {
    app_state
        .plan_entitlement_service
        .update_plan_entitlement(
            id,
            req.entitlement_key.as_deref(),
            req.entitlement_value.as_deref(),
            req.value_type.as_deref(),
            req.description.as_deref(),
            Some(context.user_id),
        )
        .await
        .map_err(to_err)?;
    Ok(Json(R::ok()))
}

/// 删除套餐权益
pub async fn delete_plan_entitlement(
    State(app_state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<Json<R<()>>, IdentityError> {
    app_state
        .plan_entitlement_service
        .delete_plan_entitlement(id)
        .await
        .map_err(to_err)?;
    Ok(Json(R::ok()))
}

/// 创建租户订阅
pub async fn create_subscription(
    State(app_state): State<Arc<AppState>>,
    context: RequestContext,
    axum::Json(req): axum::Json<CreateTenantSubscriptionRequest>,
) -> Result<Json<R<CreateTenantSubscriptionResponse>>, IdentityError> {
    let subscription_id = app_state
        .tenant_subscription_service
        .create_subscription(
            req.tenant_id,
            req.plan_id,
            req.start_at,
            req.expire_at,
            req.auto_renew,
            Some(context.user_id),
        )
        .await
        .map_err(to_err)?;
    Ok(Json(R::ok_with_data(CreateTenantSubscriptionResponse {
        subscription_id,
    })))
}

/// 更新租户订阅
pub async fn update_subscription(
    State(app_state): State<Arc<AppState>>,
    context: RequestContext,
    Path(tenant_id): Path<i64>,
    axum::Json(req): axum::Json<UpdateTenantSubscriptionRequest>,
) -> Result<Json<R<()>>, IdentityError> {
    app_state
        .tenant_subscription_service
        .update_subscription(
            tenant_id,
            req.plan_id,
            req.status.as_deref(),
            req.start_at,
            req.expire_at,
            req.auto_renew,
            Some(context.user_id),
        )
        .await
        .map_err(to_err)?;
    Ok(Json(R::ok()))
}

/// 获取租户所有订阅信息（包含套餐信息）
pub async fn get_subscriptions(
    State(app_state): State<Arc<AppState>>,
    Path(tenant_id): Path<i64>,
) -> Result<Json<R<Vec<TenantSubscriptionInfo>>>, IdentityError> {
    let infos = app_state
        .tenant_subscription_service
        .get_subscriptions_with_plan(tenant_id)
        .await
        .map_err(to_err)?;
    Ok(Json(R::ok_with_data(infos)))
}

/// 获取租户当前激活的订阅信息（包含套餐信息）
pub async fn get_active_subscription(
    State(app_state): State<Arc<AppState>>,
    Path(tenant_id): Path<i64>,
) -> Result<Json<R<Option<TenantSubscriptionInfo>>>, IdentityError> {
    let info = app_state
        .tenant_subscription_service
        .get_active_subscription_with_plan(tenant_id)
        .await
        .map_err(to_err)?;
    Ok(Json(R::ok_with_data(info)))
}

/// 记录用量
pub async fn record_usage(
    State(app_state): State<Arc<AppState>>,
    axum::Json(req): axum::Json<RecordUsageRequest>,
) -> Result<Json<R<()>>, IdentityError> {
    app_state
        .tenant_usage_service
        .record_usage(
            req.tenant_id,
            &req.entitlement_key,
            req.delta,
            &req.source,
            req.ref_id.as_deref(),
        )
        .await
        .map_err(to_err)?;
    Ok(Json(R::ok()))
}

/// 获取租户用量
pub async fn get_tenant_usage(
    State(app_state): State<Arc<AppState>>,
    Path(tenant_id): Path<i64>,
) -> Result<Json<R<Vec<TenantUsageInfo>>>, IdentityError> {
    let usages = app_state
        .tenant_usage_service
        .get_tenant_usage(tenant_id)
        .await
        .map_err(to_err)?;
    let infos = usages.into_iter().map(TenantUsageInfo::from).collect();
    Ok(Json(R::ok_with_data(infos)))
}

#[derive(Debug, Deserialize)]
pub struct GetUsageLogsQuery {
    pub entitlement_key: Option<String>,
    pub limit: Option<u64>,
}

/// 获取租户用量日志
pub async fn get_usage_logs(
    State(app_state): State<Arc<AppState>>,
    Path(tenant_id): Path<i64>,
    Query(query): Query<GetUsageLogsQuery>,
) -> Result<Json<R<Vec<TenantUsageLogInfo>>>, IdentityError> {
    let logs = app_state
        .tenant_usage_service
        .get_usage_logs(tenant_id, query.entitlement_key.as_deref(), query.limit)
        .await
        .map_err(to_err)?;
    let infos = logs.into_iter().map(TenantUsageLogInfo::from).collect();
    Ok(Json(R::ok_with_data(infos)))
}

/// 取消订阅（退订）
pub async fn cancel_subscription(
    State(app_state): State<Arc<AppState>>,
    context: RequestContext,
    Path(tenant_id): Path<i64>,
) -> Result<Json<R<()>>, IdentityError> {
    app_state
        .tenant_subscription_service
        .cancel_subscription(tenant_id, Some(context.user_id))
        .await
        .map_err(to_err)?;
    Ok(Json(R::ok_with_data(())))
}
