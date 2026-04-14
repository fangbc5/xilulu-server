# OAuth2 / OpenID Connect / SSO 架构设计文档

## 1. 架构概述

### 1.1 设计目标

- **复用 sa-token**：充分利用 sa-token 已有的会话管理、权限控制、SSO 能力
- **协议标准化**：在 sa-token 基础上新增 OAuth2/OIDC 标准协议支持
- **最小侵入**：不修改 sa-token 核心，只在上层构建协议适配层
- **通用性**：支持多种认证协议（OAuth2、OIDC），未来可扩展 SAML
- **可扩展性**：易于添加新的客户端类型和授权流程
- **高性能**：复用 sa-token 的 Redis 存储，保持 O(1) 性能
- **安全性**：符合 OAuth2/OIDC 安全最佳实践

### 1.2 核心原则

1. **复用优先**：sa-token 已有的功能（会话、权限、SSO）直接复用，不重复实现
2. **协议适配层**：在 sa-token 之上构建 OAuth2/OIDC 协议适配层
3. **双 Token 策略**：
   - sa-token 的 token：用于内部服务认证（保持现状）
   - OAuth2/OIDC JWT token：用于外部客户端（新增）
4. **状态管理分离**：授权码、OAuth2 Token 独立管理，但复用 sa-token 的会话存储

### 1.3 sa-token 已有能力（无需重新实现）

✅ **已提供的能力**：
- 用户登录/登出（`StpUtil::login()`, `StpUtil::logout()`）
- Token 生成和管理（基于 Redis）
- 会话管理（Session 存储、过期、刷新）
- 权限验证（`sa_check_login` 中间件）
- 角色权限（RBAC 支持）
- SSO 单点登录（同域、跨域支持）
- Token 存储（Redis Storage）

⚠️ **需要适配的部分**：
- OAuth2/OIDC 标准协议端点
- 授权码流程
- JWT Token 生成（用于 OAuth2/OIDC）
- 客户端注册表
- Discovery 端点
- UserInfo 端点

## 2. 整体架构

```
┌───────────────────────────────────────────────────────────────┐
│                      Client Applications                      │
│              (Web App / Mobile App / SPA / API)               │
└───────────────────────┬───────────────────────────────────────┘
                        │
                        │ HTTP/HTTPS
                        │
┌───────────────────────▼───────────────────────────────────────┐
│                    API Gateway / Load Balancer                │
└───────────────────────┬───────────────────────────────────────┘
                        │
                        │
┌───────────────────────▼───────────────────────────────────────┐
│                      ms-auth Service                          │
│  ┌─────────────────────────────────────────────────────────┐  │
│  │          Protocol Abstraction Layer                     │  │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐               │  │
│  │  │ OAuth2   │  │  OIDC    │  │  SAML    │               │  │
│  │  │ Handler  │  │ Handler  │  │ Handler  │               │  │
│  │  └──────────┘  └──────────┘  └──────────┘               │  │
│  └─────────────────────────────────────────────────────────┘  │
│  ┌─────────────────────────────────────────────────────────┐  │
│  │          Authentication Flow Engine                     │ │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐            │ │
│  │  │ Auth Code│  │  Implicit │  │  Client  │            │ │
│  │  │  Flow    │  │   Flow    │  │Credentials│           │ │
│  │  └──────────┘  └──────────┘  └──────────┘            │ │
│  └─────────────────────────────────────────────────────────┘ │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │          Token Management                                │ │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐            │ │
│  │  │   JWT    │  │  Refresh │  │  Revoke  │            │ │
│  │  │ Generator│  │  Token   │  │  Service │            │ │
│  │  └──────────┘  └──────────┘  └──────────┘            │ │
│  └─────────────────────────────────────────────────────────┘ │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │          Session & State Management                     │ │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐            │ │
│  │  │  Auth    │  │  Session │  │  SSO     │            │ │
│  │  │  Code    │  │  Store   │  │  Manager │            │ │
│  │  └──────────┘  └──────────┘  └──────────┘            │ │
│  └─────────────────────────────────────────────────────────┘ │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │          Client Registry                                │ │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐            │ │
│  │  │ Client   │  │ Redirect │  │  Scopes  │            │ │
│  │  │  Store   │  │   URI    │  │  Manager │            │ │
│  │  └──────────┘  └──────────┘  └──────────┘            │ │
│  └─────────────────────────────────────────────────────────┘ │
└───────────────────────┬───────────────────────────────────────┘
                        │
        ┌───────────────┼───────────────┐
        │               │               │
┌───────▼──────┐ ┌──────▼──────┐ ┌──────▼──────┐
│    Redis     │ │ ms-identity │ │   Kafka     │
│  (Cache/     │ │   Service   │ │ (Events)    │
│   Session)   │ │             │ │             │
└──────────────┘ └─────────────┘ └─────────────┘
```

