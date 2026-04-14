// 公共校验模块
// 用于登录和注册的验证码校验

use crate::{
    error::AuthError,
    model::{LoginRequest, RegisterRequest},
    service::{ImageCaptchaService, VerifyCodeService},
    AppState,
};

/// 校验请求中的验证码
///
/// # 参数
/// - `state`: 应用状态
/// - `req`: 登录或注册请求（包含验证码信息）
///
/// # 返回
/// - `Ok(())`: 校验通过
/// - `Err(AuthError)`: 校验失败
pub async fn validate_captcha(
    state: &AppState,
    username: Option<&str>,
    password: Option<&str>,
    mobile: Option<&str>,
    email: Option<&str>,
    code: Option<&str>,
    captcha_id: Option<&str>,
    captcha: Option<&str>,
) -> Result<(), AuthError> {
    // 如果是用户名 + 密码登录/注册，则先校验图片验证码
    let is_username_password = username.is_some() && password.is_some();
    if is_username_password {
        let captcha_id =
            captcha_id.ok_or_else(|| AuthError::BadRequest("验证码ID不能为空".to_string()))?;
        let captcha = captcha.ok_or_else(|| AuthError::BadRequest("验证码不能为空".to_string()))?;

        let passed = ImageCaptchaService::verify(state, captcha_id, captcha)
            .await
            .map_err(|e| AuthError::InternalError(format!("验证码校验失败: {}", e)))?;

        if !passed {
            return Err(AuthError::BadRequest(
                "验证码错误或已过期，请重新获取".to_string(),
            ));
        }
    }

    // 如果是手机号或邮箱登录/注册，则先本地校验短信/邮箱验证码
    let is_mobile_or_email = mobile.is_some() || email.is_some();
    if is_mobile_or_email {
        let account = mobile
            .or(email)
            .ok_or_else(|| AuthError::BadRequest("账号不能为空".to_string()))?;

        let code = code.ok_or_else(|| AuthError::BadRequest("验证码不能为空".to_string()))?;

        let passed = VerifyCodeService::verify(state, account, code)
            .await
            .map_err(|e| AuthError::InternalError(format!("验证码校验失败: {}", e)))?;

        if !passed {
            return Err(AuthError::BadRequest(
                "验证码错误或已过期，请重新获取".to_string(),
            ));
        }
    }

    Ok(())
}

/// 从登录请求中提取并校验验证码
pub async fn validate_login_request(state: &AppState, req: &LoginRequest) -> Result<(), AuthError> {
    validate_captcha(
        state,
        req.username.as_deref(),
        req.password.as_deref(),
        req.mobile.as_deref(),
        req.email.as_deref(),
        req.code.as_deref(),
        req.captcha_id.as_deref(),
        req.captcha.as_deref(),
    )
    .await
}

/// 从注册请求中提取并校验验证码
pub async fn validate_register_request(
    state: &AppState,
    req: &RegisterRequest,
) -> Result<(), AuthError> {
    validate_captcha(
        state,
        req.username.as_deref(),
        req.password.as_deref(),
        req.mobile.as_deref(),
        req.email.as_deref(),
        req.code.as_deref(),
        req.captcha_id.as_deref(),
        req.captcha.as_deref(),
    )
    .await
}
