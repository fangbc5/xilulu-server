# ms-auth - 身份认证与授权服务

> 一个基于 Rust 构建的高性能身份认证与授权服务，致力于对标 Keycloak，提供企业级的身份访问管理（IAM）解决方案。

## 📋 目录

- [项目简介](#项目简介)
- [核心功能对比 Keycloak](#核心功能对比-keycloak)
- [当前功能特性](#当前功能特性)
- [架构设计](#架构设计)
- [改进与优化建议](#改进与优化建议)
- [发展计划（Roadmap）](#发展计划roadmap)
- [快速开始](#快速开始)
- [API 文档](#api-文档)
- [贡献指南](#贡献指南)

## 项目简介

`ms-auth` 是一个现代化的身份认证与授权微服务，采用 Rust 语言开发，基于 Axum 框架构建。项目旨在提供高性能、高可用、易扩展的身份访问管理解决方案，对标 Keycloak 的功能特性，同时保持 Rust 生态系统的性能优势和类型安全。

### 核心优势

- 🚀 **高性能**：基于 Rust 零成本抽象，O(1) 时间复杂度操作
- 🔒 **安全性**：多重验证机制，支持图片验证码、短信/邮箱验证码
- 🏢 **多租户支持**：原生支持多租户架构，灵活的租户选择机制
- 📦 **微服务架构**：与 ms-identity 服务解耦，支持分布式部署
- 🎯 **易扩展**：模块化设计，支持插件化扩展
- 💾 **Redis 缓存**：基于 Redis 的 Token 存储和会话管理

## 核心功能对比 Keycloak

| 功能模块 | Keycloak | ms-auth (当前) | ms-auth (目标) |
|---------|----------|----------------|----------------|
| **单点登录 (SSO)** | ✅ OAuth2 / OpenID Connect / SAML | ⚠️ Token 基础实现 | ✅ 完整 OIDC/OAuth2 支持 |
| **用户注册** | ✅ 完整注册流程 | ✅ 基础注册（用户名/手机/邮箱） | ✅ 增强注册（邮箱验证、社交登录） |
| **用户登录** | ✅ 多种登录方式 | ✅ 用户名/手机/邮箱 + 密码/验证码 | ✅ 社交登录、WebAuthn |
| **多因素认证 (MFA)** | ✅ TOTP、SMS、Email | ❌ 未实现 | ✅ TOTP、SMS、Email |
| **角色与权限** | ✅ RBAC、ABAC、细粒度权限 | ⚠️ 基础角色（依赖 ms-identity） | ✅ 完整 RBAC/ABAC 支持 |
| **身份联邦** | ✅ LDAP、AD、社交登录 | ❌ 未实现 | ✅ LDAP、AD、OAuth2 社交登录 |
| **管理控制台** | ✅ Web UI 管理后台 | ❌ 仅 API | ✅ Web 管理后台 |
| **会话管理** | ✅ 会话列表、撤销、超时 | ⚠️ 基础会话（Redis） | ✅ 完整会话管理 |
| **密码策略** | ✅ 可配置密码规则 | ⚠️ 基础验证 | ✅ 可配置密码策略 |
| **审计日志** | ✅ 完整审计日志 | ⚠️ 基础日志 | ✅ 完整审计日志 |
| **多租户** | ✅ Realm 隔离 | ✅ 租户隔离 | ✅ 增强多租户支持 |
| **Token 管理** | ✅ JWT、Refresh Token | ⚠️ Sa-Token（非标准 JWT） | ✅ 标准 JWT 支持 |
| **API 网关集成** | ✅ 多种集成方式 | ⚠️ 基础集成 | ✅ 完整集成方案 |

**图例说明：**
- ✅ 已实现
- ⚠️ 部分实现
- ❌ 未实现

## 当前功能特性

### 1. 用户认证

#### 登录功能
- ✅ 多种登录方式支持
  - 用户名 + 密码 + 图片验证码
  - 手机号 + 验证码
  - 邮箱 + 验证码
- ✅ 多租户登录流程
  - 单租户：直接返回访问令牌
  - 多租户：返回临时令牌和租户列表，用户选择后换取正式令牌
- ✅ 验证码校验
  - 图片验证码（防机器人）
  - 短信/邮箱验证码（防暴力破解）

#### 注册功能
- ✅ 多种注册方式
  - 用户名 + 密码 + 图片验证码
  - 手机号 + 验证码
  - 邮箱 + 验证码
- ✅ 自动昵称生成
  - 支持纯中文、纯英文、中英文混合
  - 低重复率（基于时间戳哈希）
  - 可配置数字后缀

### 2. 验证码服务

#### 图片验证码
- ✅ 5 位字符验证码生成
- ✅ Base64 PNG 图片输出
- ✅ IP 频率限制（1 分钟 1 次）
- ✅ Redis 存储（5 分钟有效期）
- ✅ 一次性使用（验证后删除）

#### 短信/邮箱验证码
- ✅ 6 位数字验证码
- ✅ 账号频率限制（1 分钟 1 次）
- ✅ Redis 存储（5 分钟有效期）
- ✅ Kafka 异步发送通知
- ✅ 一次性使用（验证后删除）

### 3. Token 管理

- ✅ 基于 Sa-Token 的 Token 生成
- ✅ Redis 存储 Token 会话
- ✅ Token 过期时间管理（24 小时）
- ✅ 临时 Token 机制（多租户选择）
- ⚠️ Token 刷新（待实现）
- ⚠️ Token 撤销（待实现）

### 4. 用户信息

- ✅ 用户 Profile 查询
- ✅ 用户登出
- ⚠️ Profile 修改（待实现）
- ⚠️ 密码修改（待实现）

### 5. 多租户支持

- ✅ 租户列表查询
- ✅ 租户选择流程
- ✅ 租户隔离（基于 ms-identity）
- ⚠️ 租户管理（待实现）

## 架构设计

### 技术栈

```
ms-auth
├── Web 框架: Axum (Rust)
├── Token 管理: Sa-Token (Redis 存储)
├── 缓存: Redis (Token、验证码、会话)
├── 消息队列: Kafka (验证码通知)
├── 服务发现: Nacos (服务注册与发现)
├── gRPC 客户端: Tonic (调用 ms-identity)
└── 日志: Tracing
```

### 模块结构

```
ms-auth/
├── src/
│   ├── handler/          # HTTP 处理器
│   │   ├── auth.rs      # 认证相关（登录、注册、登出）
│   │   └── code.rs      # 验证码相关
│   ├── service/         # 业务服务层
│   │   ├── image_captcha.rs      # 图片验证码服务
│   │   ├── verify_code.rs        # 短信/邮箱验证码服务
│   │   ├── temp_token.rs         # 临时 Token 服务
│   │   ├── validation.rs         # 公共校验模块
│   │   └── nickname_generator.rs # 昵称生成器
│   ├── client/          # 外部服务客户端
│   │   └── identity.rs  # ms-identity gRPC 客户端
│   ├── model/           # 数据模型
│   ├── error.rs         # 错误定义
│   └── router.rs        # 路由配置
```

### 数据流

```
用户请求
  ↓
HTTP Handler (auth.rs / code.rs)
  ↓
Service Layer (验证码、Token、校验)
  ↓
Redis (缓存层)
  ↓
gRPC Client → ms-identity (用户信息、租户)
  ↓
Kafka (异步通知)
```

## 改进与优化建议

### 🔴 高优先级（P0）

#### 1. 标准协议支持
- **OAuth2 / OpenID Connect**
  - 实现授权码模式（Authorization Code Flow）
  - 实现客户端凭证模式（Client Credentials）
  - 实现 Token Refresh 流程
  - 支持 JWT Token（当前使用 Sa-Token，非标准）
  - 实现 `.well-known/openid-configuration` 端点

#### 2. 安全性增强
- **密码策略**
  - 可配置密码复杂度规则
  - 密码历史记录（防止重复使用）
  - 密码过期策略
- **登录安全**
  - 登录失败次数限制（账户锁定）
  - IP 黑名单/白名单
  - 异常登录检测（异地登录提醒）
- **Token 安全**
  - Token 撤销机制
  - Token 刷新机制
  - Token 绑定 IP/设备

#### 3. 多因素认证 (MFA)
- TOTP（基于时间的一次性密码）
- SMS 验证码作为 MFA
- Email 验证码作为 MFA
- WebAuthn / FIDO2（未来）

### 🟡 中优先级（P1）

#### 4. 角色与权限系统
- **RBAC（基于角色的访问控制）**
  - 角色定义与管理
  - 角色分配
  - 权限继承
- **ABAC（基于属性的访问控制）**
  - 策略引擎
  - 动态权限计算
- **细粒度权限**
  - 资源级权限控制
  - 操作级权限控制

#### 5. 身份联邦
- **社交登录**
  - GitHub OAuth2
  - Google OAuth2
  - 微信登录
  - 支付宝登录
- **企业身份提供商**
  - LDAP 集成
  - Active Directory 集成
  - SAML 2.0 支持

#### 6. 用户管理功能
- **用户 Profile**
  - Profile 修改 API
  - 头像上传
  - 个人信息管理
- **账户管理**
  - 密码修改
  - 密码重置（邮箱/手机验证）
  - 账户注销
  - 账户锁定/解锁

#### 7. 会话管理
- **会话列表查询**
  - 当前活跃会话
  - 历史会话记录
- **会话控制**
  - 强制下线
  - 会话超时配置
  - 并发登录限制

### 🟢 低优先级（P2）

#### 8. 管理控制台
- **Web UI 开发**
  - 用户管理界面
  - 角色权限管理界面
  - 应用客户端管理
  - 审计日志查看
  - 系统配置管理

#### 9. 审计与监控
- **审计日志**
  - 登录日志
  - 操作日志
  - 安全事件日志
- **监控指标**
  - 登录成功率
  - Token 使用统计
  - 验证码发送统计
  - 性能指标（响应时间、QPS）

#### 10. 性能优化
- **数据库优化**
  - 连接池调优
  - 查询优化
  - 批量操作支持
- **缓存优化**
  - 多级缓存策略
  - 缓存预热
  - 缓存失效策略优化
- **分布式支持**
  - 多节点部署
  - 负载均衡
  - 会话共享

## 发展计划（Roadmap）

### 🎯 阶段 1：基础完善（0-3 个月）

**目标**：完善核心功能，提升安全性

#### 功能清单
- [ ] 完成注册功能增强（邮箱验证、手机验证）
- [ ] 实现密码策略配置
- [ ] 实现登录失败次数限制
- [ ] 实现 Token 刷新机制
- [ ] 实现 Token 撤销机制
- [ ] 完善公共校验模块（密码强度、格式校验）
- [ ] 优化昵称生成器（降低重复率）
- [ ] 添加用户 Profile 修改 API
- [ ] 添加密码修改 API
- [ ] 添加密码重置 API

#### 技术债务
- [ ] 代码重构（提取公共逻辑）
- [ ] 单元测试覆盖率提升到 80%
- [ ] 集成测试完善
- [ ] API 文档完善（OpenAPI/Swagger）

### 🚀 阶段 2：标准协议支持（3-6 个月）

**目标**：实现 OAuth2 / OpenID Connect 标准协议

#### 功能清单
- [ ] OAuth2 授权码模式实现
- [ ] OAuth2 客户端凭证模式实现
- [ ] OpenID Connect 基础实现
- [ ] JWT Token 支持（替换 Sa-Token）
- [ ] Token 刷新流程
- [ ] `.well-known/openid-configuration` 端点
- [ ] 社交登录（GitHub、Google）
- [ ] 多因素认证（TOTP、SMS MFA）

#### 集成与测试
- [ ] OAuth2 客户端测试
- [ ] OIDC 兼容性测试
- [ ] 性能测试与优化

### 🏢 阶段 3：企业级特性（6-12 个月）

**目标**：实现企业级 IAM 功能

#### 功能清单
- [ ] 完整 RBAC 权限系统
- [ ] ABAC 策略引擎
- [ ] LDAP / AD 集成
- [ ] SAML 2.0 支持
- [ ] Web 管理控制台（前端）
- [ ] 审计日志系统
- [ ] 会话管理增强
- [ ] 多租户增强（Realm 隔离）

#### 运维与监控
- [ ] 监控指标完善
- [ ] 告警系统
- [ ] 日志聚合与分析

### 🌟 阶段 4：对标 Keycloak（12 个月以上）

**目标**：全面对标 Keycloak，提供企业级 IAM 解决方案

#### 功能清单
- [ ] 可定制认证流程（SPI / Plugin）
- [ ] 主题与 UI 定制
- [ ] WebAuthn / FIDO2 支持
- [ ] 高级安全特性
  - 设备指纹识别
  - 地理位置验证
  - 行为分析
- [ ] 国际化 / 本地化
- [ ] 大规模性能优化
- [ ] 安全性审计

#### 生态建设
- [ ] SDK 开发（多语言）
- [ ] 插件市场
- [ ] 社区建设
- [ ] 文档与教程

## 快速开始

### 环境要求

- Rust 1.70+
- Redis 6.0+
- Kafka 2.8+（可选，用于验证码通知）
- Nacos（可选，用于服务发现）

### 安装与运行

```bash
# 克隆项目
git clone <repository-url>
cd hula-server/ms-auth

# 配置环境变量
cp .env.example .env
# 编辑 .env 文件，配置 Redis、Kafka 等连接信息

# 编译运行
cargo run --release
```

### 配置示例

```toml
# config.toml
[redis]
url = "redis://localhost:6379"

[kafka]
brokers = ["localhost:9092"]

[nacos]
server_addr = "localhost:8848"
namespace = "public"
```

## API 文档

### 认证接口

#### 1. 用户登录
```http
POST /api/login
Content-Type: application/json

{
  "username": "user123",
  "password": "password123",
  "captcha_id": "uuid",
  "captcha": "ABC12"
}
```

#### 2. 用户注册
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

#### 3. 选择租户
```http
POST /api/select-tenant
Content-Type: application/json

{
  "temp_token": "temporary_token",
  "tenant_id": 123
}
```

#### 4. 用户登出
```http
GET /api/logout
Authorization: Bearer <token>
```

#### 5. 获取用户信息
```http
GET /api/user/profile
Authorization: Bearer <token>
```

### 验证码接口

#### 1. 获取图片验证码
```http
GET /api/captcha
```

#### 2. 发送验证码（短信/邮箱）
```http
POST /api/send-verify-code
Content-Type: application/json

{
  "account": "13800138000"  // 或 "user@example.com"
}
```

## 贡献指南

我们欢迎所有形式的贡献！请查看 [CONTRIBUTING.md](CONTRIBUTING.md) 了解详细信息。

### 开发流程

1. Fork 项目
2. 创建功能分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 开启 Pull Request

### 代码规范

- 遵循 Rust 官方代码风格
- 使用 `cargo fmt` 格式化代码
- 使用 `cargo clippy` 检查代码
- 编写单元测试和集成测试

## 许可证

本项目采用 MIT 或 Apache-2.0 许可证。

## 联系方式

- 项目主页：<repository-url>
- 问题反馈：<issues-url>
- 讨论区：<discussions-url>

---

**注意**：本项目正在积极开发中，API 可能会发生变化。建议在生产环境使用前仔细测试。