## 3. 核心模块设计

### 3.1 OAuth2/OIDC 协议适配层（新增）

**目的**：在 sa-token 之上构建标准协议支持，不修改 sa-token 核心

**设计思路**：
- OAuth2 授权端点 → 调用 sa-token 登录 → 复用现有会话
- OAuth2 Token 端点 → 基于 sa-token 会话生成 JWT Token
- OAuth2 验证端点 → 验证 JWT 或查询 sa-token 会话

**关键接口**（概念设计，不写具体代码）：

```
OAuth2 Authorization Endpoint (/oauth2/authorize)
  → 检查用户是否已登录（通过 sa-token）
  → 如果未登录，重定向到登录页面（复用现有登录流程）
  → 如果已登录，显示授权同意页面
  → 用户同意后，生成授权码
  → 重定向到客户端

OAuth2 Token Endpoint (/oauth2/token)
  → 验证授权码
  → 获取 sa-token 会话信息（user_id, tenant_id）
  → 生成 JWT Access Token（包含用户信息）
  → 生成 JWT Refresh Token
  → 可选：生成 OIDC ID Token
  → 返回 Token 响应

OAuth2 Introspect Endpoint (/oauth2/introspect)
  → 验证 JWT Token 签名
  → 查询 sa-token 会话是否有效
  → 返回 Token 状态

OIDC UserInfo Endpoint (/oidc/userinfo)
  → 验证 Access Token
  → 从 sa-token 会话获取用户信息
  → 调用 ms-identity 获取详细用户信息
  → 返回用户信息 JSON

OIDC Discovery Endpoint (/.well-known/openid-configuration)
  → 返回 OIDC 配置元数据（端点地址、支持的流程等）

OIDC JWKS Endpoint (/.well-known/jwks.json)
  → 返回公钥集合（用于验证 ID Token）
```

### 3.2 认证流程引擎（复用 sa-token + 新增 OAuth2 流程）

**目的**：管理不同的 OAuth2 授权流程，底层复用 sa-token

**流程设计**：

#### Authorization Code Flow（授权码流程）

```
1. 客户端重定向到 /oauth2/authorize
2. 检查 sa-token 会话（StpUtil::get_login_id()）
   - 如果未登录 → 重定向到现有登录页面（/api/login）
   - 如果已登录 → 继续流程
3. 显示授权同意页面（可选，如果用户已授权过可跳过）
4. 用户同意后：
   - 生成授权码（存储到 Redis: oauth2:code:{code}）
   - 重定向到客户端: {redirect_uri}?code={code}&state={state}

5. 客户端调用 /oauth2/token
   - 验证授权码
   - 从 sa-token 获取会话信息
   - 生成 JWT Access Token（包含 user_id, client_id, scopes）
   - 生成 JWT Refresh Token
   - 返回 Token
```

#### Refresh Token Flow（刷新流程）

```
1. 客户端调用 /oauth2/token (grant_type=refresh_token)
2. 验证 Refresh Token（JWT 签名 + 查询 Redis）
3. 从 Refresh Token 获取 user_id
4. 验证 sa-token 会话是否仍然有效
5. 生成新的 Access Token 和 Refresh Token
6. 返回新 Token
```

#### Client Credentials Flow（客户端凭证流程）

```
1. 客户端调用 /oauth2/token (grant_type=client_credentials)
2. 验证 client_id 和 client_secret
3. 生成 Access Token（不包含用户信息，只有 client_id）
4. 返回 Token（不返回 Refresh Token）
```

**关键点**：
- 用户认证：完全复用 sa-token 的登录逻辑
- 会话管理：复用 sa-token 的 Redis 会话存储
- Token 生成：新增 JWT Token 生成（用于 OAuth2/OIDC）
- Token 验证：JWT 签名验证 + sa-token 会话验证（双重验证）

### 3.3 Token 管理策略（双 Token 系统）

**目的**：在保持 sa-token 的同时，新增 OAuth2/OIDC 标准 Token

**双 Token 策略**：

#### sa-token Token（内部使用，保持不变）
- **用途**：内部服务认证、API 网关认证
- **格式**：sa-token 的随机字符串（当前实现）
- **存储**：Redis（sa-token 管理）
- **验证**：sa-token 中间件（`sa_check_login`）

#### OAuth2/OIDC JWT Token（外部客户端，新增）
- **用途**：第三方应用、SPA、移动应用
- **格式**：标准 JWT（包含签名）
- **存储**：
  - Access Token：可 stateless（JWT）或存储在 Redis（用于撤销）
  - Refresh Token：存储在 Redis（用于撤销和轮换）
- **验证**：JWT 签名验证 + 可选查询 sa-token 会话

**Token 映射关系**：

```
OAuth2 JWT Token
  ↓ (包含 user_id)
sa-token Session (Redis)
  ↓ (通过 user_id 关联)
用户权限、角色信息
```

