use fbc_starter::{AppResult, Server};
use sqlxplus::DbPool;
use std::sync::Arc;

mod config;
mod context;
mod error;
mod grpc;

mod modules;
mod router;
mod state;

#[tokio::main]
async fn main() -> AppResult<()> {
    Server::run(|builder| {
        // 获取 MySQL 连接池
        let mysql_pool = match builder.app_state().mysql.as_ref() {
            Some(pool) => pool.clone(),
            None => {
                panic!("MySQL 连接池未初始化");
            }
        };

        // 创建 sqlxplus DbPool
        let db_pool = Arc::new(DbPool::from_mysql_pool(mysql_pool).expect("创建 DbPool 失败"));

        let config = config::IdentityConfig::new(builder.config().clone()).expect("加载配置失败");

        // 创建应用状态（包含所有 Service）
        let app_state = Arc::new(state::AppState::new(db_pool, config));

        // 创建 HTTP 路由
        let http_router = router::create_router(app_state.clone());

        // 创建 gRPC 服务并注册
        let user_service = app_state.user_service.clone();
        let user_tenant_service = app_state.user_tenant_service.clone();
        let tenant_service = app_state.tenant_service.clone();

        let grpc_router = tonic::service::Routes::new(grpc::IdentityServiceImpl::server(
            user_service,
            user_tenant_service,
            tenant_service,
        ))
        .add_service(grpc::DeviceServiceImpl::server(app_state.device_service.clone()));

        builder.http_router(http_router).grpc_router(grpc_router)
    })
    .await
}
