use crate::modules::department::{
    create_department, delete_department, get_children, get_department, get_department_tree,
    get_roots, list_departments, update_department,
};
use crate::modules::employee::{
    add_employee_position, add_employee_to_department, create_employee, delete_employee,
    get_employee, list_employees, remove_employee_from_department, remove_employee_position,
    update_employee,
};
use crate::modules::organization::{
    create_organization, delete_organization, get_organization, get_organization_tree,
    list_organizations, update_organization,
};
use crate::modules::position::{
    create_position, delete_position, get_position, list_positions, update_position,
};
use crate::modules::contacts::{
    contacts_department, contacts_department_members, contacts_employee_detail, contacts_entry,
    contacts_search, rebuild_search_index,
};
use crate::state::AppState;
use axum::{
    routing::{delete, get, post, put},
    Router,
};
use std::sync::Arc;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

// DTO 导入（通过各模块的 pub use re-export）
use crate::modules::organization::{
    CreateOrganizationRequest, UpdateOrganizationRequest, OrganizationResponse, OrganizationTreeNode,
};
use crate::modules::department::{
    CreateDepartmentRequest, UpdateDepartmentRequest, DepartmentResponse, DepartmentTreeNode,
};
use crate::modules::employee::{
    CreateEmployeeRequest, UpdateEmployeeRequest, EmployeeResponse,
    DepartmentBrief, PositionBrief,
    AddEmployeeToDepartmentRequest, AddEmployeePositionRequest,
    EmployeeDepartmentResponse, EmployeePositionResponse,
};
use crate::modules::position::{
    CreatePositionRequest, UpdatePositionRequest, PositionResponse,
};
use crate::modules::contacts::model::dto::{
    ContactsEntryResponse, OrganizationBrief, DepartmentSummary, LeaderBrief,
    ContactsDepartmentResponse, DepartmentInfo, MemberPreview,
    ContactsEmployeeDetailResponse, EmployeeDeptInfo, EmployeePosInfo,
    ContactsSearchResponse, ContactsMemberPageResponse,
};

/// OpenAPI 文档定义
///
/// 自动收集所有标注了 `#[utoipa::path]` 的 handler
#[derive(OpenApi)]
#[openapi(
    info(
        title = "ms-team 组织与人员中台",
        version = "0.1.0",
        description = "Xilulu 微服务生态的统一组织管理中台。\n\n支持多组织树、部门管理、岗位管理、员工管理、通讯录浏览与搜索等能力。",
        contact(name = "fangbc5@gmail.com"),
    ),
    tags(
        (name = "组织管理", description = "组织 CRUD（创建/查询/树形/删除）"),
        (name = "部门管理", description = "部门 CRUD（树形/根部门/子部门）"),
        (name = "岗位管理", description = "岗位 CRUD"),
        (name = "员工管理", description = "员工 CRUD"),
        (name = "员工关系", description = "员工-部门/岗位关系管理"),
        (name = "通讯录", description = "通讯录浏览、搜索、详情"),
        (name = "管理", description = "搜索索引重建等管理接口"),
    ),
    paths(
        // 组织管理
        crate::modules::organization::create_organization,
        crate::modules::organization::get_organization,
        crate::modules::organization::get_organization_tree,
        crate::modules::organization::list_organizations,
        crate::modules::organization::update_organization,
        crate::modules::organization::delete_organization,
        // 部门管理
        crate::modules::department::create_department,
        crate::modules::department::get_department,
        crate::modules::department::list_departments,
        crate::modules::department::get_department_tree,
        crate::modules::department::update_department,
        crate::modules::department::delete_department,
        crate::modules::department::get_roots,
        crate::modules::department::get_children,
        // 岗位管理
        crate::modules::position::create_position,
        crate::modules::position::get_position,
        crate::modules::position::list_positions,
        crate::modules::position::update_position,
        crate::modules::position::delete_position,
        // 员工管理
        crate::modules::employee::create_employee,
        crate::modules::employee::get_employee,
        crate::modules::employee::list_employees,
        crate::modules::employee::update_employee,
        crate::modules::employee::delete_employee,
        crate::modules::employee::add_employee_to_department,
        crate::modules::employee::remove_employee_from_department,
        crate::modules::employee::add_employee_position,
        crate::modules::employee::remove_employee_position,
        // 通讯录
        crate::modules::contacts::contacts_entry,
        crate::modules::contacts::contacts_department,
        crate::modules::contacts::contacts_employee_detail,
        crate::modules::contacts::contacts_search,
        crate::modules::contacts::contacts_department_members,
        crate::modules::contacts::rebuild_search_index,
    ),
    components(schemas(
        // 组织
        CreateOrganizationRequest,
        UpdateOrganizationRequest,
        OrganizationResponse,
        OrganizationTreeNode,
        // 部门
        CreateDepartmentRequest,
        UpdateDepartmentRequest,
        DepartmentResponse,
        DepartmentTreeNode,
        // 岗位
        CreatePositionRequest,
        UpdatePositionRequest,
        PositionResponse,
        // 员工
        CreateEmployeeRequest,
        UpdateEmployeeRequest,
        EmployeeResponse,
        DepartmentBrief,
        PositionBrief,
        AddEmployeeToDepartmentRequest,
        AddEmployeePositionRequest,
        EmployeeDepartmentResponse,
        EmployeePositionResponse,
        // 通讯录
        ContactsEntryResponse,
        OrganizationBrief,
        DepartmentSummary,
        LeaderBrief,
        ContactsDepartmentResponse,
        DepartmentInfo,
        MemberPreview,
        ContactsEmployeeDetailResponse,
        EmployeeDeptInfo,
        EmployeePosInfo,
        ContactsSearchResponse,
        ContactsMemberPageResponse,
    ))
)]
pub struct ApiDoc;

