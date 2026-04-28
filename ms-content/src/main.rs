mod config;
mod error;
mod job;
mod modules;
mod router;
mod state;

use std::sync::Arc;

use axum::middleware;
use fbc_starter::job::{init_job_client, JobConfig};
use fbc_starter::{user_context_middleware, AppResult, Server};
use sqlxplus::DbPool;
use tracing::info;

use crate::config::ContentConfig;
use crate::modules::content::search::adapter::MeilisearchAdapter;
use crate::state::ContentState;

#[tokio::main]
async fn main() -> AppResult<()> {
    Server::run(|builder| {
        // 1. 获取 MySQL 连接池
        let fbc_app_state = builder.app_state().clone();
        let mysql_pool = fbc_app_state
            .mysql
            .as_ref()
            .expect("MySQL 连接池未初始化")
            .clone();
        let db_pool = Arc::new(
            DbPool::from_mysql_pool(mysql_pool).expect("创建 DbPool 失败"),
        );

        // 2. 加载服务配置
        let config = ContentConfig::from_env();

        // 3. 初始化 Meilisearch 搜索适配器
        let meili_url = config.meilisearch_url.clone();
        let meili_key = config.meilisearch_api_key.clone();
        let search_port: Arc<dyn modules::content::search::port::SearchPort> = Arc::new(
            MeilisearchAdapter::new(&meili_url, &meili_key)
                .expect("Meilisearch 初始化失败"),
        );

        // 4. 组装状态
        let content_state = Arc::new(ContentState::new(
            db_pool.clone(),
            search_port,
        ));

        // 5. 初始化 Job 调度（可选，非阻塞）
        let job_config = JobConfig {
            admin_address: config.xxl_admin_addr.clone(),
            access_token: config.xxl_access_token.clone(),
            app_name: "ms-content-executor".to_string(),
            executor_port: Some(config.xxl_executor_port),
            log_path: Some("logs/ms-content-job".to_string()),
        };

        if let Ok(client) = init_job_client(job_config) {
            info!(action = "job_init", "正在自动注册任务映射表...");
            if let Err(e) = client.register_async(
                Arc::new("demojob".to_string()),
                Arc::new(job::demo_job::DemoJobTask),
            ) {
                tracing::error!(error = ?e, "演示任务注册失败");
            }
        } else {
            tracing::warn!("未能启动 Job 调度客户端，跳过任务注册");
        }

        // 6. 注册 HTTP 路由（含 CORS + 用户上下文中间件）
        let http_router = router::create_router(content_state)
            .layer(middleware::from_fn(user_context_middleware))
            .layer(fbc_starter::http::create_cors_layer(builder.config()));

        builder.http_router(http_router)
    })
    .await
}
