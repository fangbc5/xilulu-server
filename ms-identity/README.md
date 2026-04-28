# ms-identity — 身份中心服务

> 统一的数字身份与账户中心，提供用户管理、租户管理、权限管理（RBAC）及 JWT Token 认证，为其他微服务提供 gRPC 内部服务接口。

## 📋 目录

- [功能特性](#-功能特性)
- [技术栈](#️-技术栈)
- [架构设计](#-架构设计)
- [项目结构](#-项目结构)
- [快速开始](#-快速开始)
- [配置说明](#️-配置说明)
- [API 文档](#-api-文档)
- [安全说明](#-安全说明)
- [开发进度](#-开发进度)
- [与其他服务的关系](#-与其他服务的关系)

---

## ✨ 功能特性

### 用户管理

- 👤 **用户 CRUD** — 创建/查询/更新/删除用户
- 🔐 **密码安全** — Argon2 加密 + 密码修改/重置
- 🏢 **多租户绑定** — 用户可属于多个租户 + 默认租户设置
- 📊 **角色分配** — 用户角色绑定与权限关联

### 租户管理

- 🏠 **租户 CRUD** — 完整的租户生命周期管理
- 📱 **应用管理** — 租户与应用的关联管理
- 🔒 **数据隔离** — 基于租户的数据访问隔离

### 权限体系（RBAC）

- 🎭 **角色管理** — 角色 CRUD + 租户级角色
- 📋 **资源管理** — 资源 CRUD + 应用级资源
- 🔗 **角色-资源关联** — 灵活的权限分配
- ✅ **权限检查** — 统一的权限验证端点
- 🏗️ **应用管理** — 多应用权限隔离

### 认证与安全

- 🎫 **JWT Token** — Access Token (15min) + Refresh Token (7 天)
- ♻️ **Token 刷新** — 无缝刷新机制
- 🛡️ **短期策略** — Access Token 短期有效，无需黑名单

### gRPC 内部服务

- 📡 **用户查询** — 为 ms-auth、ms-im 等提供用户信息查询
- 🔍 **租户查询** — 为 ms-auth 提供租户列表查询
- ✅ **权限校验** — 为网关和其他服务提供权限检查

---

## 🛠️ 技术栈

| 类目 | 技术 | 说明 |
|------|------|------|
| **框架** | fbc-starter (Axum) | HTTP + gRPC 双协议 |
| **数据库** | MySQL 8.0 | sqlx + sqlxplus ORM |
| **认证** | jsonwebtoken | JWT Token 签发与验证 |
| **密码加密** | Argon2 | 高安全度密码哈希 |
| **gRPC** | Tonic | 内部服务接口 |
| **服务发现** | Nacos | 注册/配置中心 |

---

## 🏗 架构设计

```
                    ┌─────────────────────────────┐
                    │        ms-identity          │
                    │                             │
 HTTP API ────────► │  ┌───────────────────────┐  │ ◄──── gRPC (内部服务)
                    │  │    Handler Layer       │  │
                    │  │  ├ UserHandler         │  │   ┌──────────┐
                    │  │  ├ TenantHandler       │  │   │ ms-auth  │
                    │  │  └ AuthHandler         │  │   │ ms-im    │
                    │  └───────────┬────────────┘  │   │ ms-team  │
                    │              ▼               │   └──────────┘
                    │  ┌───────────────────────┐  │
                    │  │    Service Layer       │  │
                    │  │  ├ UserService         │  │
                    │  │  ├ TenantService       │  │
                    │  │  └ AuthService         │  │
                    │  └───────────┬────────────┘  │
                    │              ▼               │
                    │  ┌───────────────────────┐  │
                    │  │  Repository Layer      │  │
                    │  │  ├ UserRepository      │  │
                    │  │  ├ TenantRepository    │  │
                    │  │  └ AuthRepository      │  │
                    │  └───────────┬────────────┘  │
                    │              ▼               │
                    │         ┌──────────┐        │
                    │         │  MySQL   │        │
                    │         └──────────┘        │
                    └─────────────────────────────┘
```

---

## 📁 项目结构

```
ms-identity/
├── src/
│   ├── main.rs              # 入口 — Server::run 启动
│   ├── config.rs            # IdentityConfig — JWT/密码/Session 配置
│   ├── error.rs             # IdentityError — 结构化错误枚举
│   ├── state.rs             # AppState — 聚合所有 Service
│   ├── context.rs           # 请求上下文（当前用户/租户）
│   ├── router.rs            # HTTP 路由配置 (37+ 端点)
│   ├── jwt/                 # 🔑 JWT 模块
│   │   └── service.rs       # JWT 签发/验证/刷新
│   ├── middleware/          # 🛡️ 中间件
│   │   └── auth.rs          # 认证中间件（Token 校验）
│   ├── grpc/                # 📡 gRPC 服务端
│   │   └── mod.rs           # 用户/租户查询 gRPC 服务
│   └── modules/             # 🧩 业务模块
│       ├── user/            # 用户模块 (10 个 API)
│       │   ├── handler.rs
│       │   ├── service.rs
│       │   ├── repository.rs
│       │   └── model/
│       │       ├── dto.rs   # 请求/响应 DTO
│       │       └── entity/  # 用户实体
│       ├── tenant/          # 租户模块 (7 个 API)
│       │   ├── handler.rs
│       │   ├── service.rs
│       │   ├── repository.rs
│       │   └── model/
│       └── auth/            # 权限模块 (17 个 API)
│           ├── handler.rs
│           ├── service.rs
│           ├── repository.rs
│           └── model/
├── Dockerfile               # Docker 构建
└── Cargo.toml
```

---

## 🚀 快速开始

### 前置条件

- Rust 1.80+
- MySQL 8.0+
- Nacos 2.0+

### 配置与运行

```bash
# 1. 配置环境变量
export APP__IDENTITY__JWT__SECRET="your-secret-key-change-in-production"
export APP__IDENTITY__JWT__ACCESS_TOKEN_EXPIRE=900     # 15 分钟
export APP__IDENTITY__JWT__REFRESH_TOKEN_EXPIRE=604800 # 7 天
export APP__IDENTITY__PASSWORD__MIN_LENGTH=8

# 2. 运行
cargo run -p ms-identity
```

---

## ⚙️ 配置说明

### 服务配置

| 变量 | 默认值 | 说明 |
|------|--------|------|
| `APP__SERVER__PORT` | `30001` | 服务端口 |
| `APP__LOG__LEVEL` | `info` | 日志级别 |
| `APP__DATABASE__URL` | — | MySQL 连接串 |
| `APP__NACOS__SERVER_ADDRS` | — | Nacos 地址 |
| `APP__NACOS__SERVICE_NAME` | `ms-identity` | 注册服务名 |

### JWT 配置

| 变量 | 默认值 | 说明 |
|------|--------|------|
| `APP__IDENTITY__JWT__SECRET` | — | JWT 签名密钥（**必须修改**） |
| `APP__IDENTITY__JWT__ACCESS_TOKEN_EXPIRE` | `900` (15 分钟) | Access Token 过期时间(秒) |
| `APP__IDENTITY__JWT__REFRESH_TOKEN_EXPIRE` | `604800` (7 天) | Refresh Token 过期时间(秒) |

### 密码配置

| 变量 | 默认值 | 说明 |
|------|--------|------|
| `APP__IDENTITY__PASSWORD__MIN_LENGTH` | `8` | 密码最小长度 |

---

## 📚 API 文档

### 认证相关 (3 个)

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/api/v1/users/login` | 用户登录 |
| POST | `/api/v1/users/refresh-token` | 刷新 Token |
| POST | `/api/v1/users/logout` | 用户登出 |

### 用户管理 (10 个)

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/api/v1/users` | 创建用户 |
| GET | `/api/v1/users/{id}` | 获取用户信息 |
| PUT | `/api/v1/users/{id}` | 更新用户信息 |
| DELETE | `/api/v1/users/{id}` | 删除用户 |
| PUT | `/api/v1/users/{id}/password` | 修改密码 |
| PUT | `/api/v1/users/{id}/password/reset` | 重置密码 |
| GET | `/api/v1/users/{id}/tenants` | 获取用户租户列表 |
| POST | `/api/v1/users/{id}/tenants` | 添加用户到租户 |
| PUT | `/api/v1/users/{id}/tenants/default` | 设置默认租户 |
| DELETE | `/api/v1/users/{id}/tenants` | 从租户移除用户 |

### 租户管理 (7 个)

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/api/v1/tenants` | 创建租户 |
| GET | `/api/v1/tenants/{id}` | 获取租户信息 |
| PUT | `/api/v1/tenants/{id}` | 更新租户信息 |
| DELETE | `/api/v1/tenants/{id}` | 删除租户 |
| GET | `/api/v1/tenants/{id}/applications` | 获取租户应用列表 |
| POST | `/api/v1/tenants/{id}/applications` | 添加应用到租户 |
| DELETE | `/api/v1/tenants/{id}/applications` | 从租户移除应用 |

### 权限管理 — 角色 (9 个)

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/api/v1/auth/roles` | 角色列表（分页） |
| POST | `/api/v1/auth/roles` | 创建角色 |
| GET | `/api/v1/auth/roles/{id}` | 获取角色详情 |
| PUT | `/api/v1/auth/roles/{id}` | 更新角色 |
| DELETE | `/api/v1/auth/roles/{id}` | 删除角色 |
| GET | `/api/v1/auth/roles/tenant/{tenant_id}` | 获取租户角色列表 |
| GET | `/api/v1/auth/roles/{id}/resources` | 获取角色资源列表 |
| POST | `/api/v1/auth/roles/{id}/resources` | 分配资源到角色 |
| DELETE | `/api/v1/auth/roles/{id}/resources` | 从角色移除资源 |

### 权限管理 — 资源 (6 个)

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/api/v1/auth/resources` | 资源列表（分页） |
| POST | `/api/v1/auth/resources` | 创建资源 |
| GET | `/api/v1/auth/resources/{id}` | 获取资源详情 |
| PUT | `/api/v1/auth/resources/{id}` | 更新资源 |
| DELETE | `/api/v1/auth/resources/{id}` | 删除资源 |
| GET | `/api/v1/auth/resources/application/{app_id}` | 获取应用资源列表 |

### 权限管理 — 应用 (4 个) + 权限检查 (1 个)

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/api/v1/auth/applications` | 创建应用 |
| GET | `/api/v1/auth/applications/{id}` | 获取应用详情 |
| PUT | `/api/v1/auth/applications/{id}` | 更新应用 |
| DELETE | `/api/v1/auth/applications/{id}` | 删除应用 |
| POST | `/api/v1/auth/check-permission` | 权限检查 |

> 共 **37+ 个 API 端点**，所有接口均需 JWT 认证。

---

## 🔐 安全说明

### Token 策略

```
登录成功 → 签发 Access Token (15 min) + Refresh Token (7 天)
        → 客户端使用 Access Token 访问受保护资源
        → Access Token 过期 → 使用 Refresh Token 刷新
        → Refresh Token 过期 → 重新登录
```

- **短期 Token 策略** — Access Token 仅 15 分钟有效，减少泄露风险
- **无黑名单机制** — Token 短期自然过期，无需维护令牌黑名单
- **Argon2 密码加密** — 业界推荐的密码哈希算法

---

## 📊 开发进度

**总体完成度：约 85%**

| 模块 | 进度 | 说明 |
|------|------|------|
| ✅ 基础架构 | 100% | 配置/路由/错误处理/中间件 |
| ✅ 用户模块 | 100% | CRUD + 密码 + 租户关联 |
| ✅ 租户模块 | 100% | CRUD + 应用关联 |
| ✅ 权限模块 | 100% | 角色/资源/应用 CRUD |
| ✅ JWT 认证 | 95% | 签发/验证/刷新 |
| ⚠️ Casbin 集成 | 0% | 权限引擎集成 |
| ⏳ gRPC 内部接口 | 部分完成 | 用户/租户查询 |
| ⏳ 单元测试 | 待开始 | 测试覆盖 |

---

## 🔗 与其他服务的关系

| 服务 | 协议 | 关系说明 |
|------|------|----------|
| **ms-auth** | gRPC | **被调用** — 提供用户认证、租户查询 |
| **ms-im** | gRPC | **被调用** — 提供用户信息查询 |
| **ms-team** | gRPC | **被调用** — 提供用户权限查询 |
| **ms-websocket** | gRPC | **被调用** — Token 验证 |

> ms-identity 是整个系统的**身份数据源头**，被其他所有需要用户信息的服务所依赖。

---

## 📄 许可证

MIT OR Apache-2.0
