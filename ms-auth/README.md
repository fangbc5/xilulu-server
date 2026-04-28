# ms-auth — 认证鉴权服务

> 基于 Rust 构建的高性能身份认证与授权服务，提供登录/注册/验证码/多租户 Token 管理，致力于对标 Keycloak。

## 📋 目录

- [功能特性](#-功能特性)
- [技术栈](#️-技术栈)
- [架构设计](#-架构设计)
- [项目结构](#-项目结构)
- [快速开始](#-快速开始)
- [配置说明](#️-配置说明)
- [API 文档](#-api-文档)
- [与 Keycloak 对比](#-与-keycloak-功能对比)
- [发展计划](#-发展计划)
- [与其他服务的关系](#-与其他服务的关系)

---

## ✨ 功能特性

### 用户认证

- 🔐 **多种登录方式**
  - 用户名 + 密码 + 图片验证码
  - 手机号 + 短信验证码
  - 邮箱 + 邮件验证码
- 📝 **多种注册方式**
  - 用户名 + 密码 + 图片验证码
  - 手机号 + 验证码
  - 邮箱 + 验证码
  - 自动昵称生成（中文/英文/混合，低重复率）

### 多租户支持

- 🏢 **租户选择流程** — 单租户直登 / 多租户二级选择
- 🔑 **临时 Token 机制** — 多租户场景的安全租户选择
- 🏠 **租户隔离** — 基于 ms-identity 的租户数据隔离

### 验证码服务

- 🖼️ **图片验证码** — 5 位字符 + Base64 PNG + IP 频率限制 (1 分钟/次)
- 📱 **短信/邮箱验证码** — 6 位数字 + 账号频率限制 + Kafka 异步发送
- ⏰ **有效期管理** — Redis 存储 (5 分钟有效期) + 一次性使用

### Token 管理

- 🎫 **Sa-Token 集成** — 基于 Sa-Token Rust 版的 Token 生成
- 💾 **Redis 会话** — Token 会话 Redis 存储
- ⏱️ **过期管理** — 24 小时有效期 + 临时 Token 机制

---

## 🛠️ 技术栈

| 类目 | 技术 | 说明 |
|------|------|------|
| **框架** | fbc-starter (Axum) | HTTP 服务 |
| **认证** | Sa-Token (Rust 版) | Token 生成与校验 |
| **缓存** | Redis | Token/验证码/会话存储 |
| **消息队列** | Kafka (生产者) | 验证码异步发送 |
| **内部通信** | gRPC | 调用 ms-identity 查询用户 |
| **服务发现** | Nacos | 注册/发现/负载均衡 |

---

## 🏗 架构设计

### 登录流程

```
客户端
  │
  │ 1. 获取图片验证码
  │ ──────────────────► ms-auth ──► Redis (存储验证码)
  │ ◄────────────────── 返回 Base64 PNG
  │
  │ 2. 登录请求 (用户名+密码+验证码)
  │ ──────────────────► ms-auth
  │                        │ 3. 校验验证码 (Redis)
  │                        │ 4. gRPC 调用 ms-identity
  │                        │ ──────────────► ms-identity
  │                        │ ◄────────────── 用户信息
  │                        │
  │                        │ 5a. 单租户 → 直接返回 Token
  │ ◄────────────────── access_token
  │
  │                        │ 5b. 多租户 → 返回临时 Token + 租户列表
  │ ◄────────────────── temp_token + tenants
  │
  │ 6. 选择租户 (temp_token + tenant_id)
  │ ──────────────────► ms-auth
  │ ◄────────────────── access_token (正式 Token)
```

### 模块结构

```
ms-auth/
├── src/
│   ├── main.rs              # 入口 — Server::run 启动
│   ├── config.rs            # AuthConfig — 认证配置
│   ├── error.rs             # 错误定义
│   ├── state.rs             # AppState — 应用状态
│   ├── router.rs            # HTTP 路由
│   ├── handler/             # 🌐 HTTP 处理器
│   │   ├── auth.rs          # 登录/注册/登出/租户选择
│   │   └── code.rs          # 验证码获取/发送
│   ├── service/             # 🧩 业务服务
│   │   ├── image_captcha.rs # 图片验证码生成
│   │   ├── verify_code.rs   # 短信/邮箱验证码
│   │   ├── temp_token.rs    # 临时 Token 管理
│   │   ├── validation.rs    # 公共校验模块
│   │   └── nickname_generator.rs  # 昵称生成器
│   ├── client/              # 外部服务客户端
│   │   └── identity.rs      # ms-identity gRPC 客户端
│   ├── kafka/               # Kafka 生产者
│   │   └── mod.rs           # 验证码通知投递
│   └── model/               # 数据模型
│       └── dto.rs           # 请求/响应 DTO
├── .env.example             # 环境变量模板
├── Dockerfile               # Docker 构建
└── Cargo.toml
```

---

## 🚀 快速开始

### 前置条件

- Rust 1.80+
- Redis 7.0+
- Kafka 3.0+（用于验证码通知投递）
- Nacos 2.0+（服务发现 + ms-identity 负载均衡）

### 配置与运行

```bash
# 1. 复制环境变量
cp ms-auth/.env.example ms-auth/.env
# 编辑 .env，配置 Redis、Kafka、Nacos 连接信息

# 2. 运行
cargo run -p ms-auth
```

---

## ⚙️ 配置说明

| 变量 | 默认值 | 说明 |
|------|--------|------|
| `APP__SERVER__PORT` | `30002` | 服务端口 |
| `APP__LOG__LEVEL` | `info` | 日志级别 |
| `APP__REDIS__URL` | — | Redis 连接串 |
| `APP__REDIS__PASSWORD` | — | Redis 密码 |
| `APP__REDIS__POOL_SIZE` | `10` | Redis 连接池大小 |
| `APP__KAFKA__BROKERS` | — | Kafka Broker 地址 |
| `APP__NACOS__SERVER_ADDRS` | — | Nacos 地址 |
| `APP__NACOS__SERVICE_NAME` | `ms-auth` | 注册服务名 |
| `APP__NACOS__SUBSCRIBE_SERVICES` | `["ms-identity"]` | 订阅的服务列表 |
| `APP__AUTH__JWT_SECRET` | — | JWT 签名密钥 |
| `APP__AUTH__ACCESS_TOKEN_TIMEOUT` | `900` | Access Token 过期时间(秒) |
| `APP__AUTH__REFRESH_TOKEN_TIMEOUT` | `604800` | Refresh Token 过期时间(秒) |
| `APP__AUTH__ENABLE_CAPTCHA_VERIFICATION` | `true` | 是否启用图片验证码 |

---

## 📚 API 文档

### 认证接口

#### 用户登录
```http
POST /api/login
Content-Type: application/json

{
  "username": "user123",        // 或 手机号 / 邮箱
  "password": "password123",
  "captcha_id": "uuid",
  "captcha": "ABC12"
}
```

#### 用户注册
```http
POST /api/register
Content-Type: application/json

{
  "username": "user123",
  "password": "password123",
  "captcha_id": "uuid",
  "captcha": "ABC12"
}
```

#### 选择租户（多租户场景）
```http
POST /api/select-tenant
Content-Type: application/json

{
  "temp_token": "temporary_token",
  "tenant_id": 123
}
```

#### 用户登出
```http
GET /api/logout
Authorization: Bearer <token>
```

#### 获取用户信息
```http
GET /api/user/profile
Authorization: Bearer <token>
```

### 验证码接口

#### 获取图片验证码
```http
GET /api/captcha
```
**响应**：返回 `captcha_id` + Base64 PNG 图片

#### 发送短信/邮箱验证码
```http
POST /api/send-verify-code
Content-Type: application/json

{
  "account": "13800138000"  // 或 "user@example.com"
}
```

---

## 🔄 与 Keycloak 功能对比

| 功能模块 | Keycloak | ms-auth (当前) | ms-auth (目标) |
|---------|----------|----------------|----------------|
| **单点登录 (SSO)** | ✅ OAuth2 / OIDC / SAML | ⚠️ Token 基础实现 | ✅ 完整 OIDC/OAuth2 |
| **多种登录** | ✅ 完整 | ✅ 用户名/手机/邮箱 | ✅ + 社交登录/WebAuthn |
| **多因素认证 (MFA)** | ✅ TOTP/SMS/Email | ❌ | ✅ TOTP/SMS/Email |
| **RBAC / ABAC** | ✅ 完整 | ⚠️ 基础 (依赖 ms-identity) | ✅ 完整 |
| **身份联邦** | ✅ LDAP/AD/社交 | ❌ | ✅ LDAP/AD/社交 |
| **管理控制台** | ✅ Web UI | ❌ 仅 API | ✅ Web 后台 |
| **Token 管理** | ✅ JWT/Refresh | ⚠️ Sa-Token | ✅ 标准 JWT |

---

## 📅 发展计划

### Phase 1 — 基础完善（0-3 个月）
- [ ] Token 刷新与撤销机制
- [ ] 密码策略配置 + 登录失败次数限制
- [ ] 用户 Profile / 密码修改 / 密码重置

### Phase 2 — 标准协议（3-6 个月）
- [ ] OAuth2 授权码模式 + 客户端凭证模式
- [ ] OpenID Connect + JWT Token
- [ ] 社交登录（GitHub/Google/微信）
- [ ] 多因素认证 (TOTP/SMS MFA)

### Phase 3 — 企业级（6-12 个月）
- [ ] RBAC / ABAC 策略引擎
- [ ] LDAP / Active Directory 集成
- [ ] Web 管理控制台
- [ ] 审计日志 + 会话管理增强

---

## 🔗 与其他服务的关系

| 服务 | 协议 | 关系说明 |
|------|------|----------|
| **ms-identity** | gRPC | 单向调用 — 用户认证/租户查询 |
| **ms-notify** | Kafka | 投递验证码发送请求 |
| **客户端** | HTTP | 登录/注册/Token 管理 |

---

## 📄 许可证

MIT OR Apache-2.0
