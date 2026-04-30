use serde::{Deserialize, Serialize};

// ==================== 请求/响应类型 ====================

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct LoginRequest {
    /// 用户名（可选，与手机号/邮箱二选一）
    pub username: Option<String>,
    /// 密码（可选，与验证码二选一）
    pub password: Option<String>,
    /// 手机号（可选，与用户名/邮箱二选一）
    pub mobile: Option<String>,
    /// 邮箱（可选，与用户名/手机号二选一）
    pub email: Option<String>,
    /// 验证码（可选，与密码二选一）
    pub code: Option<String>,
    /// 验证码ID（可选，与验证码二选一）
    pub captcha_id: Option<String>,
    /// 验证码（可选，与验证码ID二选一）
    pub captcha: Option<String>,
    /// 国家区域码（可选）
    pub region: Option<String>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct LoginResponse {
    /// 访问令牌（单租户直接返回；多租户为临时令牌）
    pub access_token: String,
    /// 刷新令牌（用于续期 access_token）
    pub refresh_token: String,
    /// access_token 有效期（秒）
    pub expires_in: i64,
    /// refresh_token 有效期（秒）
    pub refresh_expires_in: i64,
    /// 用户信息
    pub user_info: UserInfo,
    /// 租户列表（可选）多租户返回租户列表，不返回token，选择租户后返回token
    pub tenant_list: Option<Vec<TenantInfo>>,
}

/// 登录或注册请求
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct LoginOrRegisterRequest {
    /// 手机号（可选）
    pub mobile: Option<String>,
    /// 邮箱（可选）
    pub email: Option<String>,
    /// 验证码（必填，用于无感登录）
    pub code: String,
    /// 国家区域码（可选）
    pub region: Option<String>,
}

/// 登录或注册响应
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct LoginOrRegisterResponse {
    /// 是否为全新注册的用户
    pub is_new_user: bool,
    /// 内嵌原本的登录响应，包含 Token
    pub login_info: LoginResponse,
}

/// 刷新 Token 请求
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct RefreshTokenRequest {
    /// 刷新令牌
    pub refresh_token: String,
}

/// 刷新 Token 响应
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct RefreshTokenResponse {
    /// 新的访问令牌
    pub access_token: String,
    /// 新的刷新令牌（保持不变）
    pub refresh_token: String,
    /// access_token 有效期（秒）
    pub expires_in: i64,
    // 注意：refresh_token 未变化，客户端应使用登录时保存的 refresh_expires_at
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct RegisterRequest {
    /// 用户名（可选，与手机号/邮箱二选一）
    pub username: Option<String>,
    /// 密码（可选，与验证码二选一）
    pub password: Option<String>,
    /// 手机号（可选，与用户名/邮箱二选一）
    pub mobile: Option<String>,
    /// 邮箱（可选，与用户名/手机号二选一）
    pub email: Option<String>,
    /// 验证码（可选，与密码二选一）
    pub code: Option<String>,
    /// 验证码ID（可选，与验证码二选一）
    pub captcha_id: Option<String>,
    /// 验证码（可选，与验证码ID二选一）
    pub captcha: Option<String>,
    /// 昵称（可选，如果不提供则自动生成）
    pub nick_name: Option<String>,
    /// 头像（可选）
    pub avatar: Option<String>,
    /// 国家区域码（可选）
    pub region: Option<String>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct UserInfo {
    pub id: String,
    pub nickname: String,
    pub avatar: Option<String>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct TenantInfo {
    pub id: String,
    pub name: String,
    /// 是否拥有者
    pub is_owner: Option<bool>,
}

/// 选择租户请求
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct SelectTenantRequest {
    /// 租户ID
    pub tenant_id: i64,
    /// 临时token（用于多租户选择）
    pub temp_token: String,
}

/// 选择租户响应
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct SelectTenantResponse {
    /// 正式的访问令牌
    pub access_token: String,
    /// 刷新令牌
    pub refresh_token: String,
    /// access_token 有效期（秒）
    pub expires_in: i64,
    /// refresh_token 有效期（秒）
    pub refresh_expires_in: i64,
    /// 用户信息
    pub user_info: UserInfo,
}

/// 发送验证码请求
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct SendVerifyCodeRequest {
    /// 账号（手机号或邮箱）
    pub account: String,
}

/// 发送验证码响应
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct SendVerifyCodeResponse {
    /// 是否发送成功
    pub success: bool,
    /// 消息
    pub message: String,
}

/// 图片验证码响应
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ImageCaptchaResponse {
    /// 验证码标识，用于后续校验
    pub captcha_id: String,
    /// 图片 Base64（PNG），前端可拼接为 data:image/png;base64,{image_base64}
    pub image_base64: String,
}
