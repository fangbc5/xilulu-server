
use fbc_starter::{AppResult, Server};
use sqlxplus::DbPool;
use std::sync::Arc;

mod client;
mod config;
mod error;
mod grpc;
mod middleware;
mod modules;
mod router;
mod state;

#[tokio::main]
async fn main() -> AppResult<()> {
    Server::run(|builder| {
        // 从启动器获取全局 AppState（包含数据库连接池等）
        let fbc_app_state = builder.app_state().clone();

        // 获取 MySQL 连接池
        let mysql_pool = match fbc_app_state.mysql.as_ref() {
            Some(pool) => pool.clone(),
            None => {
                panic!("MySQL 连接池未初始化");
            }
        };

        // 创建 sqlxplus DbPool
        let db_pool = Arc::new(DbPool::from_mysql_pool(mysql_pool).expect("创建 DbPool 失败"));

        let config = config::OrganizationConfig::new(builder.config().clone())
            .expect("加载配置失败");

        // 创建应用状态（包含所有 Service）
        let app_state = Arc::new(state::AppState::new(fbc_app_state, db_pool, config));

        // 创建 HTTP 路由
        let http_router = router::create_router(app_state.clone());

        // 创建 gRPC 服务并注册
        let grpc_router = tonic::service::Routes::new(
            grpc::OrganizationServiceImpl::server(app_state.clone()),
        );

        builder.http_router(http_router).grpc_router(grpc_router)
    })
    .await
}
