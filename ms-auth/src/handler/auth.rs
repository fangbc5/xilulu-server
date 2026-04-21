// Author: 金书记
//
//! 认证相关代码

use crate::{
    client::IdentityClient,
    error::AuthError,
    model::{
        LoginOrRegisterRequest, LoginOrRegisterResponse, LoginRequest, LoginResponse,
        RefreshTokenRequest, RefreshTokenResponse, RegisterRequest, SelectTenantRequest,
        SelectTenantResponse, TenantInfo, UserInfo,
    },
    service::{
        validate_login_request, validate_register_request, NicknameConfig, NicknameGenerator,
        NicknameMode, TempTokenService,
    },
    AppState,
};
use axum::{extract::State, Json};
use fbc_starter::R;
use sa_token_plugin_axum::{sa_check_login, sa_ignore, StpUtil};

// ==================== 登录接口 ====================

#[sa_ignore]
pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<R<LoginResponse>>, AuthError> {
    // 0. 校验验证码（图片验证码或短信/邮箱验证码）
    validate_login_request(&state, &req).await?;

    // 1. 统一调用 identity 服务验证用户（简化判断逻辑）
    let verify_response = IdentityClient::verify(
        req.username.as_deref(),
        req.password.as_deref(),
        req.mobile.as_deref(),
        req.email.as_deref(),
        req.region.as_deref(),
    )
    .await
    .map_err(|e| AuthError::ServiceUnavailable(format!("验证失败: {}", e)))?;

    // 2. 检查验证结果
    if !verify_response.success {
        return Err(AuthError::Unauthorized(verify_response.message.clone()));
    }

    let user = verify_response
        .user
        .ok_or_else(|| AuthError::InternalError("用户信息为空".to_string()))?;

    let user_id = user.id;

    // 3. 获取用户租户列表
    let tenants_response = IdentityClient::get_user_tenants(user_id)
        .await
        .map_err(|e| AuthError::ServiceUnavailable(format!("获取租户列表失败: {}", e)))?;

    if !tenants_response.success {
        return Err(AuthError::InternalError(tenants_response.message.clone()));
    }

    let tenants = tenants_response.tenants;

    // 4. 判断租户数量，决定返回正式 token 还是临时 token
    let user_info = UserInfo {
        id: user_id.to_string(),
        nickname: user.nick_name.clone(),
        avatar: if user.avatar.is_empty() {
            None
        } else {
            Some(user.avatar.clone())
        },
    };

    // 过滤出有效的租户（status = 1）
    let valid_tenants: Vec<_> = tenants.into_iter().filter(|t| t.status == 1).collect();

    if valid_tenants.is_empty() {
        return Err(AuthError::Forbidden("用户没有可用的租户".to_string()));
    }

    // 如果只有一个租户，直接返回正式 token（无需选择）
    if valid_tenants.len() == 1 {
        let tenant = &valid_tenants[0];
        // 构造 extra claims，写入 JWT payload
        let extra = serde_json::json!({
            "tenant_id": tenant.tenant_id.to_string(),
            "username": user.username,
            "token_type": "access"
        });

        // StpUtil::login_with_extra 同时完成：JWT生成 + 会话创建 + extra写入JWT
        let token_value = StpUtil::login_with_extra(user_id, extra.clone()).await?;
        let token = token_value.to_string();

        // 生成 refresh token（sa-token RefreshTokenManager）
        let login_id = user_id.to_string();
        let refresh_token = state.refresh_token_mgr.generate(&login_id);
        state
            .refresh_token_mgr
            .store_with_extra(&refresh_token, &token, &login_id, Some(&extra))
            .await
            .map_err(|e| AuthError::InternalError(format!("存储refresh token失败: {}", e)))?;

        tracing::info!("✅ 用户 {} 登录成功，单租户直接返回JWT token", user_id);

        return Ok(Json(R::ok_with_data(LoginResponse {
            access_token: token,
            refresh_token,
            expires_in: state.auth_config.auth.access_token_timeout,
            refresh_expires_in: state.auth_config.auth.refresh_token_timeout,
            user_info,
            tenant_list: None,
        })));
    }

    // 多个租户，返回临时 token 和租户列表
    let tenant_list: Vec<TenantInfo> = valid_tenants
        .into_iter()
        .map(|t| TenantInfo {
            id: t.tenant_id.to_string(),
            name: t.name.clone(), // 从 identity 服务获取的租户名称
            is_owner: Some(t.is_owner == 1),
        })
        .collect();

    // 生成临时 token
    let temp_token_data: Vec<crate::service::temp_token::TenantInfo> = tenant_list
        .iter()
        .map(|t| crate::service::temp_token::TenantInfo {
            tenant_id: t.id.parse().unwrap_or(0),
            is_owner: t.is_owner.unwrap_or(false),
        })
        .collect();

    let temp_token = TempTokenService::create_temp_token(&state, user_id, temp_token_data)
        .await
        .map_err(|e| AuthError::InternalError(format!("生成临时token失败: {}", e)))?;

    tracing::info!(
        "✅ 用户 {} 登录成功，多租户返回临时token和租户列表",
        user_id
    );

    Ok(Json(R::ok_with_data(LoginResponse {
        access_token: temp_token,
        refresh_token: String::new(), // 多租户场景暂不返回 refresh_token，选择租户后再返回
        expires_in: 0, // 临时token不返回过期时间
        refresh_expires_in: 0,
        user_info,
        tenant_list: Some(tenant_list),
    })))
}

