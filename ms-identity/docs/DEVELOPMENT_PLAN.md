# ms-identity 服务开发计划

## 一、项目概述

### 1.1 服务定位

**ms-identity** 是身份服务，整合了用户、租户、权限三大核心领域，提供统一的身份管理和权限控制能力。

### 1.2 核心职责

- **用户管理**：用户信息管理、密码验证、用户状态管理
- **租户管理**：租户信息管理、租户应用授权、租户状态管理
- **权限管理**：角色管理、资源管理、权限分配、权限检查

### 1.3 数据表范围

#### 用户模块

- `def_user` - 用户表
- `def_user_tenant_rel` - 用户租户关系表

#### 租户模块

- `def_tenant` - 租户表
- `def_tenant_application_rel` - 租户应用关系表

#### 权限模块

- `base_role` - 角色表
- `def_resource` - 资源表
- `base_role_resource_rel` - 角色资源关系表
- `casbin_rule` - Casbin 权限规则表
- `casbin_audit_log` - Casbin 审计日志表

### 1.4 当前进度

**✅ 已完成**：

- 项目基础结构搭建
- Entity 文件集成（已放入 `modules/{user,tenant,auth}/model/entity/` 目录）
- 模块导出结构完善
- Entity 编译错误修复（修复了 `CasbinRule` 缺少主键字段的问题）
- Repository 层开发（所有 Repository 已实现，使用 sqlxplus 0.2.1，支持事务）
- Service 层开发（所有 Service 已实现，使用 `anyhow::Result`，统一错误处理）
- DTO 层开发（所有请求/响应对象已定义）
- Handler 层开发（33/36 个端点已实现，3 个占位符待完善）
- 路由配置完成（36 个路由端点已配置）
- 错误处理重构（统一使用 `IdentityError` 枚举，使用 `thiserror`）
- 应用状态管理（`AppState` 包含所有 Service 实例）
- 开发规范文档（`DEVELOPMENT_STANDARDS.md`）

**🔄 进行中**：

- Handler 层完善（3 个占位符：列表查询、权限检查）

**⏳ 待开始**：

- JWT Token 认证
- Casbin 权限引擎集成
- 列表查询功能（分页）
- 内部服务接口
- Session 管理
- 事件发布
- 测试和优化

---

## 二、项目结构

```
ms-identity/
├── src/
│   ├── main.rs                 # 服务入口
│   ├── config.rs               # 配置管理
│   ├── error.rs                # 错误定义
│   ├── router.rs               # HTTP 路由
│   ├── modules/                # 业务模块（内部模块化）
│   │   ├── mod.rs
│   │   ├── user/               # 用户模块
│   │   │   ├── mod.rs
│   │   │   ├── service.rs      # 业务逻辑
│   │   │   ├── repository.rs  # 数据访问
│   │   │   └── model.rs        # 数据模型（使用生成的 entity）
│   │   ├── tenant/             # 租户模块
│   │   │   ├── mod.rs
│   │   │   ├── service.rs
│   │   │   ├── repository.rs
│   │   │   └── model/          # 数据模型
│   │   │       ├── mod.rs      # 导出 entity、vo、resp 等
│   │   │       ├── entity/     # 数据库实体（已生成）
│   │   │       │   ├── mod.rs
│   │   │       │   ├── tenant.rs
│   │   │       │   └── tenant_application_rel.rs
│   │   │       ├── vo/         # 视图对象（待创建）
│   │   │       └── resp/      # 响应对象（待创建）
│   │   └── auth/               # 权限模块
│   │       ├── mod.rs
│   │       ├── service.rs
│   │       ├── repository.rs
│   │       ├── model/          # 数据模型
│   │       │   ├── mod.rs      # 导出 entity、vo、resp 等
│   │       │   ├── entity/    # 数据库实体（已生成）
│   │       │   │   ├── mod.rs
│   │       │   │   ├── role.rs
│   │       │   │   ├── resource.rs
│   │       │   │   ├── role_resource_rel.rs
│   │       │   │   ├── application.rs
│   │       │   │   ├── casbin_rule.rs
│   │       │   │   └── casbin_audit_log.rs
│   │       │   ├── vo/        # 视图对象（待创建）
│   │       │   └── resp/      # 响应对象（待创建）
│   │       └── casbin.rs       # Casbin 权限引擎封装
│   ├── handlers/               # HTTP 处理器
│   │   ├── mod.rs
│   │   ├── user.rs             # 用户 API handlers
│   │   ├── tenant.rs           # 租户 API handlers
│   │   └── auth.rs             # 权限 API handlers
│   └── internal/               # 内部服务接口（供其他服务调用）
│       ├── mod.rs
│       ├── user.rs             # 用户内部接口
│       ├── tenant.rs           # 租户内部接口
│       └── auth.rs             # 权限内部接口
├── Cargo.toml
└── DEVELOPMENT_PLAN.md
```

