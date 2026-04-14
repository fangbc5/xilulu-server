# OAuth2/OIDC/SSO 实施指南

## 快速开始

本文档提供实施 OAuth2/OIDC/SSO 功能的详细步骤和代码示例。

## 第一步：添加依赖

在 `Cargo.toml` 中添加必要的依赖：

```toml
[dependencies]
# JWT 支持
jsonwebtoken = "9.2"
chrono = { version = "0.4", features = ["serde"] }

# OAuth2/OIDC 相关
oauth2 = "4.4"  # 可选，用于客户端实现
url = "2.5"

# 加密支持（用于 PKCE）
sha2 = "0.10"
base64ct = "1.6"

# 异步 Trait
async-trait = "0.1"
```

## 第二步：实现 Token 生成器

创建 `src/token/jwt.rs`：

```rust
use crate::token::{TokenClaims, TokenGenerator, TokenType, IdTokenClaims};
use crate::error::AuthError;
use jsonwebtoken::{encode, decode, Header, Algorithm, EncodingKey, DecodingKey, Validation};
use chrono::{Utc, Duration};
use async_trait::async_trait;
use std::sync::Arc;

pub struct JwtTokenGenerator {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    issuer: String,
    access_token_ttl: u64,
    refresh_token_ttl: u64,
}

impl JwtTokenGenerator {
    pub fn new(
        secret: &str,
        issuer: String,
        access_token_ttl: u64,
        refresh_token_ttl: u64,
    ) -> Self {
        let encoding_key = EncodingKey::from_secret(secret.as_ref());
        let decoding_key = DecodingKey::from_secret(secret.as_ref());
        
        Self {
            encoding_key,
            decoding_key,
            issuer,
            access_token_ttl,
            refresh_token_ttl,
        }
    }
}

#[async_trait]
impl TokenGenerator for JwtTokenGenerator {
    async fn generate_access_token(
        &self,
        mut claims: TokenClaims,
    ) -> Result<String, AuthError> {
        let now = Utc::now();
        claims.iat = now.timestamp();
        claims.exp = (now + Duration::seconds(self.access_token_ttl as i64)).timestamp();
        claims.token_type = TokenType::AccessToken;
        
        encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| AuthError::InternalError(format!("Failed to encode token: {}", e)))
    }

    async fn generate_refresh_token(
        &self,
        mut claims: TokenClaims,
    ) -> Result<String, AuthError> {
        let now = Utc::now();
        claims.iat = now.timestamp();
        claims.exp = (now + Duration::seconds(self.refresh_token_ttl as i64)).timestamp();
        claims.token_type = TokenType::RefreshToken;
        
        encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| AuthError::InternalError(format!("Failed to encode token: {}", e)))
    }

    async fn generate_id_token(
        &self,
        mut claims: IdTokenClaims,
    ) -> Result<String, AuthError> {
        let now = Utc::now();
        claims.base.iat = now.timestamp();
        claims.base.exp = (now + Duration::seconds(3600)).timestamp(); // 1小时
        claims.base.token_type = TokenType::IdToken;
        claims.iss = self.issuer.clone();
        claims.aud = claims.base.client_id.clone();
        
        encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| AuthError::InternalError(format!("Failed to encode ID token: {}", e)))
    }

    async fn verify_token(
        &self,
        token: &str,
        token_type: TokenType,
    ) -> Result<TokenClaims, AuthError> {
        let mut validation = Validation::default();
        validation.algorithms = vec![Algorithm::HS256];
        
        let token_data = decode::<TokenClaims>(token, &self.decoding_key, &validation)
            .map_err(|e| AuthError::Unauthorized(format!("Invalid token: {}", e)))?;
        
        let claims = token_data.claims;
        
        // 验证 Token 类型
        if claims.token_type != token_type {
            return Err(AuthError::Unauthorized("Token type mismatch".to_string()));
        }
        
        // 验证是否过期
        let now = Utc::now().timestamp();
        if claims.exp < now {
            return Err(AuthError::Unauthorized("Token expired".to_string()));
        }
        
        Ok(claims)
    }
}
```

## 第三步：实现 Token 存储（Redis）

创建 `src/token/store.rs`：