**设计要点**：
1. **JWT Access Token**：
   - 包含：user_id, client_id, scopes, exp, iat
   - 签名算法：HS256（对称）或 RS256（非对称）
   - 过期时间：15 分钟（短期）
   - 可选存储：Redis（用于撤销检查）

2. **JWT Refresh Token**：
   - 包含：user_id, client_id, exp, iat
   - 过期时间：7 天（长期）
   - 必须存储：Redis（用于撤销和轮换）

3. **OIDC ID Token**：
   - 包含：用户信息（sub, name, email, picture 等）
   - 签名算法：RS256（非对称，推荐）
   - 过期时间：1 小时
   - 用途：客户端验证用户身份

**Token 撤销机制**：
- 撤销列表存储在 Redis：`oauth2:revoke:{token_hash}`
- 验证 Token 时检查撤销列表
- 支持批量撤销（撤销用户所有 Token）

### 3.4 客户端注册表（新增）

**目的**：管理 OAuth2/OIDC 客户端应用配置

**客户端信息结构**（概念设计）：

```
OAuth2Client {
    client_id: String,              // 客户端唯一标识
    client_secret: Option<String>,  // 客户端密钥（Public client 为 None）
    client_type: Confidential | Public,
    redirect_uris: Vec<String>,     // 允许的重定向 URI（支持通配符）
    grant_types: Vec<GrantType>,    // 允许的授权流程
    scopes: Vec<String>,            // 允许的作用域
    name: String,                   // 客户端名称
    description: Option<String>,    // 客户端描述
    tenant_id: Option<i64>,        // 租户 ID（多租户支持）
    enabled: bool,                  // 是否启用
    pkce_required: bool,            // 是否强制 PKCE（Public client）
    token_lifetime: u64,            // Token 过期时间（秒）
    refresh_token_enabled: bool,    // 是否支持 Refresh Token
}
```

**存储方案**：
- **方案1**：Redis 存储（适合小型部署）
  - Key: `oauth2:client:{client_id}`
  - Value: JSON 序列化的客户端信息
- **方案2**：数据库存储（推荐，适合生产环境）
  - 表：`oauth2_clients`
  - 支持动态注册和管理

**客户端验证流程**：
1. 从注册表获取客户端信息
2. 验证客户端是否启用
3. 验证重定向 URI 是否在允许列表中
4. 验证授权流程是否支持
5. 验证作用域是否允许
6. 验证客户端密钥（Confidential client）

### 3.5 会话管理（复用 sa-token）

**目的**：复用 sa-token 的会话管理能力，不重新实现

**sa-token 已提供的功能**：
- ✅ 用户登录创建会话（`StpUtil::login()`）
- ✅ 会话存储（Redis Storage）
- ✅ 会话过期管理（timeout 配置）
- ✅ 会话查询（`StpUtil::get_login_id()`）
- ✅ 会话销毁（`StpUtil::logout()`）
- ✅ SSO 支持（同域、跨域）

**OAuth2/OIDC 适配层需要做的**：
1. **授权码生成时**：
   - 从 sa-token 获取当前用户 ID
   - 将授权码与 sa-token 会话关联
   - 存储：`oauth2:code:{code} -> {user_id, client_id, redirect_uri, scopes}`

2. **Token 生成时**：
   - 从授权码获取 user_id
   - 验证 sa-token 会话是否仍然有效
   - 基于会话信息生成 JWT Token

3. **Token 验证时**：
   - 验证 JWT 签名
   - 可选：查询 sa-token 会话验证用户状态

4. **登出时**：
   - 调用 sa-token 登出（`StpUtil::logout()`）
   - 撤销所有相关的 OAuth2 Token
   - 可选：通知客户端登出（OIDC Logout）

**SSO 会话管理**：
- sa-token 已支持 SSO，OAuth2 适配层只需：
  - 在用户登录时创建 OAuth2 会话映射
  - 在用户登出时清理 OAuth2 Token
  - 支持跨应用 SSO（通过 sa-token 的 SSO 机制）

### 3.6 授权码管理（新增，独立于 sa-token）

**目的**：管理 OAuth2 授权码的生命周期

**授权码存储**（Redis）：
- Key: `oauth2:code:{code}`
- Value: JSON 序列化的授权码信息
- TTL: 5 分钟（一次性使用）

**授权码信息结构**：
```
AuthorizationCode {
    code: String,                    // 授权码（UUID）
    client_id: String,               // 客户端 ID
    user_id: i64,                    // 用户 ID（从 sa-token 会话获取）
    redirect_uri: String,            // 重定向 URI
    scopes: Vec<String>,             // 请求的作用域
    code_challenge: Option<String>,  // PKCE code_challenge
    code_challenge_method: Option<String>, // PKCE 方法（S256）
    created_at: i64,                 // 创建时间戳
    expires_at: i64,                 // 过期时间戳
}
```

