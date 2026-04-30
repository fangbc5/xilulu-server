// 验证码相关处理器

use crate::{
    error::AuthError,
    kafka::NotificationSender,
    model::{ImageCaptchaResponse, SendVerifyCodeRequest, SendVerifyCodeResponse},
    service::{ImageCaptchaService, VerifyCodeService, VerifyCodeType},
    AppState,
};
use axum::{extract::ConnectInfo, extract::State, http::HeaderMap, Json};
use fbc_starter::R;
use regex::Regex;
use sa_token_plugin_axum::sa_ignore;
use std::net::SocketAddr;

/// 手机号正则表达式（中国大陆）
const MOBILE_REGEX: &str = r"^1[3-9]\d{9}$";

/// 邮箱正则表达式
const EMAIL_REGEX: &str = r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$";

/// 发送验证码接口（短信/邮箱）
#[sa_ignore]
#[utoipa::path(
    post,
    path = "/api/v1/auth/send-code",
    tag = "验证码",
    request_body = SendVerifyCodeRequest,
    responses(
        (status = 200, description = "发送成功", body = R<SendVerifyCodeResponse>)
    )
)]
pub async fn send_verify_code(
    State(state): State<AppState>,
    Json(req): Json<SendVerifyCodeRequest>,
) -> Result<Json<R<SendVerifyCodeResponse>>, AuthError> {
    let account = req.account.trim();

    if account.is_empty() {
        return Err(AuthError::BadRequest("账号不能为空".to_string()));
    }

    // 使用正则表达式判断是手机号还是邮箱
    let mobile_regex = Regex::new(MOBILE_REGEX)
        .map_err(|e| AuthError::InternalError(format!("正则表达式错误: {}", e)))?;
    let email_regex = Regex::new(EMAIL_REGEX)
        .map_err(|e| AuthError::InternalError(format!("正则表达式错误: {}", e)))?;

    let (code_type, channel) = if mobile_regex.is_match(account) {
        (VerifyCodeType::Mobile, "sms")
    } else if email_regex.is_match(account) {
        (VerifyCodeType::Email, "email")
    } else {
        return Err(AuthError::BadRequest(
            "账号格式错误，请输入有效的手机号或邮箱".to_string(),
        ));
    };

    // 生成验证码
    let code = VerifyCodeService::generate_and_store(&state, account, code_type)
        .await
        .map_err(|e| AuthError::InternalError(format!("生成验证码失败: {}", e)))?;

    // 发送验证码通知到 Kafka
    NotificationSender::send_verify_code(&state, account, &code, channel)
        .await
        .map_err(|e| AuthError::InternalError(format!("发送验证码失败: {}", e)))?;

    tracing::info!("✅ 验证码已发送到: {}", account);

    Ok(Json(R::ok_with_data(SendVerifyCodeResponse {
        success: true,
        message: "验证码已发送".to_string(),
    })))
}

/// 从请求中提取客户端 IP 地址
/// 优先级：X-Forwarded-For > X-Real-IP > ConnectInfo (SocketAddr) > "unknown"
fn extract_client_ip(headers: &HeaderMap, connect_info: Option<&SocketAddr>) -> String {
    // 1. 优先从 X-Forwarded-For 获取（可能包含多个 IP，取第一个）
    if let Some(forwarded_for) = headers.get("x-forwarded-for") {
        if let Ok(forwarded_str) = forwarded_for.to_str() {
            if let Some(first_ip) = forwarded_str.split(',').next() {
                let ip = first_ip.trim();
                if !ip.is_empty() {
                    return ip.to_string();
                }
            }
        }
    }

    // 2. 其次从 X-Real-IP 获取
    if let Some(real_ip) = headers.get("x-real-ip") {
        if let Ok(ip_str) = real_ip.to_str() {
            let ip = ip_str.trim();
            if !ip.is_empty() {
                return ip.to_string();
            }
        }
    }

    // 3. 从 ConnectInfo 获取（直接连接的客户端 IP）
    if let Some(addr) = connect_info {
        return addr.ip().to_string();
    }

    // 4. 如果都没有，返回 "unknown"（频率限制会基于此，但效果有限）
    "unknown".to_string()
}

/// 获取图片验证码接口（无需认证）
#[sa_ignore]
#[utoipa::path(
    get,
    path = "/api/v1/auth/captcha",
    tag = "验证码",
    responses(
        (status = 200, description = "图片验证码", body = R<ImageCaptchaResponse>)
    )
)]
pub async fn image_captcha(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
) -> Result<Json<R<ImageCaptchaResponse>>, AuthError> {
    // 提取客户端 IP（优先从请求头，其次从连接信息）
    let client_ip = extract_client_ip(&headers, Some(&addr));

    let (captcha_id, image_base64) = ImageCaptchaService::generate(&state, &client_ip)
        .await
        .map_err(|e| {
            let error_msg = e.to_string();
            if error_msg.contains("请求过于频繁") {
                AuthError::TooManyRequests(error_msg)
            } else {
                AuthError::InternalError(format!("生成图片验证码失败: {}", e))
            }
        })?;

    tracing::info!("✅ 生成图片验证码: {} (IP: {})", captcha_id, client_ip);

    Ok(Json(R::ok_with_data(ImageCaptchaResponse {
        captcha_id,
        image_base64,
    })))
}
