use axum::routing::{get, post};
use axum::Router;
use sa_token_plugin_axum::SaTokenLayer;

use crate::handler::*;
use crate::state::AppState;

pub fn create_router(app_state: AppState) -> Router {
    Router::new()
        .nest(
            "/api/v1/auth",
            Router::new()
                // 公开接口（不需要认证）
                .route("/login", post(login))
                .route("/register", post(register))
                .route("/login-or-register", post(login_or_register))
                .route("/send-code", post(send_verify_code))
                .route("/captcha", get(image_captcha))
                .route("/refresh-token", post(refresh_token))
                // 需要登录的接口
                .route("/logout", post(logout))
                .route("/select-tenant", post(select_tenant))
                .route("/profile", get(user_profile)),
        )
        // 添加 SaTokenLayer 中间件来提取和验证 Token
        .layer(SaTokenLayer::new(app_state.sa_token.clone()))
        .with_state(app_state.clone())
}
