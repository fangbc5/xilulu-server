// 应用状态定义
// 包含所有 Service 实例和共享资源

use crate::config::IdentityConfig;
use crate::modules::auth::{ApplicationService, PermissionService, ResourceService, RoleService};
use crate::modules::plan::{
    PlanEntitlementService, PlanService, TenantSubscriptionService, TenantUsageService,
};
use crate::modules::device::DeviceService;
use crate::modules::tenant::{TenantApplicationService, TenantService};
use crate::modules::user::{UserRoleService, UserService, UserTenantService};
use sqlxplus::DbPool;
use std::sync::Arc;

/// 应用状态
/// 包含所有 Service 实例和共享资源
#[derive(Clone)]
pub struct AppState {
    /// 用户服务
    pub user_service: Arc<UserService>,
    /// 设备服务
    pub device_service: Arc<DeviceService>,
    /// 用户租户关系服务
    pub user_tenant_service: Arc<UserTenantService>,
    /// 用户角色服务
    pub user_role_service: Arc<UserRoleService>,
    /// 租户服务
    pub tenant_service: Arc<TenantService>,
    /// 租户应用关系服务
    pub tenant_application_service: Arc<TenantApplicationService>,
    /// 角色服务
    pub role_service: Arc<RoleService>,
    /// 资源服务
    pub resource_service: Arc<ResourceService>,
    /// 权限服务
    pub permission_service: Arc<PermissionService>,
    /// 应用服务
    pub application_service: Arc<ApplicationService>,
    /// 套餐服务
    pub plan_service: Arc<PlanService>,
    /// 套餐权益服务
    pub plan_entitlement_service: Arc<PlanEntitlementService>,
    /// 租户订阅服务
    pub tenant_subscription_service: Arc<TenantSubscriptionService>,
    /// 租户用量服务
    pub tenant_usage_service: Arc<TenantUsageService>,
}

impl AppState {
    /// 创建新的 AppState（同步方法）
    pub fn new(
        db_pool: Arc<DbPool>,
        config: IdentityConfig,
    ) -> Self {
        Self {
            user_service: Arc::new(UserService::new(
                db_pool.clone(),
                config.identity.password.clone(),
            )),
            device_service: Arc::new(DeviceService::new(db_pool.clone())),
            user_tenant_service: Arc::new(UserTenantService::new(db_pool.clone())),
            user_role_service: Arc::new(UserRoleService::new(db_pool.clone())),
            tenant_service: Arc::new(TenantService::new(db_pool.clone())),
            application_service: {
                let app_service = Arc::new(ApplicationService::new(db_pool.clone()));
                app_service.clone()
            },
            tenant_application_service: {
                let app_service = Arc::new(ApplicationService::new(db_pool.clone()));
                Arc::new(TenantApplicationService::new(
                    db_pool.clone(),
                    app_service.clone(),
                ))
            },
            role_service: Arc::new(RoleService::new(db_pool.clone())),
            resource_service: Arc::new(ResourceService::new(db_pool.clone())),
            permission_service: Arc::new(PermissionService::new(db_pool.clone())),
            plan_service: {
                let plan_service = Arc::new(PlanService::new(db_pool.clone()));
                plan_service.clone()
            },
            plan_entitlement_service: Arc::new(PlanEntitlementService::new(db_pool.clone())),
            tenant_subscription_service: {
                let plan_service = Arc::new(PlanService::new(db_pool.clone()));
                Arc::new(TenantSubscriptionService::new(
                    db_pool.clone(),
                    plan_service.clone(),
                ))
            },
            tenant_usage_service: Arc::new(TenantUsageService::new(db_pool.clone())),
        }
    }
}
