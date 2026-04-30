use axum::routing::{get, post};
use axum::Router;
use sa_token_plugin_axum::SaTokenLayer;

use crate::handler::*;
use crate::model::{
    ImageCaptchaResponse, LoginOrRegisterRequest, LoginOrRegisterResponse, LoginRequest,
    LoginResponse, RefreshTokenRequest, RefreshTokenResponse, RegisterRequest, SelectTenantRequest,
    SelectTenantResponse, SendVerifyCodeRequest, SendVerifyCodeResponse, TenantInfo, UserInfo,
};
use crate::state::AppState;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

/// OpenAPI 文档定义
#[derive(OpenApi)]
#[openapi(
    info(
        title = "ms-auth 认证鉴权服务",
        version = "0.1.0",
        description = "Xilulu 认证鉴权微服务 API。\n\n支持账号密码登录、手机号/邮箱验证码登录、用户注册、Token 刷新、登出等功能。",
        contact(name = "fangbc5@gmail.com"),
    ),
    tags(
        (name = "认证", description = "登录 / 注册 / 登出 / Token 刷新"),
        (name = "验证码", description = "短信验证码 / 图片验证码"),
    ),
    paths(
        crate::handler::login,
        crate::handler::register,
        crate::handler::login_or_register,
        crate::handler::select_tenant,
        crate::handler::user_profile,
        crate::handler::logout,
        crate::handler::refresh_token,
        crate::handler::send_verify_code,
        crate::handler::image_captcha,
    ),
    components(schemas(
        LoginRequest,
        LoginResponse,
        LoginOrRegisterRequest,
        LoginOrRegisterResponse,
        RefreshTokenRequest,
        RefreshTokenResponse,
        RegisterRequest,
        UserInfo,
        TenantInfo,
        SelectTenantRequest,
        SelectTenantResponse,
        SendVerifyCodeRequest,
        SendVerifyCodeResponse,
        ImageCaptchaResponse,
    ))
)]
pub struct ApiDoc;

pub fn create_router(app_state: AppState) -> Router {
    Router::new()
        // ---- Swagger UI（内嵌资源，无需 CDN）----
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
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