/// 创建路由
pub fn create_router(app_state: Arc<AppState>) -> Router {
    Router::new()
        // ---- Swagger UI（内嵌资源，无需 CDN）----
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .nest(
            "/api/v1/team",
            Router::new()
                // 组织管理
                .nest(
                    "/organizations",
                    Router::new()
                        .route("/", get(list_organizations))
                        .route("/", post(create_organization))
                        .route("/tree", get(get_organization_tree))
                        .route("/{id}", get(get_organization))
                        .route("/{id}", put(update_organization))
                        .route("/{id}", delete(delete_organization)),
                )
                // 部门管理
                .nest(
                    "/departments",
                    Router::new()
                        .route("/", get(list_departments))
                        .route("/", post(create_department))
                        .route("/roots", get(get_roots))
                        .route("/tree", get(get_department_tree))
                        .route("/{id}", get(get_department))
                        .route("/{id}", put(update_department))
                        .route("/{id}", delete(delete_department))
                        .route("/{id}/children", get(get_children)),
                )
                // 岗位管理
                .nest(
                    "/positions",
                    Router::new()
                        .route("/", get(list_positions))
                        .route("/", post(create_position))
                        .route("/{id}", get(get_position))
                        .route("/{id}", put(update_position))
                        .route("/{id}", delete(delete_position)),
                )
                // 员工管理
                .nest(
                    "/employees",
                    Router::new()
                        .route("/", get(list_employees))
                        .route("/", post(create_employee))
                        .route("/{id}", get(get_employee))
                        .route("/{id}", put(update_employee))
                        .route("/{id}", delete(delete_employee))
                        // 员工-部门关系
                        .route("/{id}/departments", post(add_employee_to_department))
                        .route(
                            "/{employee_id}/departments/{department_id}",
                            delete(remove_employee_from_department),
                        )
                        // 员工-岗位关系
                        .route("/{id}/positions", post(add_employee_position))
                        .route(
                            "/{employee_id}/positions/{position_id}",
                            delete(remove_employee_position),
                        ),
                )
                // 通讯录接口
                .nest(
                    "/contacts",
                    Router::new()
                        .route("/entry", get(contacts_entry))
                        .route("/departments/{dept_id}", get(contacts_department))
                        .route("/employees/{id}", get(contacts_employee_detail))
                        .route("/search", get(contacts_search))
                        .route("/departments/{dept_id}/members", get(contacts_department_members)),
                )
                // 管理接口
                .nest(
                    "/admin",
                    Router::new().route("/search/rebuild", post(rebuild_search_index)),
                ),
        )
        .with_state(app_state)
}