// ==================== 注册接口 ====================

#[sa_ignore]
pub async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<R<String>>, AuthError> {
    // 0. 校验验证码（图片验证码或短信/邮箱验证码）
    validate_register_request(&state, &req).await?;

    // 1. 生成昵称（如果用户未提供则自动生成）
    let generated_nickname_opt;
    let nickname = if let Some(nick_name) = &req.nick_name {
        Some(nick_name.as_str())
    } else {
        // 自动生成昵称
        let config = NicknameConfig {
            mode: NicknameMode::Auto,
            with_number: false,
            max_length: 16,
        };
        generated_nickname_opt = Some(NicknameGenerator::generate(config));
        tracing::info!("自动生成昵称: {}", generated_nickname_opt.as_ref().unwrap());
        generated_nickname_opt.as_deref()
    };

    // 2. 调用 identity 服务注册用户
    let register_response = IdentityClient::register_user(
        req.username.as_deref(),
        req.password.as_deref(),
        req.mobile.as_deref(),
        req.email.as_deref(),
        nickname,
        req.avatar.as_deref(),
        req.region.as_deref(),
    )
    .await
    .map_err(|e| AuthError::ServiceUnavailable(format!("注册失败: {}", e)))?;

    // 2. 检查注册结果
    if !register_response.success {
        return Err(AuthError::BadRequest(register_response.message.clone()));
    }

    let user_id = register_response.user_id;
    tracing::info!("✅ 用户注册成功，用户ID: {}", user_id);

    Ok(Json(R::ok_with_data("注册成功，请登录".to_string())))
}

// ==================== 登录或注册融合接口 ====================

