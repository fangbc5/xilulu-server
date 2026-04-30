// HTTP 路由定义

use crate::modules::{auth, device, plan, tenant, user};
use crate::state::AppState;
use axum::{
    routing::{delete, get, post, put},
    Router,
};
use std::sync::Arc;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

// DTO 导入（用于 OpenAPI schemas 注册）
use crate::modules::user::model::dto::{
    AddUserToTenantRequest, AssignRoleToUserRequest, BatchAssignRolesToUserRequest,
    ChangePasswordRequest, CreateUserRequest, CreateUserResponse, GetActiveUserCountRequest,
    ListUsersRequest, RemoveRoleFromUserRequest, ResetPasswordRequest, SetDefaultTenantRequest,
    UpdateUserRequest, UserInfo, UserRoleInfo, UserTenantInfo,
};
use crate::modules::tenant::model::dto::{
    AddApplicationToTenantRequest, CreateTenantRequest, CreateTenantResponse,
    ListTenantsRequest, TenantInfo, UpdateTenantRequest,
};
use crate::modules::auth::model::dto::{
    ApplicationInfo, AssignResourceToRoleRequest, CheckPermissionRequest, CheckPermissionResponse,
    CreateApplicationRequest, CreateApplicationResponse, CreateResourceRequest,
    CreateResourceResponse, CreateRoleRequest, CreateRoleResponse, GetMenuResourcesRequest,
    GetUserMenusRequest, ListApplicationsRequest, ListResourcesRequest, ListRolesRequest,
    MenuResourcesByType, ResourceInfo, RoleInfo, UpdateApplicationRequest, UpdateResourceRequest,
    UpdateRoleRequest,
};
use crate::modules::device::model::dto::{DeviceInfo, RegisterDeviceRequest, UnregisterDeviceRequest};
use crate::modules::plan::model::dto::{
    CreatePlanEntitlementRequest, CreatePlanRequest, CreatePlanResponse,
    CreateTenantSubscriptionRequest, CreateTenantSubscriptionResponse, ListPlansRequest,
    PlanEntitlementInfo, PlanInfo, RecordUsageRequest, TenantSubscriptionInfo, TenantUsageInfo,
    TenantUsageLogInfo, UpdatePlanEntitlementRequest, UpdatePlanRequest,
    UpdateTenantSubscriptionRequest,
};
use crate::modules::plan::handler::GetUsageLogsQuery;