**授权码流程**：
1. **生成授权码**：
   - 用户登录后（通过 sa-token）
   - 用户同意授权
   - 生成 UUID 授权码
   - 存储到 Redis（5 分钟 TTL）
   - 重定向到客户端

2. **消费授权码**：
   - 客户端调用 Token 端点
   - 验证授权码是否存在且未过期
   - 验证 client_id 匹配
   - 验证 redirect_uri 匹配
   - 验证 PKCE（如果使用）
   - 从授权码获取 user_id
   - 验证 sa-token 会话是否仍然有效
   - 删除授权码（一次性使用）
   - 生成 Token

**PKCE 支持**：
- 支持 `code_challenge` 和 `code_verifier`
- 方法：S256（SHA256）
- Public client 强制要求 PKCE

## 4. 目录结构设计

```
ms-auth/
├── src/
│   ├── protocol/              # 协议抽象层
│   │   ├── mod.rs
│   │   ├── trait.rs           # ProtocolHandler trait
│   │   ├── oauth2/            # OAuth2 实现
│   │   │   ├── mod.rs
│   │   │   ├── handler.rs
│   │   │   └── models.rs
│   │   ├── oidc/              # OpenID Connect 实现
│   │   │   ├── mod.rs
│   │   │   ├── handler.rs
│   │   │   ├── models.rs
│   │   │   └── discovery.rs   # .well-known/openid-configuration
│   │   └── saml/              # SAML 2.0 实现（未来）
│   │       └── mod.rs
│   ├── flow/                  # 授权流程引擎
│   │   ├── mod.rs
│   │   ├── trait.rs           # FlowHandler trait
│   │   ├── authorization_code.rs
│   │   ├── implicit.rs
│   │   ├── client_credentials.rs
│   │   ├── password.rs
│   │   └── refresh_token.rs
│   ├── token/                 # Token 管理
│   │   ├── mod.rs
│   │   ├── generator.rs       # TokenGenerator trait + JWT 实现
│   │   ├── store.rs           # TokenStore trait + Redis 实现
│   │   ├── jwt.rs             # JWT 工具
│   │   ├── claims.rs          # Token Claims 定义
│   │   └── revoke.rs          # Token 撤销服务
│   ├── client/                # 客户端注册表
│   │   ├── mod.rs
│   │   ├── registry.rs        # 客户端注册表
│   │   ├── store.rs           # 客户端存储（Redis/DB）
│   │   └── validator.rs       # 客户端验证
│   ├── session/               # 会话管理
│   │   ├── mod.rs
│   │   ├── manager.rs         # SessionManager trait
│   │   ├── store.rs          # 会话存储（Redis）
│   │   └── sso.rs            # SSO 会话管理
│   ├── code/                  # 授权码管理
│   │   ├── mod.rs
│   │   ├── manager.rs         # CodeManager trait
│   │   ├── store.rs          # 授权码存储（Redis）
│   │   └── pkce.rs           # PKCE 支持
│   ├── handler/               # HTTP 处理器
│   │   ├── mod.rs
│   │   ├── auth.rs           # 原有认证接口
│   │   ├── code.rs           # 验证码接口
│   │   ├── oauth2.rs         # OAuth2 端点
│   │   │   ├── authorize.rs  # /oauth2/authorize
│   │   │   ├── token.rs      # /oauth2/token
│   │   │   ├── introspect.rs # /oauth2/introspect
│   │   │   └── revoke.rs     # /oauth2/revoke
│   │   └── oidc.rs           # OIDC 端点
│   │       ├── authorize.rs  # /oidc/authorize
│   │       ├── token.rs      # /oidc/token
│   │       ├── userinfo.rs   # /oidc/userinfo
│   │       └── discovery.rs  # /.well-known/openid-configuration
│   ├── service/               # 业务服务层
│   │   ├── mod.rs
│   │   ├── image_captcha.rs
│   │   ├── verify_code.rs
│   │   ├── temp_token.rs
│   │   ├── validation.rs
│   │   └── nickname_generator.rs
│   ├── middleware/            # 中间件
│   │   ├── mod.rs
│   │   ├── oauth2.rs         # OAuth2 Token 验证中间件
│   │   └── rate_limit.rs     # 限流中间件
│   ├── model/                 # 数据模型
│   │   ├── mod.rs
│   │   ├── auth.rs
│   │   └── oauth2.rs         # OAuth2/OIDC 模型
│   ├── error.rs
│   ├── router.rs
│   └── state.rs
```

## 5. 关键实现细节

### 5.1 OAuth2 授权码流程