---

## 三、开发阶段

### 阶段 1：项目基础搭建（1-2 天）

#### 1.1 项目结构创建 ✅

- [x] 创建目录结构
- [x] 创建 Cargo.toml
- [x] 创建基础模块文件
- [x] 配置工作空间

#### 1.2 配置管理

- [ ] 完善 `config.rs`（JWT、Session、密码加密配置）
- [ ] 创建 `.env.example` 配置文件示例
- [ ] 集成 fbc-starter 配置系统

#### 1.3 错误处理 ✅

- [x] 定义错误类型和错误码（`IdentityError` 枚举）
- [x] 使用 `thiserror` 定义结构化错误
- [x] 统一使用 `anyhow::Result<T>` 作为返回值
- [x] 实现错误转换（`From<sqlx::Error>`, `From<argon2::Error>`, `From<jsonwebtoken::errors::Error>`）
- [x] 实现错误码映射（`code()` 方法）
- [x] Handler 层统一错误响应处理
- [ ] 添加错误日志记录（可选优化）

#### 1.4 路由和处理器框架 ✅（90%）

- [x] 创建路由结构
- [x] 创建处理器框架
- [x] 实现所有 Handler（33/36 个端点）
- [x] DTO 层定义（所有请求/响应对象）
- [x] 应用状态管理（`AppState`）
- [ ] 添加请求验证中间件
- [ ] 添加认证中间件（Token 验证）

---

### 阶段 2：数据层开发（2-3 天）

#### 2.1 Entity 集成 ✅

- [x] 等待用户提供生成的 entity 文件
- [x] 将 entity 文件放入 `src/modules/{user,tenant,auth}/model/entity/` 目录
- [x] 创建 model 模块导出
- [x] 创建 entity/mod.rs 统一导出所有 entity
- [x] 修复 entity 模块编译问题

**当前结构**：

```
modules/
├── user/
│   └── model/
│       └── entity/
│           ├── mod.rs          # 导出 User, TenantUserRel
│           ├── user.rs
│           └── tenant_user_rel.rs
├── tenant/
│   └── model/
│       └── entity/
│           ├── mod.rs          # 导出 Tenant, TenantApplicationRel
│           ├── tenant.rs
│           └── tenant_application_rel.rs
└── auth/
    └── model/
        └── entity/
            ├── mod.rs          # 导出 Role, Resource, RoleResourceRel, Application, CasbinRule, CasbinAuditLog
            ├── role.rs
            ├── resource.rs
            ├── role_resource_rel.rs
            ├── application.rs
            ├── casbin_rule.rs
            └── casbin_audit_log.rs
```

**说明**：model 目录预留了 vo（视图对象）、resp（响应对象）等模型的位置，后续可以在此目录下创建相应模块。

#### 2.2 Repository 层实现 ✅

**用户模块 Repository**：