```rust
use crate::token::{TokenClaims, TokenStore};
use crate::error::AuthError;
use crate::AppState;
use async_trait::async_trait;
use fbc_starter::cache::TokenService;
use serde_json;

pub struct RedisTokenStore {
    app_state: AppState,
}

impl RedisTokenStore {
    pub fn new(app_state: AppState) -> Self {
        Self { app_state }
    }
}

#[async_trait]
impl TokenStore for RedisTokenStore {
    async fn store_token(
        &self,
        token: &str,
        claims: &TokenClaims,
        ttl: u64,
    ) -> Result<(), AuthError> {
        let mut redis_conn = self.app_state
            .fbc_app_state
            .redis()
            .await
            .map_err(|e| AuthError::InternalError(format!("Redis connection failed: {}", e)))?;
        
        let key = format!("token:access:{}", token);
        let value = serde_json::to_string(claims)
            .map_err(|e| AuthError::InternalError(format!("Failed to serialize claims: {}", e)))?;
        
        TokenService::set_token(&mut redis_conn, &key, &value, ttl)
            .await
            .map_err(|e| AuthError::InternalError(format!("Failed to store token: {}", e)))?;
        
        Ok(())
    }

    async fn get_token(
        &self,
        token: &str,
    ) -> Result<Option<TokenClaims>, AuthError> {
        let mut redis_conn = self.app_state
            .fbc_app_state
            .redis()
            .await
            .map_err(|e| AuthError::InternalError(format!("Redis connection failed: {}", e)))?;
        
        let key = format!("token:access:{}", token);
        let value: Option<String> = TokenService::get_token(&mut redis_conn, &key)
            .await
            .map_err(|e| AuthError::InternalError(format!("Failed to get token: {}", e)))?;
        
        if let Some(v) = value {
            let claims: TokenClaims = serde_json::from_str(&v)
                .map_err(|e| AuthError::InternalError(format!("Failed to deserialize claims: {}", e)))?;
            Ok(Some(claims))
        } else {
            Ok(None)
        }
    }

    async fn revoke_token(&self, token: &str) -> Result<(), AuthError> {
        let mut redis_conn = self.app_state
            .fbc_app_state
            .redis()
            .await
            .map_err(|e| AuthError::InternalError(format!("Redis connection failed: {}", e)))?;
        
        // 添加到撤销列表
        let revoke_key = format!("revoke:token:{}", token);
        TokenService::set_token(&mut redis_conn, &revoke_key, "1", 86400) // 24小时
            .await
            .map_err(|e| AuthError::InternalError(format!("Failed to revoke token: {}", e)))?;
        
        // 删除原 Token
        let token_key = format!("token:access:{}", token);
        TokenService::delete_token(&mut redis_conn, &token_key)
            .await
            .map_err(|e| AuthError::InternalError(format!("Failed to delete token: {}", e)))?;
        
        Ok(())
    }

    async fn is_revoked(&self, token: &str) -> Result<bool, AuthError> {
        let mut redis_conn = self.app_state
            .fbc_app_state
            .redis()
            .await
            .map_err(|e| AuthError::InternalError(format!("Redis connection failed: {}", e)))?;
        
        let revoke_key = format!("revoke:token:{}", token);
        let exists: bool = TokenService::exists_token(&mut redis_conn, &revoke_key)
            .await
            .map_err(|e| AuthError::InternalError(format!("Failed to check revocation: {}", e)))?;
        
        Ok(exists)
    }
}
```

## 第四步：实现客户端注册表

创建 `src/client/registry.rs`：