```
1. 客户端重定向到授权端点
   GET /oauth2/authorize?
     response_type=code
     &client_id=xxx
     &redirect_uri=xxx
     &scope=xxx
     &state=xxx
     &code_challenge=xxx (PKCE)

2. 用户登录（如果未登录）
   → 重定向到登录页面
   → 登录成功后回到授权页面

3. 用户授权
   → 生成授权码（5分钟有效期）
   → 存储到 Redis: auth_code:{code} -> {client_id, user_id, redirect_uri, scopes, code_challenge}
   → 重定向到客户端: {redirect_uri}?code={code}&state={state}

4. 客户端交换 Token
   POST /oauth2/token
     grant_type=authorization_code
     &code={code}
     &redirect_uri={redirect_uri}
     &client_id=xxx
     &client_secret=xxx
     &code_verifier=xxx (PKCE)

5. 验证授权码
   → 验证 code 是否存在且未过期
   → 验证 client_id、redirect_uri 匹配
   → 验证 PKCE（如果使用）
   → 消费授权码（删除）

6. 生成 Token
   → 生成 Access Token (JWT, 15分钟)
   → 生成 Refresh Token (JWT, 7天)
   → 存储到 Redis: token:{access_token} -> {claims}
   → 存储 Refresh Token 映射: refresh:{refresh_token} -> {access_token}

7. 返回 Token
   {
     "access_token": "...",
     "token_type": "Bearer",
     "expires_in": 900,
     "refresh_token": "...",
     "scope": "..."
   }
```

### 5.2 OpenID Connect 扩展

```
在 OAuth2 基础上添加：

1. ID Token
   → 生成 ID Token (JWT)
   → 包含用户信息：sub, name, email, picture 等
   → 签名使用 RS256（非对称加密）

2. UserInfo 端点
   GET /oidc/userinfo
   Authorization: Bearer {access_token}
   → 返回用户信息 JSON

3. Discovery 端点
   GET /.well-known/openid-configuration
   → 返回 OIDC 配置元数据

4. JWKs 端点
   GET /.well-known/jwks.json
   → 返回公钥集合（用于验证 ID Token）
```

### 5.3 SSO 会话管理

```
1. 用户首次登录
   → 创建 SSO 会话（全局会话）
   → 存储到 Redis: sso_session:{session_id} -> {user_id, tenant_id, created_at, last_access}
   → 设置 Cookie: SSO_SESSION_ID={session_id}

2. 用户访问其他应用
   → 检查 SSO 会话是否存在
   → 如果存在，自动创建应用级会话
   → 返回应用 Token

3. 用户登出
   → 销毁 SSO 会话
   → 通知所有应用登出（通过事件）
   → 清除 Cookie

4. 会话同步
   → 使用 Redis Pub/Sub 或 Kafka 事件
   → 应用监听会话变更事件
   → 自动同步会话状态
```

### 5.4 Token 存储策略

```rust
// Redis Key 设计

// Access Token
token:access:{token_hash} -> {
    "user_id": 123,
    "client_id": "xxx",
    "scopes": ["read", "write"],
    "expires_at": 1234567890
}
TTL: 15分钟

// Refresh Token
token:refresh:{token_hash} -> {
    "user_id": 123,
    "client_id": "xxx",
    "access_token": "xxx",
    "expires_at": 1234567890
}
TTL: 7天

// 授权码
code:auth:{code} -> {
    "client_id": "xxx",
    "user_id": 123,
    "redirect_uri": "xxx",
    "scopes": ["read"],
    "code_challenge": "xxx",
    "created_at": 1234567890
}
TTL: 5分钟

// SSO 会话
session:sso:{session_id} -> {
    "user_id": 123,
    "tenant_id": 456,
    "created_at": 1234567890,
    "last_access": 1234567890
}
TTL: 24小时

// Token 撤销列表（黑名单）
revoke:token:{token_hash} -> "1"
TTL: 与 Token 过期时间一致
```

## 6. 与现有系统集成（复用 sa-token 策略）

### 6.1 保留现有认证方式（完全不变）

**现有接口保持不变**：
- `/api/login` - 用户登录（使用 sa-token）
- `/api/register` - 用户注册
- `/api/logout` - 用户登出（使用 sa-token）
- `/api/user/profile` - 用户信息查询
- `/api/select-tenant` - 租户选择

**新增 OAuth2/OIDC 接口**（不影响现有接口）：
- `/oauth2/authorize` - OAuth2 授权端点
- `/oauth2/token` - OAuth2 Token 端点
- `/oauth2/introspect` - Token 验证端点
- `/oauth2/revoke` - Token 撤销端点
- `/oidc/userinfo` - OIDC 用户信息端点
- `/.well-known/openid-configuration` - OIDC Discovery
- `/.well-known/jwks.json` - JWKS 公钥端点

### 6.2 双 Token 系统（推荐方案）

**策略**：sa-token Token 和 OAuth2 JWT Token 并存，各司其职

#### sa-token Token（内部使用，保持不变）
- **用途**：
  - 内部服务间认证
  - 现有 API 接口认证（`/api/*`）
  - API 网关认证