- [x] `UserRepository` - 用户数据访问
  - [x] `find_by_id(id: i64) -> Result<User>`
  - [x] `find_by_username(username: &str) -> Result<User>`
  - [x] `find_by_email(email: &str) -> Result<User>`
  - [x] `find_by_mobile(mobile: &str) -> Result<User>`
  - [x] `create(user: &User) -> Result<i64>`
  - [x] `update(user: &User) -> Result<()>`
  - [x] `delete(id: i64) -> Result<()>`
  - [x] `exists_by_username(username: &str) -> Result<bool>`
  - [x] `exists_by_email(email: &str) -> Result<bool>`
- [x] `UserTenantRelRepository` - 用户租户关系数据访问
  - [x] `find_by_user_id(user_id: i64) -> Result<Vec<TenantUserRel>>`
  - [x] `find_by_tenant_id(tenant_id: i64) -> Result<Vec<TenantUserRel>>`
  - [x] `find_by_user_and_tenant(user_id: i64, tenant_id: i64) -> Result<Option<TenantUserRel>>`
  - [x] `create(rel: &TenantUserRel) -> Result<i64>`
  - [x] `delete(user_id: i64, tenant_id: i64) -> Result<()>`
  - [x] `update(rel: &TenantUserRel) -> Result<()>`

**租户模块 Repository**：

- [x] `TenantRepository` - 租户数据访问
  - [x] `find_by_id(id: i64) -> Result<Tenant>`
  - [x] `find_by_name(name: &str) -> Result<Tenant>`
  - [x] `create(tenant: &Tenant) -> Result<i64>`
  - [x] `update(tenant: &Tenant) -> Result<()>`
  - [x] `delete(id: i64) -> Result<()>`
  - [x] `exists_by_name(name: &str) -> Result<bool>`
- [x] `TenantApplicationRelRepository` - 租户应用关系数据访问
  - [x] `find_by_tenant_id(tenant_id: i64) -> Result<Vec<TenantApplicationRel>>`
  - [x] `find_by_application_id(app_id: i64) -> Result<Vec<TenantApplicationRel>>`
  - [x] `find_by_tenant_and_application(tenant_id: i64, app_id: i64) -> Result<Option<TenantApplicationRel>>`
  - [x] `create(rel: &TenantApplicationRel) -> Result<i64>`
  - [x] `delete(tenant_id: i64, app_id: i64) -> Result<()>`
  - [x] `update(rel: &TenantApplicationRel) -> Result<()>`

**权限模块 Repository**：

- [x] `RoleRepository` - 角色数据访问
  - [x] `find_by_id(id: i64) -> Result<Role>`
  - [x] `find_by_code(code: &str, tenant_id: i64) -> Result<Role>`
  - [x] `find_by_tenant_id(tenant_id: i64) -> Result<Vec<Role>>`
  - [x] `create(role: &Role) -> Result<i64>`
  - [x] `update(role: &Role) -> Result<()>`
  - [x] `delete(id: i64) -> Result<()>`
- [x] `ResourceRepository` - 资源数据访问
  - [x] `find_by_id(id: i64) -> Result<Resource>`
  - [x] `find_by_application_id(app_id: i64) -> Result<Vec<Resource>>`
  - [x] `find_by_parent_id(parent_id: i64) -> Result<Vec<Resource>>`
  - [x] `create(resource: &Resource) -> Result<i64>`
  - [x] `update(resource: &Resource) -> Result<()>`
  - [x] `delete(id: i64) -> Result<()>`
- [x] `RoleResourceRelRepository` - 角色资源关系数据访问
  - [x] `find_by_role_id(role_id: i64) -> Result<Vec<RoleResourceRel>>`
  - [x] `find_by_resource_id(resource_id: i64) -> Result<Vec<RoleResourceRel>>`
  - [x] `find_by_role_and_resource(role_id: i64, resource_id: i64) -> Result<Option<RoleResourceRel>>`
  - [x] `create(rel: &RoleResourceRel) -> Result<i64>`
  - [x] `delete(role_id: i64, resource_id: i64) -> Result<()>`
  - [x] `update(rel: &RoleResourceRel) -> Result<()>`