#[sa_ignore]
pub async fn login_or_register(
    State(state): State<AppState>,
    Json(req): Json<LoginOrRegisterRequest>,
) -> Result<Json<R<LoginOrRegisterResponse>>, AuthError> {
    // 0. 复用现有的校验逻辑进行验证码查验
    let login_req = LoginRequest {
        username: None,
        password: None,
        mobile: req.mobile.clone(),
        email: req.email.clone(),
        code: Some(req.code.clone()),
        captcha_id: None,
        captcha: None,
        region: req.region.clone(),
    };
    validate_login_request(&state, &login_req).await?;

    let account = req.mobile.as_deref().or(req.email.as_deref()).unwrap_or("");
    if account.is_empty() {
        return Err(AuthError::BadRequest("手机号或邮箱不能为空".to_string()));
    }

    // 1. 通过 ms-identity 的 search_user 查询是否存在该账户
    let search_resp = IdentityClient::search_user(account)
        .await
        .map_err(|e| AuthError::ServiceUnavailable(format!("检查用户状态失败: {}", e)))?;

    let is_new_user = !search_resp.success || search_resp.user.is_none();

    let user_id = if is_new_user {
        // 全新用户自动执行无感注册
        let config = NicknameConfig {
            mode: NicknameMode::Auto,
            with_number: false,
            max_length: 16,
        };
        let nickname = NicknameGenerator::generate(config);

        let register_response = IdentityClient::register_user(
            None,
            Some("123456"), // 默认内部占位密码或保持兼容性随机分配
            req.mobile.as_deref(),
            req.email.as_deref(),
            Some(&nickname),
            None,
            req.region.as_deref(),
        )
        .await
        .map_err(|e| AuthError::ServiceUnavailable(format!("自动注册失败: {}", e)))?;

        if !register_response.success {
            return Err(AuthError::InternalError(register_response.message));
        }
        tracing::info!(
            "✅ 融合登录：检测为新用户并注册成功，用户ID: {}",
            register_response.user_id
        );
        register_response.user_id
    } else {
        search_resp.user.unwrap().id
    };

    // 2. 组装用户信息与租户，发放登录 Token (复用常规登录逻辑)
    let tenants_response = IdentityClient::get_user_tenants(user_id)
        .await
        .map_err(|e| AuthError::ServiceUnavailable(format!("获取租户列表失败: {}", e)))?;

    if !tenants_response.success {
        return Err(AuthError::InternalError(tenants_response.message));
    }

    let tenants = tenants_response.tenants;

    // 提取最新的用户信息返回
    let user_resp = IdentityClient::get_user_info(user_id)
        .await
        .map_err(|e| AuthError::ServiceUnavailable(format!("获取用户信息失败: {}", e)))?;
    if !user_resp.success {
        return Err(AuthError::InternalError(user_resp.message));
    }
    let user = user_resp.user.unwrap();

    let user_info = UserInfo {
        id: user_id.to_string(),
        nickname: user.nick_name.clone(),
        avatar: if user.avatar.is_empty() {
            None
        } else {
            Some(user.avatar.clone())
        },
    };

    let valid_tenants: Vec<_> = tenants.into_iter().filter(|t| t.status == 1).collect();

    // 如果没有租户（如对于刚注册的新人由于并未默认分配），为了不妨碍使用，可能需要先进入首页创建租户，这里暂时原样打回或发放Token。（这里延续原逻辑抛错，或者为新用户特事特办发放Token，由于原系统要求必须有租户，我们需要保证这里过关。因为这是自动注册的，如果不分配租户会报错。）

    if valid_tenants.is_empty() {
        // TODO: 可选：如果新用户没有默认租户，可以通过后端逻辑为其创建。如果系统允许无租户的 Token，这里应改写。
        return Err(AuthError::Forbidden(
            "用户没有可用的租户，请先创建租户".to_string(),
        ));
    }

    if valid_tenants.len() == 1 {
        let tenant = &valid_tenants[0];
        let extra = serde_json::json!({
            "tenant_id": tenant.tenant_id.to_string(),
            "username": user.username,
            "token_type": "access"
        });

        let token_value = StpUtil::login_with_extra(user_id, extra.clone()).await?;
        let token = token_value.to_string();

        let login_id = user_id.to_string();
        let refresh_token = state.refresh_token_mgr.generate(&login_id);
        state
            .refresh_token_mgr
            .store_with_extra(&refresh_token, &token, &login_id, Some(&extra))
            .await
            .map_err(|e| AuthError::InternalError(format!("存储refresh token失败: {}", e)))?;

        tracing::info!("✅ 用户 {} 融合登录成功，单租户直接返回JWT token", user_id);

        return Ok(Json(R::ok_with_data(LoginOrRegisterResponse {
            is_new_user,
            login_info: LoginResponse {
                access_token: token,
                refresh_token,
                expires_in: state.auth_config.auth.access_token_timeout,
                refresh_expires_in: state.auth_config.auth.refresh_token_timeout,
                user_info,
                tenant_list: None,
            },
        })));
    }

    let tenant_list: Vec<TenantInfo> = valid_tenants
        .into_iter()
        .map(|t| TenantInfo {
            id: t.tenant_id.to_string(),
            name: t.name.clone(),
            is_owner: Some(t.is_owner == 1),
        })
        .collect();

    let temp_token_data: Vec<crate::service::temp_token::TenantInfo> = tenant_list
        .iter()
        .map(|t| crate::service::temp_token::TenantInfo {
            tenant_id: t.id.parse().unwrap_or(0),
            is_owner: t.is_owner.unwrap_or(false),
        })
        .collect();

    let temp_token = TempTokenService::create_temp_token(&state, user_id, temp_token_data)
        .await
        .map_err(|e| AuthError::InternalError(format!("生成临时token失败: {}", e)))?;

    tracing::info!("✅ 用户 {} 融合登录成功，多租户返回临时token", user_id);

    Ok(Json(R::ok_with_data(LoginOrRegisterResponse {
        is_new_user,
        login_info: LoginResponse {
            access_token: temp_token,
            refresh_token: String::new(),
            expires_in: 0, // 临时token不返回过期时间
            refresh_expires_in: 0,
            user_info,
            tenant_list: Some(tenant_list),
        },
    })))
}