- **格式**：sa-token 随机字符串（当前实现）
- **存储**：Redis（sa-token 管理）
- **验证**：sa-token 中间件（`sa_check_login`）
- **特点**：高性能、简单、适合内部使用

#### OAuth2/OIDC JWT Token（外部客户端，新增）
- **用途**：
  - 第三方应用集成
  - SPA（单页应用）
  - 移动应用
  - 需要标准协议的场景
- **格式**：标准 JWT（包含签名）
- **存储**：
  - Access Token：可 stateless（JWT）或 Redis（用于撤销）
  - Refresh Token：Redis（必须存储）
- **验证**：JWT 签名验证 + 可选查询 sa-token 会话
- **特点**：标准化、可验证、适合外部集成

**Token 映射关系**：
```
OAuth2 JWT Token (包含 user_id)
  ↓
sa-token Session (通过 user_id 关联)
  ↓
用户权限、角色信息（sa-token 管理）
```

**优势**：
- ✅ 不破坏现有系统
- ✅ 内部服务继续使用 sa-token（高性能）
- ✅ 外部客户端使用标准 OAuth2/OIDC（兼容性）
- ✅ 两种 Token 可以共存

### 6.3 客户端注册（新增）

**存储方案**：

**方案1：数据库存储（推荐，生产环境）**
- 表：`oauth2_clients`
- 字段：client_id, client_secret, redirect_uris, grant_types, scopes 等
- 支持动态注册（OAuth2 Dynamic Client Registration）
- 支持管理后台管理

**方案2：Redis 存储（适合小型部署）**
- Key: `oauth2:client:{client_id}`
- Value: JSON 序列化的客户端信息
- 不支持动态注册
- 适合测试和开发环境

**客户端管理接口**（新增）：
- `POST /oauth2/register` - 动态客户端注册（可选）
- `GET /admin/clients` - 客户端列表（管理后台）
- `POST /admin/clients` - 创建客户端（管理后台）
- `PUT /admin/clients/{client_id}` - 更新客户端（管理后台）
- `DELETE /admin/clients/{client_id}` - 删除客户端（管理后台）

### 6.4 会话管理集成（复用 sa-token）

**OAuth2 授权流程中的会话处理**：

1. **用户登录时**：
   ```
   OAuth2 授权端点 (/oauth2/authorize)
     ↓
   检查 sa-token 会话（StpUtil::get_login_id()）
     ↓
   如果未登录 → 重定向到现有登录页面 (/api/login)
     ↓
   用户登录 → sa-token 创建会话（StpUtil::login()）
     ↓
   重定向回 OAuth2 授权页面
   ```

2. **生成授权码时**：
   ```
   从 sa-token 获取用户 ID（StpUtil::get_login_id()）
     ↓
   生成授权码
     ↓
   存储授权码到 Redis（关联 user_id）
   ```

3. **生成 Token 时**：
   ```
   验证授权码
     ↓
   获取 user_id
     ↓
   验证 sa-token 会话是否有效（StpUtil::get_login_id()）
     ↓
   生成 JWT Token（包含 user_id）
   ```

4. **用户登出时**：
   ```
   调用 sa-token 登出（StpUtil::logout()）
     ↓
   撤销所有相关的 OAuth2 Token
     ↓
   可选：通知客户端登出（OIDC Logout）
   ```

**关键点**：
- ✅ 用户认证完全复用 sa-token
- ✅ 会话管理完全复用 sa-token
- ✅ OAuth2 适配层只负责协议转换
- ✅ 不修改 sa-token 核心代码

## 7. 安全性考虑

### 7.1 PKCE (Proof Key for Code Exchange)

- **必须支持**：SPA 和移动应用
- **实现**：code_challenge、code_verifier

### 7.2 Token 安全

- **签名算法**：HS256（对称）或 RS256（非对称）
- **Token 绑定**：可选的 IP/设备绑定
- **Token 撤销**：支持立即撤销
- **Token 轮换**：Refresh Token 使用后轮换

### 7.3 重定向 URI 验证

- **严格验证**：redirect_uri 必须在客户端注册列表中
- **通配符支持**：支持 `https://*.example.com/*` 模式
- **协议限制**：禁止不安全的 redirect_uri

### 7.4 速率限制

- **授权端点**：防止暴力破解
- **Token 端点**：防止 Token 滥用
- **基于 IP 和 Client ID**：双重限流

## 8. 性能优化

### 8.1 缓存策略

- **客户端信息**：Redis 缓存（5分钟）
- **Token 验证**：Redis 缓存（避免重复 JWT 验证）
- **用户信息**：Redis 缓存（1分钟）

### 8.2 批量操作

- **Token 撤销**：批量撤销用户所有 Token
- **会话查询**：批量获取用户会话列表

### 8.3 异步处理