- [x] `CasbinRuleRepository` - Casbin 规则数据访问
  - [x] `load_all() -> Result<Vec<CasbinRule>>`
  - [x] `add_rule(rule: &CasbinRule) -> Result<()>`
  - [x] `remove_rule(rule: &CasbinRule) -> Result<()>`
  - [x] `find_by_ptype(ptype: &str) -> Result<Vec<CasbinRule>>`
- [x] `ApplicationRepository` - 应用数据访问
  - [x] `find_by_id(id: i64) -> Result<Application>`
  - [x] `find_by_code(code: &str) -> Result<Application>`
  - [x] `create(app: &Application) -> Result<i64>`
  - [x] `update(app: &Application) -> Result<()>`
  - [x] `delete(id: i64) -> Result<()>`

**技术要点**：

- ✅ 升级到 `sqlxplus` 0.1.6，使用 `DbExecutor` trait 统一支持 `DbPool` 和 `Transaction`
- ✅ Repository 改为 unit struct（无字段），不再持有 `Arc<DbPool>`
- ✅ 所有 Repository 方法使用 `E: DbExecutor` 泛型参数，支持事务操作
- ✅ 使用 `sqlxplus::Crud` trait 的方法进行 CRUD 操作（`find_by_id`, `find_one`, `find_all`, `insert`, `update`, `delete_by_id`, `count`）
- ✅ 使用 `sqlxplus::QueryBuilder` 构建查询条件
- ✅ sqlxplus 自动处理软删除字段（`is_del`），无需手动添加条件
- ✅ sqlxplus 的 `find_one` 方法自动添加 `LIMIT 1`，无需手动指定
- ✅ `delete` 方法使用 `Entity::delete_by_id(executor, id)` 自动处理软删除
- ✅ 错误处理统一使用 `AppResult<T>`，移除 `IdentityResult<T>`
- ✅ 使用 `error_helpers` 模块的辅助函数创建错误（如 `error_helpers::user_not_found()`、`error_helpers::database_error()`）
- ✅ 业务错误使用 `AppError::BizError`，通用错误使用 `AppError::CommonError`
- ✅ 模块导出统一：`mod` 声明为私有，`pub use` 重新导出公共接口

---

### 阶段 3：业务逻辑层开发（3-4 天）✅

#### 3.1 用户模块 Service ✅

**UserService**：

- [x] `verify_password(username: &str, password: &str) -> Result<User>`
  - 验证用户名密码
  - 检查用户状态（是否禁用）
  - 更新密码错误次数
- [x] `get_user_info(user_id: i64) -> Result<User>`
  - 获取用户基本信息
- [x] `create_user(...) -> Result<i64>`
  - 密码加密
  - 创建用户记录
- [x] `update_user(...) -> Result<()>`
- [x] `delete_user(user_id: i64) -> Result<()>`
- [x] `change_password(user_id: i64, old_password: &str, new_password: &str) -> Result<()>`
- [x] `reset_password(user_id: i64, new_password: &str) -> Result<()>`

**UserTenantService**：

- [x] `add_user_to_tenant(user_id: i64, tenant_id: i64, is_default: bool) -> Result<()>`
- [x] `remove_user_from_tenant(user_id: i64, tenant_id: i64) -> Result<()>`
- [x] `set_default_tenant(user_id: i64, tenant_id: i64) -> Result<()>`
- [x] `get_user_tenants(user_id: i64) -> Result<Vec<TenantUserRel>>`

#### 3.2 租户模块 Service ✅

**TenantService**：

- [x] `get_tenant_info(tenant_id: i64) -> Result<Tenant>`
- [x] `get_tenant_by_name(name: &str) -> Result<Tenant>`
- [x] `create_tenant(...) -> Result<i64>`
- [x] `update_tenant(...) -> Result<()>`
- [x] `delete_tenant(tenant_id: i64, update_by: Option<i64>) -> Result<()>`

