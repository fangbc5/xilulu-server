mod cache;
mod client;
mod config;
mod error;
mod grpc;
mod kafka;
mod modules;
mod router;
mod state;

use std::sync::Arc;

use fbc_starter::{AppResult, Server};
use grpc::HealthService;
use sqlxplus::DbPool;
use state::ImState;

use crate::modules::contact::service::ContactService;
use crate::modules::friend::service::FriendService;
use crate::modules::message::service::MessageService;
use crate::modules::room::service::RoomService;
use crate::modules::sync::service::SyncService;

#[tokio::main]
async fn main() -> AppResult<()> {
    Server::run(|builder| {
        let fbc_app_state = builder.app_state().clone();

        // 数据库初始化（启动阶段失败不可恢复）
        let mysql_pool = fbc_app_state
            .mysql
            .as_ref()
            .expect("MySQL 连接池未初始化")
            .clone();
        let db_pool = Arc::new(
            DbPool::from_mysql_pool(mysql_pool).expect("创建 DbPool 失败"),
        );

        // 初始化服务
        let friend_service = Arc::new(FriendService::new(db_pool.clone(), fbc_app_state.clone()));
        let room_service = Arc::new(RoomService::new(db_pool.clone(), fbc_app_state.clone()));
        let contact_service = Arc::new(ContactService::new(db_pool.clone(), fbc_app_state.clone()));
        let message_service = Arc::new(MessageService::new(db_pool.clone(), fbc_app_state.clone()));
        let sync_service = Arc::new(SyncService::new(db_pool.clone()));

        let im_state = Arc::new(ImState {
            fbc: fbc_app_state,
            db_pool,
            friend_service,
            room_service,
            contact_service,
            message_service,
            sync_service,
        });

        // HTTP 路由
        let http_router = router::create_routes(im_state.clone());

        // gRPC 服务
        let grpc_router =
            tonic::service::Routes::new(HealthService::new().into_server())
            .add_service(crate::grpc::ImServiceImpl::new(im_state.clone()).into_server());

        builder.http_router(http_router).grpc_router(grpc_router)
    })
    .await
}