- **事件发布**：Token 生成、撤销等事件异步发布到 Kafka
- **日志记录**：审计日志异步写入

## 9. 实施计划（基于 sa-token）

### Phase 1: OAuth2 基础端点（2-3周）

**目标**：实现 OAuth2 核心端点，复用 sa-token 登录

1. ✅ 实现客户端注册表（Redis 或数据库）
2. ✅ 实现授权码管理（Redis 存储）
3. ✅ 实现 JWT Token 生成器
4. ✅ 实现 `/oauth2/authorize` 端点
   - 复用 sa-token 登录检查
   - 复用现有登录页面
   - 生成授权码
5. ✅ 实现 `/oauth2/token` 端点
   - 验证授权码
   - 从 sa-token 获取用户信息
   - 生成 JWT Token

### Phase 2: OAuth2 完整流程（2-3周）

1. ✅ 实现授权码流程（Authorization Code Flow）
2. ✅ 实现客户端凭证流程（Client Credentials Flow）
3. ✅ 实现刷新 Token 流程（Refresh Token Flow）
4. ✅ 实现 PKCE 支持
5. ✅ 实现 Token 验证端点（`/oauth2/introspect`）
6. ✅ 实现 Token 撤销端点（`/oauth2/revoke`）

### Phase 3: OpenID Connect（2周）

1. ✅ 实现 ID Token 生成
2. ✅ 实现 UserInfo 端点（`/oidc/userinfo`）
3. ✅ 实现 Discovery 端点（`/.well-known/openid-configuration`）
4. ✅ 实现 JWKs 端点（`/.well-known/jwks.json`）
5. ✅ 实现 OIDC Logout（可选）

### Phase 4: SSO 增强（1-2周）

**注意**：sa-token 已支持 SSO，此阶段主要是 OAuth2 层面的增强

1. ✅ OAuth2 Token 与 sa-token 会话关联
2. ✅ 跨应用 SSO（通过 sa-token SSO 机制）
3. ✅ 登出通知（OIDC Logout）
4. ✅ 会话同步（可选，通过 Kafka 事件）

### Phase 5: 集成与测试（2周）

1. ✅ 与现有系统集成测试
2. ✅ OAuth2 客户端测试
3. ✅ OIDC 兼容性测试
4. ✅ 性能测试
5. ✅ 安全审计

**关键原则**：
- ✅ 不修改 sa-token 核心代码
- ✅ 复用 sa-token 的所有能力
- ✅ 只在协议层做适配
- ✅ 保持现有接口不变

## 10. sa-token 已有功能 vs 需要新增功能

### 10.1 sa-token 已提供的功能（无需重新实现）

| 功能模块 | sa-token 提供 | 如何使用 |
|---------|--------------|---------|
| **用户登录** | ✅ `StpUtil::login()` | 直接调用，创建会话 |
| **用户登出** | ✅ `StpUtil::logout()` | 直接调用，销毁会话 |
| **会话查询** | ✅ `StpUtil::get_login_id()` | 获取当前用户 ID |
| **会话存储** | ✅ Redis Storage | 自动管理，无需关心 |
| **会话过期** | ✅ timeout 配置 | 配置中设置 |
| **权限验证** | ✅ `sa_check_login` 中间件 | 中间件自动验证 |
| **角色权限** | ✅ RBAC 支持 | sa-token 提供 |
| **SSO 支持** | ✅ 同域、跨域 SSO | sa-token 提供 |
| **Token 生成** | ✅ 随机字符串 Token | 内部使用 |

### 10.2 需要新增的功能（OAuth2/OIDC 协议层）

| 功能模块 | 状态 | 说明 |
|---------|------|------|
| **OAuth2 授权端点** | ❌ 新增 | `/oauth2/authorize` - 复用 sa-token 登录 |
| **OAuth2 Token 端点** | ❌ 新增 | `/oauth2/token` - 生成 JWT Token |
| **OAuth2 验证端点** | ❌ 新增 | `/oauth2/introspect` - 验证 Token |
| **OAuth2 撤销端点** | ❌ 新增 | `/oauth2/revoke` - 撤销 Token |
| **OIDC UserInfo** | ❌ 新增 | `/oidc/userinfo` - 返回用户信息 |
| **OIDC Discovery** | ❌ 新增 | `/.well-known/openid-configuration` |
| **OIDC JWKS** | ❌ 新增 | `/.well-known/jwks.json` |
| **授权码管理** | ❌ 新增 | Redis 存储授权码 |
| **客户端注册表** | ❌ 新增 | 管理 OAuth2 客户端 |
| **JWT Token 生成** | ❌ 新增 | 生成标准 JWT Token |
| **PKCE 支持** | ❌ 新增 | 支持 code_challenge/verifier |

### 10.3 集成策略总结

**核心思想**：在 sa-token 之上构建 OAuth2/OIDC 协议适配层

**数据流示例**：