**TenantApplicationService**：

- [x] `add_application_to_tenant(tenant_id: i64, application_id: i64, ...) -> Result<i64>`
- [x] `remove_application_from_tenant(tenant_id: i64, application_id: i64) -> Result<()>`
- [x] `get_tenant_applications(tenant_id: i64) -> Result<Vec<TenantApplicationRel>>`
- [x] `get_application_tenants(application_id: i64) -> Result<Vec<TenantApplicationRel>>`

#### 3.3 权限模块 Service ✅

**RoleService**：

- [x] `get_role_info(role_id: i64) -> Result<Role>`
- [x] `get_role_by_code(code: &str, tenant_id: i64) -> Result<Role>`
- [x] `get_tenant_roles(tenant_id: i64) -> Result<Vec<Role>>`
- [x] `create_role(...) -> Result<i64>`
- [x] `update_role(...) -> Result<()>`
- [x] `delete_role(role_id: i64, update_by: Option<i64>) -> Result<()>`

**ResourceService**：

- [x] `get_resource_info(resource_id: i64) -> Result<Resource>`
- [x] `get_application_resources(app_id: i64) -> Result<Vec<Resource>>`
- [x] `get_child_resources(parent_id: i64) -> Result<Vec<Resource>>`
- [x] `create_resource(...) -> Result<i64>`
- [x] `update_resource(...) -> Result<()>`
- [x] `delete_resource(resource_id: i64, update_by: Option<i64>) -> Result<()>`

**PermissionService**：

- [x] `assign_resource_to_role(role_id: i64, resource_id: i64, create_by: Option<i64>) -> Result<i64>`
- [x] `remove_resource_from_role(role_id: i64, resource_id: i64) -> Result<()>`
- [x] `get_role_resources(role_id: i64) -> Result<Vec<RoleResourceRel>>`
- [x] `get_resource_roles(resource_id: i64) -> Result<Vec<RoleResourceRel>>`

**ApplicationService**：

- [x] `get_application_info(app_id: i64) -> Result<Application>`
- [x] `get_application_by_code(code: &str) -> Result<Application>`
- [x] `create_application(...) -> Result<i64>`
- [x] `update_application(...) -> Result<()>`
- [x] `delete_application(app_id: i64, update_by: Option<i64>) -> Result<()>`

**CasbinService**：

- [ ] `init_casbin_engine() -> Result<CasbinEngine>`
  - 从数据库加载规则
  - 初始化 Casbin 引擎
- [ ] `check_permission(subject: &str, object: &str, action: &str) -> Result<bool>`
  - 使用 Casbin 检查权限
- [ ] `add_permission_rule(rule: CasbinRule) -> Result<()>`
  - 添加权限规则到数据库和内存
- [ ] `remove_permission_rule(rule: CasbinRule) -> Result<()>`
  - 从数据库和内存移除规则
- [ ] `reload_rules() -> Result<()>`
  - 重新加载规则（权限变更后）

---

### 阶段 4：HTTP API 开发（2-3 天）✅（90%）

#### 4.1 用户 API ✅

**公开 API**：

- [x] `POST /api/v1/users/login` - 用户登录（Token 待实现）
- [x] `POST /api/v1/users` - 创建用户
- [x] `GET /api/v1/users/{id}` - 获取用户信息
- [x] `PUT /api/v1/users/{id}` - 更新用户信息
- [x] `DELETE /api/v1/users/{id}` - 删除用户
- [ ] `GET /api/v1/users` - 用户列表查询（分页）
- [x] `PUT /api/v1/users/{id}/password` - 修改密码
- [x] `PUT /api/v1/users/{id}/password/reset` - 重置密码
- [x] `GET /api/v1/users/{id}/tenants` - 获取用户租户列表
- [x] `POST /api/v1/users/{id}/tenants` - 添加用户到租户
- [x] `PUT /api/v1/users/{id}/tenants/default` - 设置默认租户
- [x] `DELETE /api/v1/users/{id}/tenants` - 从租户移除用户

