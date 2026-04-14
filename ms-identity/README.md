# ms-identity

身份认证与授权微服务，提供用户管理、租户管理、权限管理和 JWT Token 认证功能。

## 📋 目录

- [功能特性](#功能特性)
- [技术栈](#技术栈)
- [快速开始](#快速开始)
- [配置说明](#配置说明)
- [API 文档](#api-文档)
- [项目结构](#项目结构)
- [开发指南](#开发指南)
- [开发进度](#开发进度)

## ✨ 功能特性

### 核心功能

- ✅ **用户管理**

  - 用户注册、登录、登出
  - 用户信息 CRUD
  - 密码修改和重置
  - 多租户支持（用户可属于多个租户）

- ✅ **租户管理**

  - 租户 CRUD
  - 租户应用关联管理
  - 多租户隔离

- ✅ **权限管理**

  - 角色管理（Role）
  - 资源管理（Resource）
  - 应用管理（Application）
  - 角色资源关联
  - 权限检查（待 Casbin 集成）

- ✅ **JWT Token 认证**
  - Access Token（15 分钟有效期）
  - Refresh Token（7 天有效期）
  - Token 刷新机制
  - 认证中间件
  - 短期 Token 策略（无需黑名单）

### 技术特性

- 🏗️ **模块化架构**：用户、租户、权限三大模块
- 🔒 **安全认证**：JWT Token + 密码加密（Argon2）
- 📊 **分页查询**：支持游标分页（CursorPageBaseResp）
- 🎯 **统一错误处理**：结构化错误定义和处理
- 📝 **RESTful API**：37 个 API 端点
- 🚀 **高性能**：基于 Axum 异步框架

## 🛠 技术栈

### 核心依赖

- **Rust** - 系统编程语言
- **Axum** - 异步 Web 框架
- **sqlxplus** - 数据库 ORM 和查询构建器
- **fbc-starter** - 应用启动框架（集成 Nacos、MySQL、Kafka、Redis）
- **jsonwebtoken** - JWT Token 处理
- **argon2** - 密码加密
- **casbin** - 权限控制引擎（待集成）

### 其他依赖

- **anyhow** - 错误处理（Service/Repository 层）
- **thiserror** - 结构化错误定义
- **serde** - 序列化/反序列化
- **chrono** - 时间处理
- **deadpool-redis** - Redis 连接池

## 🚀 快速开始

### 前置要求

- Rust 1.70+
- MySQL 8.0+
- Redis（可选，用于缓存）
- Nacos（用于配置中心）

### 安装

```bash
# 克隆项目
git clone <repository-url>
cd hula-server/ms-identity

# 编译项目
cargo build --release
```

### 配置

创建配置文件或通过环境变量配置：

```bash
# JWT 配置
export APP__IDENTITY__JWT__SECRET="your-secret-key-change-in-production"
export APP__IDENTITY__JWT__ACCESS_TOKEN_EXPIRE=900  # 15分钟
export APP__IDENTITY__JWT__REFRESH_TOKEN_EXPIRE=604800  # 7天

# 密码配置
export APP__IDENTITY__PASSWORD__MIN_LENGTH=8

# Session 配置
export APP__IDENTITY__SESSION__EXPIRE=86400  # 24小时
```

### 运行

```bash
# 开发模式
cargo run

# 生产模式
cargo run --release
```

## ⚙️ 配置说明

### JWT Token 配置

| 配置项                 | 环境变量                                   | 默认值                                 | 说明                         |
| ---------------------- | ------------------------------------------ | -------------------------------------- | ---------------------------- |
| `secret`               | `APP__IDENTITY__JWT__SECRET`               | `your-secret-key-change-in-production` | JWT 签名密钥                 |
| `access_token_expire`  | `APP__IDENTITY__JWT__ACCESS_TOKEN_EXPIRE`  | `900` (15 分钟)                        | Access Token 过期时间（秒）  |
| `refresh_token_expire` | `APP__IDENTITY__JWT__REFRESH_TOKEN_EXPIRE` | `604800` (7 天)                        | Refresh Token 过期时间（秒） |

### 密码配置

| 配置项       | 环境变量                              | 默认值 | 说明         |
| ------------ | ------------------------------------- | ------ | ------------ |
| `min_length` | `APP__IDENTITY__PASSWORD__MIN_LENGTH` | `8`    | 密码最小长度 |

### Session 配置

| 配置项   | 环境变量                         | 默认值            | 说明                   |
| -------- | -------------------------------- | ----------------- | ---------------------- |
| `expire` | `APP__IDENTITY__SESSION__EXPIRE` | `86400` (24 小时) | Session 过期时间（秒） |

## 📚 API 文档

### 认证相关

#### 用户登录

```http
POST /api/v1/users/login
Content-Type: application/json

{
  "username": "admin",
  "password": "password123"
}
```

**响应：**

```json
{
  "code": 0,
  "data": {
    "user": {
      "id": 1,
      "username": "admin",
      "email": "admin@example.com",
      "nick_name": "管理员"
    },
    "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
    "refresh_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
    "access_token_expire": 900,
    "refresh_token_expire": 604800
  }
}
```

#### 刷新 Token

```http
POST /api/v1/users/refresh-token
Content-Type: application/json

{
  "refresh_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
}
```

#### 用户登出

```http
POST /api/v1/users/logout
Authorization: Bearer <access_token>
```

### 用户管理

| 方法   | 路径                                 | 说明             | 认证 |
| ------ | ------------------------------------ | ---------------- | ---- |
| POST   | `/api/v1/users`                      | 创建用户         | ✅   |
| GET    | `/api/v1/users/{id}`                 | 获取用户信息     | ✅   |
| PUT    | `/api/v1/users/{id}`                 | 更新用户信息     | ✅   |
| DELETE | `/api/v1/users/{id}`                 | 删除用户         | ✅   |
| PUT    | `/api/v1/users/{id}/password`        | 修改密码         | ✅   |
| PUT    | `/api/v1/users/{id}/password/reset`  | 重置密码         | ✅   |
| GET    | `/api/v1/users/{id}/tenants`         | 获取用户租户列表 | ✅   |
| POST   | `/api/v1/users/{id}/tenants`         | 添加用户到租户   | ✅   |
| PUT    | `/api/v1/users/{id}/tenants/default` | 设置默认租户     | ✅   |
| DELETE | `/api/v1/users/{id}/tenants`         | 从租户移除用户   | ✅   |

### 租户管理

| 方法   | 路径                                | 说明             | 认证 |
| ------ | ----------------------------------- | ---------------- | ---- |
| POST   | `/api/v1/tenants`                   | 创建租户         | ✅   |
| GET    | `/api/v1/tenants/{id}`              | 获取租户信息     | ✅   |
| PUT    | `/api/v1/tenants/{id}`              | 更新租户信息     | ✅   |
| DELETE | `/api/v1/tenants/{id}`              | 删除租户         | ✅   |
| GET    | `/api/v1/tenants/{id}/applications` | 获取租户应用列表 | ✅   |
| POST   | `/api/v1/tenants/{id}/applications` | 添加应用到租户   | ✅   |
| DELETE | `/api/v1/tenants/{id}/applications` | 从租户移除应用   | ✅   |

### 权限管理

#### 角色管理

| 方法   | 路径                                    | 说明                 | 认证 |
| ------ | --------------------------------------- | -------------------- | ---- |
| GET    | `/api/v1/auth/roles`                    | 获取角色列表（分页） | ✅   |
| POST   | `/api/v1/auth/roles`                    | 创建角色             | ✅   |
| GET    | `/api/v1/auth/roles/{id}`               | 获取角色信息         | ✅   |
| PUT    | `/api/v1/auth/roles/{id}`               | 更新角色             | ✅   |
| DELETE | `/api/v1/auth/roles/{id}`               | 删除角色             | ✅   |
| GET    | `/api/v1/auth/roles/tenant/{tenant_id}` | 获取租户角色列表     | ✅   |
| GET    | `/api/v1/auth/roles/{id}/resources`     | 获取角色资源列表     | ✅   |
| POST   | `/api/v1/auth/roles/{id}/resources`     | 分配资源到角色       | ✅   |
| DELETE | `/api/v1/auth/roles/{id}/resources`     | 从角色移除资源       | ✅   |

#### 资源管理

| 方法   | 路径                                          | 说明                 | 认证 |
| ------ | --------------------------------------------- | -------------------- | ---- |
| GET    | `/api/v1/auth/resources`                      | 获取资源列表（分页） | ✅   |
| POST   | `/api/v1/auth/resources`                      | 创建资源             | ✅   |
| GET    | `/api/v1/auth/resources/{id}`                 | 获取资源信息         | ✅   |
| PUT    | `/api/v1/auth/resources/{id}`                 | 更新资源             | ✅   |
| DELETE | `/api/v1/auth/resources/{id}`                 | 删除资源             | ✅   |
| GET    | `/api/v1/auth/resources/application/{app_id}` | 获取应用资源列表     | ✅   |

#### 应用管理

| 方法   | 路径                             | 说明         | 认证 |
| ------ | -------------------------------- | ------------ | ---- |
| POST   | `/api/v1/auth/applications`      | 创建应用     | ✅   |
| GET    | `/api/v1/auth/applications/{id}` | 获取应用信息 | ✅   |
| PUT    | `/api/v1/auth/applications/{id}` | 更新应用     | ✅   |
| DELETE | `/api/v1/auth/applications/{id}` | 删除应用     | ✅   |

#### 权限检查

| 方法 | 路径                            | 说明     | 认证 |
| ---- | ------------------------------- | -------- | ---- |
| POST | `/api/v1/auth/check-permission` | 检查权限 | ✅   |

## 📁 项目结构

```
ms-identity/
├── src/
│   ├── main.rs                 # 应用入口
│   ├── config.rs              # 配置定义
│   ├── error.rs                # 错误定义
│   ├── state.rs                # 应用状态
│   ├── context.rs              # 请求上下文
│   ├── router.rs               # 路由配置
│   ├── jwt/                    # JWT Token 模块
│   │   ├── mod.rs
│   │   └── service.rs          # JWT 服务
│   ├── middleware/             # 中间件
│   │   ├── mod.rs
│   │   └── auth.rs             # 认证中间件
│   └── modules/                # 业务模块
│       ├── user/               # 用户模块
│       │   ├── handler.rs      # HTTP 处理器
│       │   ├── service.rs      # 业务逻辑
│       │   ├── repository.rs   # 数据访问
│       │   └── model/
│       │       ├── dto.rs      # 数据传输对象
│       │       └── entity/     # 实体定义
│       ├── tenant/             # 租户模块
│       └── auth/               # 权限模块
├── Cargo.toml                  # 依赖配置
├── README.md                    # 项目文档
├── PROGRESS_REPORT.md          # 开发进度
├── DEVELOPMENT_PLAN.md         # 开发计划
└── NEXT_STEPS.md               # 下一步计划
```

## 🛠 开发指南

### 开发规范

项目遵循统一的开发规范，详见 [`DEVELOPMENT_PLAN.md`](./DEVELOPMENT_PLAN.md) 和 [`PROGRESS_REPORT.md`](./PROGRESS_REPORT.md)。

### 核心设计原则

1. **模块化设计**：按业务领域划分模块（user、tenant、auth）
2. **分层架构**：Handler → Service → Repository → Entity
3. **统一错误处理**：使用 `IdentityError` 枚举统一错误类型
4. **DTO 模式**：请求/响应对象定义在模块的 `model/dto.rs`
5. **依赖注入**：通过 `AppState` 管理所有服务实例

### 添加新功能

1. **定义实体**：在 `modules/{module}/model/entity/` 下定义实体
2. **实现 Repository**：在 `modules/{module}/repository.rs` 实现数据访问
3. **实现 Service**：在 `modules/{module}/service.rs` 实现业务逻辑
4. **定义 DTO**：在 `modules/{module}/model/dto.rs` 定义请求/响应对象
5. **实现 Handler**：在 `modules/{module}/handler.rs` 实现 HTTP 处理器
6. **配置路由**：在 `router.rs` 中添加路由

### 运行测试

```bash
# 运行所有测试
cargo test

# 运行特定模块的测试
cargo test --package ms-identity --lib modules::user
```

## 📊 开发进度

**总体完成度：约 85%**

### 已完成 ✅

- ✅ 基础架构（100%）
- ✅ 数据模型层（100%）
- ✅ Repository 层（100%）
- ✅ Service 层（100%）
- ✅ DTO 层（100%）
- ✅ Handler 层（97%，37/38 个端点）
- ✅ 路由配置（100%）
- ✅ 错误处理（100%）
- ✅ JWT 认证（95%）

### 进行中 🔄

- ⚠️ Casbin 权限引擎集成（0%）

### 待开始 ⏳

- ⏳ 列表查询功能（用户列表、租户列表）
- ⏳ 内部服务接口（gRPC）
- ⏳ Session 管理
- ⏳ 事件发布（Kafka）
- ⏳ 单元测试和集成测试

详细进度请查看 [`PROGRESS_REPORT.md`](./PROGRESS_REPORT.md)。

## 🔐 安全说明

### Token 策略

- **Access Token**：15 分钟有效期，短期有效，自然过期
- **Refresh Token**：7 天有效期，用于刷新 Access Token
- **无黑名单机制**：采用短期 Token 策略，无需维护黑名单

### 密码安全

- 使用 **Argon2** 算法加密密码
- 支持密码最小长度配置
- 密码错误次数限制（待实现）

### 认证流程

1. 用户登录 → 生成 Access Token + Refresh Token
2. 客户端使用 Access Token 访问受保护资源
3. Access Token 过期后，使用 Refresh Token 刷新
4. Refresh Token 过期后，需要重新登录

## 📝 许可证

[许可证信息]

## 👥 贡献者

[贡献者列表]

## 📮 联系方式

[联系方式]

---

**最后更新**：2025-01-02