```
外部客户端请求 OAuth2 授权
  ↓
/oauth2/authorize 端点
  ↓
检查 sa-token 会话（StpUtil::get_login_id()）
  ↓
如果未登录 → 重定向到 /api/login（现有登录接口）
  ↓
用户登录 → sa-token 创建会话（StpUtil::login()）
  ↓
生成授权码 → 存储到 Redis（oauth2:code:{code}）
  ↓
重定向到客户端（带授权码）
  ↓
客户端调用 /oauth2/token
  ↓
验证授权码 → 获取 user_id
  ↓
验证 sa-token 会话（StpUtil::get_login_id()）
  ↓
生成 JWT Token（包含 user_id）
  ↓
返回 Token 给客户端
```

**关键优势**：
1. ✅ **零侵入**：不修改 sa-token 核心代码
2. ✅ **复用最大化**：用户认证、会话管理完全复用 sa-token
3. ✅ **双 Token 系统**：内部用 sa-token，外部用 OAuth2 JWT
4. ✅ **向后兼容**：现有接口和功能完全不受影响
5. ✅ **标准化**：支持 OAuth2/OIDC 标准协议

## 11. 示例代码结构

### 10.1 OAuth2 授权端点

```rust
// src/handler/oauth2/authorize.rs

pub async fn authorize(
    State(state): State<AppState>,
    Query(params): Query<AuthorizeParams>,
) -> Result<Response, AuthError> {
    // 1. 验证客户端
    let client = state.client_registry.get(&params.client_id).await?;
    
    // 2. 验证重定向 URI
    client.validate_redirect_uri(&params.redirect_uri)?;
    
    // 3. 检查用户是否已登录
    let session = get_sso_session(&state, &request).await?;
    
    if session.is_none() {
        // 重定向到登录页面
        return redirect_to_login(&params);
    }
    
    // 4. 检查用户是否已授权
    if !is_authorized(&state, &session.user_id, &params.client_id).await? {
        // 显示授权页面
        return show_consent_page(&params);
    }
    
    // 5. 生成授权码
    let code = state.code_manager.generate_code(
        &params.client_id,
        session.user_id,
        &params.redirect_uri,
        &params.scope,
        params.code_challenge.clone(),
    ).await?;
    
    // 6. 重定向到客户端
    Ok(redirect_to_client(&params.redirect_uri, &code, &params.state))
}
```

### 10.2 Token 端点

```rust
// src/handler/oauth2/token.rs

pub async fn token(
    State(state): State<AppState>,
    Form(params): Form<TokenParams>,
) -> Result<Json<TokenResponse>, AuthError> {
    // 1. 验证客户端
    let client = state.client_registry.get(&params.client_id).await?;
    client.validate_secret(params.client_secret.as_deref())?;
    
    // 2. 根据 grant_type 处理
    match params.grant_type {
        GrantType::AuthorizationCode => {
            handle_authorization_code(&state, &client, &params).await
        }
        GrantType::RefreshToken => {
            handle_refresh_token(&state, &client, &params).await
        }
        GrantType::ClientCredentials => {
            handle_client_credentials(&state, &client, &params).await
        }
        _ => Err(AuthError::UnsupportedGrantType),
    }
}
```

## 11. 配置示例

```toml
# config.toml

[oauth2]
# 授权码过期时间（秒）
authorization_code_ttl = 300
# Access Token 过期时间（秒）
access_token_ttl = 900
# Refresh Token 过期时间（秒）
refresh_token_ttl = 604800
# Token 签名算法
token_signing_algorithm = "RS256"
# Token 签名密钥路径
token_signing_key_path = "/etc/ms-auth/private_key.pem"
token_verification_key_path = "/etc/ms-auth/public_key.pem"

[oidc]
# ID Token 过期时间（秒）
id_token_ttl = 3600
# 发行者标识
issuer = "https://auth.example.com"
# 支持的声明
supported_claims = ["sub", "name", "email", "picture", "email_verified"]

[sso]
# SSO 会话过期时间（秒）
session_ttl = 86400
# 会话同步事件主题
session_sync_topic = "sso.session.sync"
# Cookie 域名
cookie_domain = ".example.com"
# Cookie 安全标志
cookie_secure = true
cookie_http_only = true

[client]
# 客户端存储类型（redis/database）
store_type = "database"
# 是否支持动态注册
dynamic_registration_enabled = true
# 默认客户端作用域
default_scopes = ["openid", "profile", "email"]
```

## 12. 总结

这个架构设计提供了：

1. **通用性**：通过抽象层支持多种协议
2. **可扩展性**：插件化设计，易于添加新功能
3. **高性能**：基于 Redis 缓存，O(1) 操作
4. **安全性**：符合 OAuth2/OIDC 安全最佳实践
5. **兼容性**：与现有系统无缝集成

下一步可以按照实施计划逐步实现各个模块。