**内部 API**（供 ms-oauth 调用）：

- [ ] `POST /api/internal/users/verify-password` - 验证密码
- [ ] `GET /api/internal/users/:id` - 获取用户信息
- [ ] `GET /api/internal/users/:id/tenants` - 获取用户租户列表

#### 4.2 租户 API ✅

**公开 API**：

- [x] `POST /api/v1/tenants` - 创建租户
- [x] `GET /api/v1/tenants/{id}` - 获取租户信息
- [x] `PUT /api/v1/tenants/{id}` - 更新租户信息
- [x] `DELETE /api/v1/tenants/{id}` - 删除租户
- [ ] `GET /api/v1/tenants` - 租户列表查询（分页）
- [x] `GET /api/v1/tenants/{id}/applications` - 获取租户应用列表
- [x] `POST /api/v1/tenants/{id}/applications` - 添加应用到租户
- [x] `DELETE /api/v1/tenants/{id}/applications` - 从租户移除应用

**内部 API**：

- [ ] `GET /api/internal/tenants/:id` - 获取租户信息
- [ ] `GET /api/internal/tenants/:id/status` - 检查租户状态

#### 4.3 权限 API ✅（90%）

**公开 API**：

- [x] `POST /api/v1/auth/roles` - 创建角色
- [x] `GET /api/v1/auth/roles/{id}` - 获取角色信息
- [x] `PUT /api/v1/auth/roles/{id}` - 更新角色
- [x] `DELETE /api/v1/auth/roles/{id}` - 删除角色
- [ ] `GET /api/v1/auth/roles` - 角色列表查询（分页，占位符）
- [x] `GET /api/v1/auth/roles/tenant/{tenant_id}` - 获取租户角色列表
- [x] `GET /api/v1/auth/roles/{id}/resources` - 获取角色资源列表
- [x] `POST /api/v1/auth/roles/{id}/resources` - 分配资源到角色
- [x] `DELETE /api/v1/auth/roles/{id}/resources` - 从角色移除资源
- [x] `POST /api/v1/auth/resources` - 创建资源
- [x] `GET /api/v1/auth/resources/{id}` - 获取资源信息
- [x] `PUT /api/v1/auth/resources/{id}` - 更新资源
- [x] `DELETE /api/v1/auth/resources/{id}` - 删除资源
- [ ] `GET /api/v1/auth/resources` - 资源列表查询（分页，占位符）
- [x] `GET /api/v1/auth/resources/application/{app_id}` - 获取应用资源列表
- [x] `POST /api/v1/auth/applications` - 创建应用
- [x] `GET /api/v1/auth/applications/{id}` - 获取应用信息
- [x] `PUT /api/v1/auth/applications/{id}` - 更新应用
- [x] `DELETE /api/v1/auth/applications/{id}` - 删除应用
- [ ] `POST /api/v1/auth/check-permission` - 检查权限（占位符，待 Casbin 集成）

**内部 API**（供其他服务调用）：

- [ ] `GET /api/internal/auth/users/:user_id/permissions` - 获取用户权限
- [ ] `POST /api/internal/auth/check-permission` - 权限检查
- [ ] `POST /api/internal/auth/reload-rules` - 重新加载权限规则

---

### 阶段 5：认证和 Session 管理（2-3 天）

#### 5.1 Token 管理

- [ ] JWT Token 生成
  - [ ] Access Token（短期，1 小时）
  - [ ] Refresh Token（长期，7 天）
- [ ] Token 验证
  - [ ] 签名验证
  - [ ] 过期检查
  - [ ] 黑名单检查（登出后）
- [ ] Token 刷新
  - [ ] Refresh Token 换 Access Token

#### 5.2 Session 管理

- [ ] Redis Session 存储
  - [ ] Session 创建
  - [ ] Session 读取
  - [ ] Session 更新
  - [ ] Session 删除