/// OpenAPI 文档定义
#[derive(OpenApi)]
#[openapi(
    info(
        title = "ms-identity 身份管理服务",
        version = "0.1.0",
        description = "Xilulu 身份管理微服务 API。\n\n支持用户管理、租户管理、角色/资源/应用权限管理、设备管理、套餐与订阅管理。",
        contact(name = "fangbc5@gmail.com"),
    ),
    tags(
        (name = "用户管理", description = "用户 CRUD / 密码 / 租户关联 / 角色分配"),
        (name = "租户管理", description = "租户 CRUD / 应用关联"),
        (name = "角色管理", description = "角色 CRUD / 租户角色"),
        (name = "资源管理", description = "资源 CRUD / 菜单 / 菜单子资源"),
        (name = "应用管理", description = "应用 CRUD"),
        (name = "权限管理", description = "角色资源分配 / 权限检查"),
        (name = "设备管理", description = "设备注册/注销/查询"),
        (name = "套餐管理", description = "套餐 CRUD"),
        (name = "套餐权益管理", description = "套餐权益 CRUD"),
        (name = "租户订阅管理", description = "订阅 CRUD / 用量 / 退订"),
    ),
    paths(
        // 用户管理
        crate::modules::user::handler::get_user,
        crate::modules::user::handler::list_users,
        crate::modules::user::handler::create_user,
        crate::modules::user::handler::update_user,
        crate::modules::user::handler::delete_user,
        crate::modules::user::handler::change_password,
        crate::modules::user::handler::reset_password,
        crate::modules::user::handler::add_user_to_tenant,
        crate::modules::user::handler::remove_user_from_tenant,
        crate::modules::user::handler::set_default_tenant,
        crate::modules::user::handler::get_user_tenants,
        crate::modules::user::handler::get_user_roles,
        crate::modules::user::handler::assign_role_to_user,
        crate::modules::user::handler::batch_assign_roles_to_user,
        crate::modules::user::handler::remove_role_from_user,
        crate::modules::user::handler::get_user_count,
        crate::modules::user::handler::get_active_user_count,
        // 设备管理
        crate::modules::device::handler::register_device,
        crate::modules::device::handler::unregister_device,
        crate::modules::device::handler::get_my_devices,
        // 租户管理
        crate::modules::tenant::handler::list_tenants,
        crate::modules::tenant::handler::get_tenant,
        crate::modules::tenant::handler::create_tenant,
        crate::modules::tenant::handler::update_tenant,
        crate::modules::tenant::handler::delete_tenant,
        crate::modules::tenant::handler::add_application_to_tenant,
        crate::modules::tenant::handler::remove_application_from_tenant,
        crate::modules::tenant::handler::get_tenant_applications,
        crate::modules::tenant::handler::get_tenant_count,
        // 角色管理
        crate::modules::auth::handler::get_role,
        crate::modules::auth::handler::create_role,
        crate::modules::auth::handler::update_role,
        crate::modules::auth::handler::delete_role,
        crate::modules::auth::handler::get_tenant_roles,
        crate::modules::auth::handler::list_roles,
        // 资源管理
        crate::modules::auth::handler::get_resource,
        crate::modules::auth::handler::create_resource,
        crate::modules::auth::handler::update_resource,
        crate::modules::auth::handler::delete_resource,
        crate::modules::auth::handler::get_application_resources,
        crate::modules::auth::handler::list_resources,
        crate::modules::auth::handler::get_current_user_menus,
        crate::modules::auth::handler::get_current_user_menu_resources,
        // 应用管理
        crate::modules::auth::handler::get_application,
        crate::modules::auth::handler::create_application,
        crate::modules::auth::handler::update_application,
        crate::modules::auth::handler::delete_application,
        crate::modules::auth::handler::list_applications,
        // 权限管理
        crate::modules::auth::handler::assign_resource_to_role,
        crate::modules::auth::handler::remove_resource_from_role,
        crate::modules::auth::handler::get_role_resources,
        crate::modules::auth::handler::check_permission,
        // 套餐管理
        crate::modules::plan::handler::get_plan,
        crate::modules::plan::handler::create_plan,
        crate::modules::plan::handler::update_plan,
        crate::modules::plan::handler::delete_plan,
        crate::modules::plan::handler::list_plans,
        // 套餐权益管理
        crate::modules::plan::handler::create_plan_entitlement,
        crate::modules::plan::handler::get_plan_entitlements,
        crate::modules::plan::handler::update_plan_entitlement,
        crate::modules::plan::handler::delete_plan_entitlement,
        // 租户订阅管理
        crate::modules::plan::handler::create_subscription,
        crate::modules::plan::handler::get_subscriptions,
        crate::modules::plan::handler::get_active_subscription,
        crate::modules::plan::handler::update_subscription,
        crate::modules::plan::handler::cancel_subscription,
        crate::modules::plan::handler::record_usage,
        crate::modules::plan::handler::get_tenant_usage,
        crate::modules::plan::handler::get_usage_logs,
    ),
    components(schemas(
        // 用户
        CreateUserRequest, CreateUserResponse, UpdateUserRequest, UserInfo,
        ChangePasswordRequest, ResetPasswordRequest, ListUsersRequest,
        GetActiveUserCountRequest, UserTenantInfo, AddUserToTenantRequest,
        SetDefaultTenantRequest, UserRoleInfo, AssignRoleToUserRequest,
        BatchAssignRolesToUserRequest, RemoveRoleFromUserRequest,
        // 租户
        CreateTenantRequest, CreateTenantResponse, UpdateTenantRequest, TenantInfo,
        ListTenantsRequest, AddApplicationToTenantRequest,
        // 角色
        CreateRoleRequest, CreateRoleResponse, UpdateRoleRequest, RoleInfo, ListRolesRequest,
        // 资源
        CreateResourceRequest, CreateResourceResponse, UpdateResourceRequest, ResourceInfo,
        ListResourcesRequest, MenuResourcesByType, GetUserMenusRequest, GetMenuResourcesRequest,
        AssignResourceToRoleRequest,
        // 应用
        CreateApplicationRequest, CreateApplicationResponse, UpdateApplicationRequest,
        ApplicationInfo, ListApplicationsRequest,
        // 权限
        CheckPermissionRequest, CheckPermissionResponse,
        // 设备
        RegisterDeviceRequest, UnregisterDeviceRequest, DeviceInfo,
        // 套餐
        CreatePlanRequest, CreatePlanResponse, UpdatePlanRequest, PlanInfo, ListPlansRequest,
        // 套餐权益
        CreatePlanEntitlementRequest, UpdatePlanEntitlementRequest, PlanEntitlementInfo,
        // 租户订阅
        CreateTenantSubscriptionRequest, CreateTenantSubscriptionResponse,
        UpdateTenantSubscriptionRequest, TenantSubscriptionInfo,
        // 用量
        RecordUsageRequest, TenantUsageInfo, TenantUsageLogInfo, GetUsageLogsQuery,
    ))
)]
pub struct ApiDoc;

/// 创建 HTTP 路由
pub fn create_router(app_state: Arc<AppState>) -> Router {
    Router::new()
        // ---- Swagger UI（内嵌资源，无需 CDN）----
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
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
