use fbc_starter::{AppResult, Server};
use ::sa_token_core::config::SaTokenConfig;
use ::sa_token_core::refresh::RefreshTokenManager;
use sa_token_plugin_axum::*;
use std::sync::Arc;

use crate::config::AuthConfig;
use crate::state::AppState;

mod client;
mod config;
mod error;
mod handler;

mod kafka;
mod model;
mod router;
mod service;
mod state;

#[tokio::main]
async fn main() -> AppResult<()> {
    Server::run(|builder| {
        // 从启动器获取全局 AppState（包含 Redis 等）
        let fbc_app_state = builder.app_state().clone();
        let fbc_config = builder.config().clone();

        // 加载 Auth 服务配置
        let auth_config = AuthConfig::new(fbc_config.clone());

        // 检查 Redis 连接（必需）
        if fbc_app_state.redis.is_none() {
            panic!("Redis 连接未初始化，SSO 服务需要 Redis");
        }

        // 创建 Redis 存储（需要构建带密码的 URL）
        let redis_config = fbc_config.redis.as_ref().unwrap();
        let redis_url = if let Some(ref pwd) = redis_config.password {
            let url = &redis_config.url;
            if !url.contains('@') {
                if let Some(prefix_end) = url.find("://") {
                    let prefix = &url[..prefix_end + 3];
                    let rest = &url[prefix_end + 3..];
                    format!("{}:{}@{}", prefix, pwd, rest)
                } else {
                    url.clone()
                }
            } else {
                url.clone()
            }
        } else {
            redis_config.url.clone()
        };
        let storage = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                RedisStorage::new(&redis_url, "sa_token:")
                    .await
                    .expect("Failed to create Redis storage")
            })
        });

        let storage_arc: Arc<dyn SaStorage> = Arc::new(storage);

        // 使用构建器模式创建 sa-token 状态
        // token_style 设置为 Jwt，sa-token 会通过 JwtManager 生成 JWT token
        let sa_token_config = SaTokenConfig {
            token_name: "Authorization".to_string(),
            timeout: auth_config.auth.access_token_timeout,
            token_style: ::sa_token_core::config::TokenStyle::Jwt,
            jwt_secret_key: Some(auth_config.auth.jwt_secret.clone()),
            enable_refresh_token: true,
            refresh_token_timeout: auth_config.auth.refresh_token_timeout,
            ..Default::default()
        };

        // 创建 SaTokenManager 并初始化 StpUtil
        let sa_token_manager = ::sa_token_core::SaTokenManager::new(
            storage_arc.clone(),
            sa_token_config.clone(),
        );
        StpUtil::init_manager(sa_token_manager.clone());

        let sa_token_state = SaTokenState {
            manager: Arc::new(sa_token_manager),
        };

        // 创建 RefreshTokenManager（用于管理 refresh token）
        let refresh_token_mgr = RefreshTokenManager::new(
            storage_arc.clone(),
            Arc::new(sa_token_config),
        );

        let app_state = AppState {
            fbc_app_state: fbc_app_state.clone(),
            sa_token: sa_token_state,
            refresh_token_mgr,
            auth_config: Arc::new(auth_config),
        };
        let http_router = router::create_router(app_state);
        builder.http_router(http_router)
    })
    .await
}