- [ ] Session 过期管理
  - [ ] 自动过期（Redis TTL）
  - [ ] 手动清除

#### 5.3 密码管理

- [ ] 密码加密
  - [ ] bcrypt 加密
  - [ ] argon2 加密（可选）
- [ ] 密码验证
- [ ] 密码强度检查
- [ ] 密码历史记录（可选）

---

### 阶段 6：Casbin 权限引擎集成（2-3 天）

#### 6.1 Casbin 初始化

- [ ] 数据库适配器开发
  - [ ] 实现 Casbin Adapter
  - [ ] 从数据库加载规则
  - [ ] 规则变更监听
- [ ] 权限模型配置
  - [ ] RBAC 模型配置
  - [ ] 支持多租户
  - [ ] 支持应用级别权限

#### 6.2 权限检查实现

- [ ] 用户权限检查
  - [ ] 从用户获取角色
  - [ ] 从角色获取权限
  - [ ] Casbin 规则检查
- [ ] 资源权限检查
  - [ ] 资源路径匹配
  - [ ] HTTP 方法匹配
- [ ] 权限缓存
  - [ ] Redis 缓存权限检查结果
  - [ ] 权限变更时清除缓存

#### 6.3 权限规则管理

- [ ] 规则 CRUD
  - [ ] 添加规则
  - [ ] 删除规则
  - [ ] 更新规则
  - [ ] 查询规则
- [ ] 规则批量操作
- [ ] 规则导入导出

---

### 阶段 7：内部服务接口（1-2 天）

#### 7.1 用户内部接口

- [ ] `verify_password` - 密码验证接口
- [ ] `get_user_info` - 获取用户信息接口
- [ ] `get_user_tenants` - 获取用户租户列表接口

#### 7.2 租户内部接口

- [ ] `get_tenant_info` - 获取租户信息接口
- [ ] `check_tenant_status` - 检查租户状态接口
- [ ] `check_tenant_app_authorized` - 检查租户应用授权接口

#### 7.3 权限内部接口

- [ ] `get_user_permissions` - 获取用户权限接口
- [ ] `check_permission` - 权限检查接口
- [ ] `reload_rules` - 重新加载规则接口

---

### 阶段 8：事件发布（1-2 天）

#### 8.1 权限变更事件

- [ ] 角色变更事件
  - [ ] 角色创建
  - [ ] 角色更新
  - [ ] 角色删除
- [ ] 资源变更事件
  - [ ] 资源创建
  - [ ] 资源更新
  - [ ] 资源删除
- [ ] 权限分配变更事件
  - [ ] 角色资源关联变更

#### 8.2 Kafka 事件发布

- [ ] 集成 fbc-starter Kafka Producer
- [ ] 定义事件格式
- [ ] 发布权限变更事件
- [ ] 其他服务订阅事件更新缓存

---

### 阶段 9：测试和优化（2-3 天）

#### 9.1 单元测试

- [ ] Repository 层测试
- [ ] Service 层测试
- [ ] Handler 层测试

#### 9.2 集成测试

- [ ] API 集成测试
- [ ] 权限检查测试
- [ ] 多租户隔离测试

#### 9.3 性能优化

- [ ] 数据库查询优化
- [ ] 权限检查缓存优化
- [ ] Casbin 规则加载优化

---

## 四、技术栈

### 4.1 核心依赖

- **Web 框架**：Axum（fbc-starter 提供）
- **数据库**：SQLx + sqlxplus（MySQL）
- **缓存**：Redis（fbc-starter 提供）
- **权限引擎**：Casbin
- **Token**：JWT（jsonwebtoken）
- **密码加密**：bcrypt / argon2

### 4.2 基础设施

- **配置管理**：fbc-starter Config
- **日志**：tracing
- **错误处理**：thiserror + fbc-starter AppError
- **消息队列**：Kafka（fbc-starter 提供，用于事件发布）

---

## 五、API 设计规范

