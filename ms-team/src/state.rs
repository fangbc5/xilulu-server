// 应用状态定义
// 包含所有 Service 实例和共享资源

use crate::config::OrganizationConfig;
use crate::modules::contacts::service::ContactsService;
use crate::modules::contacts::permission::default::DefaultPermission;
use crate::modules::contacts::search::adapter::MeilisearchAdapter;
use crate::modules::department::DepartmentService;
use crate::modules::employee::{EmployeeDepartmentService, EmployeePositionService, EmployeeService};
use crate::modules::organization::OrganizationService;
use crate::modules::position::PositionService;
use fbc_starter::AppState as FbcAppState;
use sqlxplus::DbPool;
use std::sync::Arc;

/// 应用状态
/// 包含所有 Service 实例和共享资源
#[derive(Clone)]
pub struct AppState {
    /// fbc-starter 的 AppState（包含 Redis 等）
    pub fbc_app_state: Arc<FbcAppState>,
    /// 数据库连接池
    pub db_pool: Arc<DbPool>,
    /// 配置
    pub config: OrganizationConfig,
    /// 组织服务
    pub organization_service: Arc<OrganizationService>,
    /// 部门服务
    pub department_service: Arc<DepartmentService>,
    /// 岗位服务
    pub position_service: Arc<PositionService>,
    /// 员工服务
    pub employee_service: Arc<EmployeeService>,
    /// 员工部门关系服务
    pub employee_department_service: Arc<EmployeeDepartmentService>,
    /// 员工岗位关系服务
    pub employee_position_service: Arc<EmployeePositionService>,
    /// 通讯录服务
    pub contacts_service: Arc<ContactsService>,
}

impl AppState {
    /// 创建新的 AppState
    pub fn new(
        fbc_app_state: Arc<FbcAppState>,
        db_pool: Arc<DbPool>,
        config: OrganizationConfig,
    ) -> Self {
        // 先创建独立的服务
        let organization_service = Arc::new(OrganizationService::new(db_pool.clone()));
        let department_service = Arc::new(DepartmentService::new(db_pool.clone(), fbc_app_state.clone()));
        let position_service = Arc::new(PositionService::new(db_pool.clone()));

        // 创建有依赖的服务，注入 department_service
        let employee_service = Arc::new(
            EmployeeService::new(db_pool.clone())
                .with_department_service(department_service.clone()),
        );
        let employee_department_service = Arc::new(
            EmployeeDepartmentService::new(db_pool.clone())
                .with_department_service(department_service.clone()),
        );
        let employee_position_service = Arc::new(EmployeePositionService::new(db_pool.clone()));

        // 构建搜索引擎适配器
        let search_port: Arc<dyn crate::modules::contacts::search::port::EmployeeSearchPort> =
            match MeilisearchAdapter::new(&config.meilisearch.url, &config.meilisearch.api_key) {
                Ok(adapter) => {
                    tracing::info!("Meilisearch 适配器初始化成功: {}", config.meilisearch.url);
                    Arc::new(adapter)
                }
                Err(e) => {
                    tracing::warn!("Meilisearch 适配器初始化失败，搜索功能将不可用: {}", e);
                    // 仍然创建一个默认实例，搜索时会降级到 MySQL
                    Arc::new(MeilisearchAdapter::new("http://127.0.0.1:7700", "").expect("Meilisearch 兜底初始化失败"))
                }
            };

        // 构建通讯录权限引擎（Phase 1: 全开放策略）
        let permission: Arc<dyn crate::modules::contacts::permission::port::ContactsPermission> =
            Arc::new(DefaultPermission);

        // 构建通讯录聚合服务
        let contacts_service = Arc::new(ContactsService::new(
            db_pool.clone(),
            organization_service.clone(),
            department_service.clone(),
            employee_service.clone(),
            employee_department_service.clone(),
            employee_position_service.clone(),
            search_port,
            permission,
            config.contacts.clone(),
        ));

        Self {
            organization_service,
            department_service,
            position_service,
            employee_service,
            employee_department_service,
            employee_position_service,
            contacts_service,
            fbc_app_state,
            db_pool,
            config,
        }
    }
}
