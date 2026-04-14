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
use crate::state::AppState;
use axum::{
    routing::{delete, get, post, put},
    Router,
};
use std::sync::Arc;

/// 创建路由
pub fn create_router(app_state: Arc<AppState>) -> Router {
    Router::new()
        .nest(
            "/api/v1",
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
                ),
        )
        .with_state(app_state)
}