/// 选择租户接口
#[sa_ignore]
pub async fn select_tenant(
    State(state): State<AppState>,
    Json(req): Json<SelectTenantRequest>,
) -> Result<Json<R<SelectTenantResponse>>, AuthError> {
    // 1. 验证临时 token 并获取数据
    let temp_token_data = TempTokenService::verify_and_get(&state, &req.temp_token)
        .await
        .map_err(|e| AuthError::Unauthorized(format!("临时token无效或已过期: {}", e)))?;

    // 2. 验证租户ID是否在租户列表中
    let _tenant = temp_token_data
        .tenant_list
        .iter()
        .find(|t| t.tenant_id == req.tenant_id)
        .ok_or_else(|| AuthError::BadRequest("租户ID不在可用列表中".to_string()))?;

    // 3. 获取用户信息
    let user_response = IdentityClient::get_user_info(temp_token_data.user_id)
        .await
        .map_err(|e| AuthError::ServiceUnavailable(format!("获取用户信息失败: {}", e)))?;

    if !user_response.success {
        return Err(AuthError::InternalError(user_response.message.clone()));
    }

    let user = user_response
        .user
        .ok_or_else(|| AuthError::InternalError("用户信息为空".to_string()))?;

    // 4. 构造 extra claims 并通过 StpUtil 登录（JWT生成 + 会话创建）
    let extra = serde_json::json!({
        "tenant_id": req.tenant_id.to_string(),
        "username": user.username,
        "token_type": "access"
    });

    let token_value = StpUtil::login_with_extra(temp_token_data.user_id, extra.clone()).await?;
    let token = token_value.to_string();

    // 5. 生成 refresh token
    let login_id = temp_token_data.user_id.to_string();
    let refresh_token = state.refresh_token_mgr.generate(&login_id);
    state
        .refresh_token_mgr
        .store_with_extra(&refresh_token, &token, &login_id, Some(&extra))
        .await
        .map_err(|e| AuthError::InternalError(format!("存储refresh token失败: {}", e)))?;

    let user_info = UserInfo {
        id: user.id.to_string(),
        nickname: user.nick_name.clone(),
        avatar: if user.avatar.is_empty() {
            None
        } else {
            Some(user.avatar.clone())
        },
    };

    tracing::info!(
        "✅ 用户 {} 选择租户 {} 成功",
        temp_token_data.user_id,
        req.tenant_id
    );

    Ok(Json(R::ok_with_data(SelectTenantResponse {
        access_token: token,
        refresh_token,
        expires_in: state.auth_config.auth.access_token_timeout,
        refresh_expires_in: state.auth_config.auth.refresh_token_timeout,
        user_info,
    })))
}

/// 获取用户信息接口
#[sa_check_login]
pub async fn user_profile() -> Result<Json<R<UserInfo>>, AuthError> {
    // 从当前上下文获取用户 ID（StpUtil 会自动从 SaTokenContext 中获取）
    let login_id = StpUtil::get_login_id_as_string()
        .await
        .map_err(|e| AuthError::Unauthorized(format!("获取用户ID失败: {}", e)))?;

    let user_id: i64 = login_id
        .parse()
        .map_err(|_| AuthError::BadRequest("用户ID格式错误".to_string()))?;

    // 从 identity 服务获取用户信息
    let user_response = IdentityClient::get_user_info(user_id)
        .await
        .map_err(|e| AuthError::ServiceUnavailable(format!("获取用户信息失败: {}", e)))?;

    if !user_response.success {
        return Err(AuthError::InternalError(user_response.message.clone()));
    }

    let user = user_response
        .user
        .ok_or_else(|| AuthError::InternalError("用户信息为空".to_string()))?;

    let info = UserInfo {
        id: user.id.to_string(),
        nickname: user.nick_name.clone(),
        avatar: if user.avatar.is_empty() {
            None
        } else {
            Some(user.avatar.clone())
        },
    };

    Ok(Json(R::ok_with_data(info)))
}

/// 登出接口
#[sa_check_login]
pub async fn logout() -> Result<Json<R<()>>, AuthError> {
    // 使用 StpUtil 登出当前会话
    StpUtil::logout_current()
        .await
        .map_err(|e| AuthError::InternalError(format!("登出失败: {}", e)))?;

    tracing::info!("✅ 用户登出成功");
    Ok(Json(R::ok()))
}

// ==================== 刷新 Token 接口 ====================

/// 刷新 Token 接口
///
/// 使用 refresh_token 获取新的 access_token + refresh_token 对。
/// 旧 token 会被失效（通过 StpUtil::logout_by_login_id）。
#[sa_ignore]
pub async fn refresh_token(
    State(state): State<AppState>,
    Json(req): Json<RefreshTokenRequest>,
) -> Result<Json<R<RefreshTokenResponse>>, AuthError> {
    // 1. 使用 RefreshTokenManager 刷新 access_token（内部自动更新保持 extra_data）
    let (new_access_token, login_id) = state
        .refresh_token_mgr
        .refresh_access_token(&req.refresh_token)
        .await
        .map_err(|e| AuthError::Unauthorized(format!("refresh token 无效或已过期: {}", e)))?;

    // 注意：取消“删除旧 refresh token”和“重新生成 refresh token”这会导致 extra_data 丢失的错误行为。
    // 源码中 refresh_access_token() 已经正确把 extra_data 原样保留了下来并映射到了新 access_token。
    
    tracing::info!("✅ 用户 {} token 刷新成功", login_id);

    Ok(Json(R::ok_with_data(RefreshTokenResponse {
        access_token: new_access_token.to_string(),
        refresh_token: req.refresh_token, // 继续使用此刷新令牌以保全 extra_data
        expires_in: state.auth_config.auth.access_token_timeout,
    })))
}
