// HTTP 路由定义

use crate::modules::{auth, device, plan, tenant, user};
use crate::state::AppState;
use axum::{
    routing::{delete, get, post, put},
    Router,
};
use std::sync::Arc;

/// 创建 HTTP 路由
pub fn create_router(app_state: Arc<AppState>) -> Router {
    Router::new()
        // API v1 路由（身份管理服务）
        .nest(
            "/api/v1/identity",
            Router::new()
                // 用户管理路由
                .nest(
                    "/users",
                    Router::new()
                        .route("/", get(user::list_users))
                        .route("/", post(user::create_user))
                        .route("/{id}", get(user::get_user))
                        .route("/{id}", put(user::update_user))
                        .route("/{id}", delete(user::delete_user))
                        .route("/{id}/password", put(user::change_password))
                        .route("/{id}/password/reset", put(user::reset_password))
                        .route("/count", get(user::get_user_count))
                        .route("/count/active", get(user::get_active_user_count))
                        .route("/{id}/tenants", get(user::get_user_tenants))
                        .route("/{id}/tenants", post(user::add_user_to_tenant))
                        .route("/{id}/tenants/default", put(user::set_default_tenant))
                        .route("/{id}/tenants", delete(user::remove_user_from_tenant))
                        .route(
                            "/{user_id}/tenants/{tenant_id}/roles",
                            get(user::get_user_roles),
                        )
                        .route(
                            "/{user_id}/tenants/{tenant_id}/roles",
                            post(user::assign_role_to_user),
                        )
                        .route(
                            "/{user_id}/tenants/{tenant_id}/roles/batch",
                            post(user::batch_assign_roles_to_user),
                        )
                        .route(
                            "/{user_id}/tenants/{tenant_id}/roles/{role_id}",
                            delete(user::remove_role_from_user),
                        ),
                )
                // 设备管理路由
                .nest(
                    "/devices",
                    Router::new()
                        .route("/register", post(device::register_device))
                        .route("/unregister", post(device::unregister_device))
                        .route("/", get(device::get_my_devices)),
                )
                // 租户管理路由
                .nest(
                    "/tenants",
                    Router::new()
                        .route("/count", get(tenant::get_tenant_count))
                        .route("/", get(tenant::list_tenants))
                        .route("/", post(tenant::create_tenant))
                        .route("/{id}", get(tenant::get_tenant))
                        .route("/{id}", put(tenant::update_tenant))
                        .route("/{id}", delete(tenant::delete_tenant))
                        .route("/{id}/applications", get(tenant::get_tenant_applications))
                        .route(
                            "/{id}/applications",
                            post(tenant::add_application_to_tenant),
                        )
                        .route(
                            "/{id}/applications",
                            delete(tenant::remove_application_from_tenant),
                        ),
                )
                // 角色管理路由
                .nest(
                    "/roles",
                    Router::new()
                        .route("/", get(auth::list_roles))
                        .route("/", post(auth::create_role))
                        .route("/{id}", get(auth::get_role))
                        .route("/{id}", put(auth::update_role))
                        .route("/{id}", delete(auth::delete_role))
                        .route("/tenant/{tenant_id}", get(auth::get_tenant_roles))
                        .route("/{id}/resources", get(auth::get_role_resources))
                        .route("/{id}/resources", post(auth::assign_resource_to_role))
                        .route("/{id}/resources", delete(auth::remove_resource_from_role)),
                )
                // 资源管理路由
                .nest(
                    "/resources",
                    Router::new()
                        .route("/", get(auth::list_resources))
                        .route("/", post(auth::create_resource))
                        .route("/{id}", get(auth::get_resource))
                        .route("/{id}", put(auth::update_resource))
                        .route("/{id}", delete(auth::delete_resource))
                        .route(
                            "/application/{app_id}",
                            get(auth::get_application_resources),
                        )
                        .route("/menus", get(auth::get_current_user_menus))
                        .route(
                            "/menu-resources",
                            get(auth::get_current_user_menu_resources),
                        ),
                )
                // 应用管理路由
                .nest(
                    "/applications",
                    Router::new()
                        .route("/", get(auth::list_applications))
                        .route("/", post(auth::create_application))
                        .route("/{id}", get(auth::get_application))
                        .route("/{id}", put(auth::update_application))
                        .route("/{id}", delete(auth::delete_application)),
                )
                // 权限检查
                .route("/check-permission", post(auth::check_permission))
                // 套餐管理路由
                .nest(
                    "/plans",
                    Router::new()
                        .route("/", get(plan::list_plans))
                        .route("/", post(plan::create_plan))
                        .route("/{id}", get(plan::get_plan))
                        .route("/{id}", put(plan::update_plan))
                        .route("/{id}", delete(plan::delete_plan))
                        .route("/{id}/entitlements", get(plan::get_plan_entitlements))
                        .route("/{id}/entitlements", post(plan::create_plan_entitlement))
                        .route("/entitlements/{id}", put(plan::update_plan_entitlement))
                        .route("/entitlements/{id}", delete(plan::delete_plan_entitlement))
                        .route("/subscriptions/{tenant_id}", get(plan::get_subscriptions))
                        .route(
                            "/subscriptions/{tenant_id}/active",
                            get(plan::get_active_subscription),
                        )
                        .route(
                            "/subscriptions/{tenant_id}",
                            post(plan::create_subscription),
                        )
                        .route("/subscriptions/{tenant_id}", put(plan::update_subscription))
                        .route(
                            "/subscriptions/{tenant_id}",
                            delete(plan::cancel_subscription),
                        )
                        .route("/usage/{tenant_id}", get(plan::get_tenant_usage))
                        .route("/usage", post(plan::record_usage))
                        .route("/usage-logs/{tenant_id}", get(plan::get_usage_logs)),
                ),
        )
        .with_state(app_state)
}