```rust
use crate::client::{ClientRegistry, OAuth2Client};
use crate::error::AuthError;
use crate::AppState;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct InMemoryClientRegistry {
    clients: Arc<RwLock<HashMap<String, OAuth2Client>>>,
}

impl InMemoryClientRegistry {
    pub fn new() -> Self {
        Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    // 初始化默认客户端（用于测试）
    pub async fn init_default_clients(&self) {
        let mut clients = self.clients.write().await;
        
        // 示例：Web 应用客户端
        clients.insert("web-app".to_string(), OAuth2Client {
            client_id: "web-app".to_string(),
            client_secret: Some("web-app-secret".to_string()),
            client_type: crate::client::ClientType::Confidential,
            redirect_uris: vec!["http://localhost:3000/callback".to_string()],
            grant_types: vec![crate::flow::GrantType::AuthorizationCode],
            scopes: vec!["openid".to_string(), "profile".to_string(), "email".to_string()],
            name: "Web Application".to_string(),
            description: Some("Example web application".to_string()),
            tenant_id: None,
            enabled: true,
        });
    }
}

#[async_trait]
impl ClientRegistry for InMemoryClientRegistry {
    async fn get_client(
        &self,
        client_id: &str,
    ) -> Result<Option<OAuth2Client>, AuthError> {
        let clients = self.clients.read().await;
        Ok(clients.get(client_id).cloned())
    }

    async fn register_client(
        &self,
        client: OAuth2Client,
    ) -> Result<OAuth2Client, AuthError> {
        let mut clients = self.clients.write().await;
        let client_id = client.client_id.clone();
        clients.insert(client_id.clone(), client.clone());
        Ok(client)
    }

    async fn update_client(
        &self,
        client_id: &str,
        client: OAuth2Client,
    ) -> Result<(), AuthError> {
        let mut clients = self.clients.write().await;
        if clients.contains_key(client_id) {
            clients.insert(client_id.to_string(), client);
            Ok(())
        } else {
            Err(AuthError::BadRequest("Client not found".to_string()))
        }
    }

    async fn delete_client(&self, client_id: &str) -> Result<(), AuthError> {
        let mut clients = self.clients.write().await;
        clients.remove(client_id);
        Ok(())
    }
}
```

## 第五步：实现授权码流程

创建 `src/flow/authorization_code.rs`：

```rust
use crate::flow::{FlowHandler, AuthorizeRequest, AuthorizeResponse, TokenRequest, TokenResponse};
use crate::error::AuthError;
use crate::code::CodeManager;
use crate::client::ClientRegistry;
use crate::token::{TokenGenerator, TokenStore, TokenClaims, TokenType};
use async_trait::async_trait;
use uuid::Uuid;

pub struct AuthorizationCodeFlow {
    code_manager: Box<dyn CodeManager>,
    client_registry: Box<dyn ClientRegistry>,
    token_generator: Box<dyn TokenGenerator>,
    token_store: Box<dyn TokenStore>,
}

impl AuthorizationCodeFlow {
    pub fn new(
        code_manager: Box<dyn CodeManager>,
        client_registry: Box<dyn ClientRegistry>,
        token_generator: Box<dyn TokenGenerator>,
        token_store: Box<dyn TokenStore>,
    ) -> Self {
        Self {
            code_manager,
            client_registry,
            token_generator,
            token_store,
        }
    }
}

#[async_trait]
impl FlowHandler for AuthorizationCodeFlow {
    async fn authorize(
        &self,
        req: AuthorizeRequest,
    ) -> Result<AuthorizeResponse, AuthError> {
        // 1. 验证客户端
        let client = self.client_registry
            .get_client(&req.client_id)
            .await?
            .ok_or_else(|| AuthError::BadRequest("Invalid client_id".to_string()))?;
        
        if !client.enabled {
            return Err(AuthError::BadRequest("Client is disabled".to_string()));
        }
        
        // 2. 验证重定向 URI
        client.validate_redirect_uri(&req.redirect_uri)?;
        
        // 3. 验证 response_type
        if req.response_type != "code" {
            return Err(AuthError::BadRequest("Invalid response_type".to_string()));
        }
        
        // 4. 生成授权码
        let code = Uuid::new_v4().to_string();
        
        // TODO: 获取当前用户 ID（从会话中）
        let user_id = 123; // 临时值，实际应从会话获取
        
        self.code_manager
            .generate_code(
                &req.client_id,
                user_id,
                &req.redirect_uri,
                req.scope.as_deref().unwrap_or("").split(' ').map(|s| s.to_string()).collect::<Vec<_>>().as_slice(),
                req.code_challenge.clone(),
            )
            .await?;
        
        Ok(AuthorizeResponse {
            code: Some(code),
            state: req.state,
            redirect_uri: req.redirect_uri,
        })
    }

    async fn exchange_token(
        &self,
        req: TokenRequest,
    ) -> Result<TokenResponse, AuthError> {
        // 1. 验证客户端
        let client = self.client_registry
            .get_client(&req.client_id)
            .await?
            .ok_or_else(|| AuthError::BadRequest("Invalid client_id".to_string()))?;
        
        client.validate_secret(req.client_secret.as_deref())?;
        
        // 2. 验证授权码
        let code_info = self.code_manager
            .consume_code(
                req.code.as_deref().ok_or_else(|| AuthError::BadRequest("code is required".to_string()))?,
                &req.client_id,
                req.client_secret.as_deref(),
                req.redirect_uri.as_deref().ok_or_else(|| AuthError::BadRequest("redirect_uri is required".to_string()))?,
            )
            .await?;
        
        // 3. 生成 Token
        let claims = TokenClaims {
            sub: code_info.user_id.to_string(),
            client_id: req.client_id.clone(),
            scope: code_info.scopes,
            exp: 0, // 将在生成时设置
            iat: 0, // 将在生成时设置
            tenant_id: None,
            token_type: TokenType::AccessToken,
        };
        
        let access_token = self.token_generator.generate_access_token(claims.clone()).await?;
        let refresh_token = self.token_generator.generate_refresh_token(claims.clone()).await?;
        
        // 4. 存储 Token
        self.token_store.store_token(&access_token, &claims, 900).await?;
        
        Ok(TokenResponse {
            access_token,
            token_type: "Bearer".to_string(),
            expires_in: 900,
            refresh_token: Some(refresh_token),
            scope: Some(code_info.scopes.join(" ")),
            id_token: None,
        })
    }

    fn validate(&self, req: &AuthorizeRequest) -> Result<(), AuthError> {
        // 验证必需参数
        if req.client_id.is_empty() {
            return Err(AuthError::BadRequest("client_id is required".to_string()));
        }
        if req.redirect_uri.is_empty() {
            return Err(AuthError::BadRequest("redirect_uri is required".to_string()));
        }
        if req.response_type != "code" {
            return Err(AuthError::BadRequest("response_type must be 'code'".to_string()));
        }
        Ok(())
    }
}
```