### 5.1 RESTful API

- 使用标准 HTTP 方法（GET、POST、PUT、DELETE）
- 资源路径使用复数形式（`/api/v1/users`）
- 使用统一响应格式（`R<T>`）

### 5.2 内部 API

- 路径前缀：`/api/internal/`
- 用于服务间调用
- 不对外暴露（通过网关过滤）

### 5.3 认证方式

- 公开 API：需要 Token 认证
- 内部 API：使用服务间密钥认证（可选）

---

## 六、数据一致性

### 6.1 事务管理

- 用户和租户关系：同一事务
- 角色和资源关系：同一事务
- 权限规则变更：事务 + 事件发布

### 6.2 缓存一致性

- 权限变更时发布事件
- 其他服务订阅事件更新缓存
- Redis 缓存过期时间：5-10 分钟

---

## 七、安全考虑

### 7.1 密码安全

- 使用强加密算法（bcrypt/argon2）
- 密码不存储明文
- 密码强度验证

### 7.2 Token 安全

- Token 签名验证
- Token 过期检查
- Token 黑名单（登出后）

### 7.3 权限安全

- 多租户数据隔离
- 权限检查不信任客户端
- 审计日志记录

---

## 八、开发时间估算

| 阶段     | 任务                | 预计时间     |
| -------- | ------------------- | ------------ |
| 1        | 项目基础搭建        | 1-2 天       |
| 2        | 数据层开发          | 2-3 天       |
| 3        | 业务逻辑层开发      | 3-4 天       |
| 4        | HTTP API 开发       | 2-3 天       |
| 5        | 认证和 Session 管理 | 2-3 天       |
| 6        | Casbin 权限引擎集成 | 2-3 天       |
| 7        | 内部服务接口        | 1-2 天       |
| 8        | 事件发布            | 1-2 天       |
| 9        | 测试和优化          | 2-3 天       |
| **总计** |                     | **16-25 天** |

---

## 九、关键决策点

### 9.1 模块化设计

- 内部模块化：保持代码结构清晰
- 未来可拆分：如果业务需要，可以按模块拆分

### 9.2 权限模型

- 使用 Casbin RBAC 模型
- 支持多租户权限隔离
- 支持应用级别权限控制

### 9.3 缓存策略

- 权限检查结果缓存（5-10 分钟）
- 用户信息缓存（1-5 分钟）
- 权限变更时清除相关缓存

---

## 十、后续优化方向

1. **性能优化**

   - 权限检查结果缓存
   - 数据库查询优化
   - 批量权限检查接口

2. **功能扩展**

   - 支持更多认证方式（OAuth2、SAML）
   - 支持组织架构权限
   - 支持数据权限（行级权限）

3. **监控和运维**
   - 权限检查性能监控
   - 权限变更审计
   - 服务健康检查

---

## 十一、开发注意事项

1. **Entity 文件位置**：等待用户提供后，放入对应模块的 `model/` 目录
2. **数据库连接**：使用 fbc-starter 的数据库连接池
3. **错误处理**：统一使用 `AppResult<T>` 和 `AppError`，通过 `error_helpers` 模块创建错误
4. **日志记录**：关键操作记录日志（登录、权限变更等）
5. **代码规范**：遵循 Rust 代码规范，使用 clippy 检查

---

**文档版本**：v1.3

**更新记录**：

- v1.3: Repository 层重构，升级到 sqlxplus 0.1.6，Repository 改为 unit struct，支持事务，统一模块导出风格  
  **创建时间**：2024-12-22  
  **最后更新**：2024-12-22
- v1.2: 错误处理重构，统一使用 `AppResult`，移除 `IdentityResult`，创建 `error_helpers` 模块  
  **创建时间**：2024-12-22  
  **最后更新**：2024-12-22
- v1.1: 更新 Repository 层实现说明，添加 sqlxplus 0.1.4 使用说明  
  **创建时间**：2024-12-22  
  **最后更新**：2024-12-22