## 第六步：创建 OAuth2 端点处理器

创建 `src/handler/oauth2.rs`：

```rust
use axum::{extract::Query, extract::State, Form, Json};
use crate::flow::{AuthorizeRequest, TokenRequest};
use crate::error::AuthError;
use crate::AppState;
use fbc_starter::R;

/// OAuth2 授权端点
/// GET /oauth2/authorize
pub async fn authorize(
    State(state): State<AppState>,
    Query(params): Query<AuthorizeRequest>,
) -> Result<Json<R<AuthorizeResponse>>, AuthError> {
    // TODO: 实现授权逻辑
    todo!()
}

/// OAuth2 Token 端点
/// POST /oauth2/token
pub async fn token(
    State(state): State<AppState>,
    Form(params): Form<TokenRequest>,
) -> Result<Json<TokenResponse>, AuthError> {
    // TODO: 实现 Token 交换逻辑
    todo!()
}
```

## 第七步：更新路由

更新 `src/router.rs`：

```rust
use axum::routing::{get, post};
use axum::Router;
use sa_token_plugin_axum::SaTokenLayer;

use crate::handler::*;
use crate::state::AppState;

pub fn create_router(app_state: AppState) -> Router {
    Router::new()
        // 原有接口
        .route("/api/login", post(login))
        .route("/api/register", post(register))
        .route("/api/select-tenant", post(select_tenant))
        .route("/api/captcha", get(image_captcha))
        .route("/api/send-verify-code", post(send_verify_code))
        
        // OAuth2 端点
        .route("/oauth2/authorize", get(oauth2::authorize))
        .route("/oauth2/token", post(oauth2::token))
        .route("/oauth2/introspect", post(oauth2::introspect))
        .route("/oauth2/revoke", post(oauth2::revoke))
        
        // OIDC 端点
        .route("/.well-known/openid-configuration", get(oidc::discovery))
        .route("/.well-known/jwks.json", get(oidc::jwks))
        .route("/oidc/userinfo", get(oidc::userinfo))
        
        // 需要登录的接口
        .route("/api/user/profile", get(user_profile))
        .route("/api/logout", get(logout))
        .layer(SaTokenLayer::new(app_state.sa_token.clone()))
        .with_state(app_state.clone())
}
```

## 下一步

1. 实现授权码管理器（`src/code/mod.rs`）
2. 实现 SSO 会话管理器（`src/session/mod.rs`）
3. 实现 OIDC Discovery 端点
4. 添加单元测试和集成测试
5. 实现 PKCE 支持
6. 添加速率限制中间件

详细的实现代码请参考架构设计文档。
