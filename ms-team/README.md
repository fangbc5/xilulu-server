# ms-team — 企业级组织管理服务

> 组织架构与团队管理核心模块，维护部门结构树、岗位体系、员工生命周期、通讯录浏览与搜索以及数据权限控制。

## 📋 目录

- [功能特性](#-功能特性)
- [技术栈](#️-技术栈)
- [架构设计](#-架构设计)
- [项目结构](#-项目结构)
- [快速开始](#-快速开始)
- [配置说明](#️-配置说明)
- [API 文档](#-api-文档)
- [与其他服务的关系](#-与其他服务的关系)

---

## ✨ 功能特性

### 组织管理

- 🏢 **组织 CRUD** — 创建、编辑、删除组织
- 🌲 **组织树** — 多级组织架构树形展示
- 🏷️ **组织类型** — 灵活的组织类型分类

### 部门管理

- 📂 **部门 CRUD** — 完整的部门管理
- 🌿 **部门树** — 多级部门嵌套树结构
- 👤 **负责人设置** — 部门负责人指定
- 📊 **员工统计** — 部门人数实时统计

### 岗位管理

- 🎯 **岗位 CRUD** — 创建、编辑、删除岗位
- 📋 **岗位分类** — 按职能分类管理
- 📈 **岗位层级** — 多层级岗位体系

### 员工管理

- 👥 **员工信息管理** — CRUD + 状态管理
- 🔗 **部门关联** — 员工与部门、岗位的多对多关系
- 📊 **状态管理** — 在职/离职/试用期等状态流转
- 🔍 **权限范围查询** — 基于数据权限的员工可见性控制

### 通讯录

- 📖 **通讯录入口** — 组织信息 + 根部门列表 + 总人数
- 🗂️ **部门展开** — 子部门 + 直属成员预览（负责人置顶）
- 👤 **联系人详情** — 全部门/全岗位关系一览
- 🔍 **全局搜索** — Meilisearch 全文搜索 + MySQL 降级
- 📄 **成员分页** — 支持跨子部门的成员分页浏览
- 📖 **OpenAPI 文档** — 内置 Utoipa + Swagger UI 可交互式接口文档

### 权限与安全

- 🔐 **多租户隔离** — 基于租户 ID 的数据隔离
- 🛡️ **数据权限** — 结合 ms-identity 的 RBAC 权限控制
- 📝 **操作审计** — 关键操作审计日志

---

## 🛠️ 技术栈

| 类目 | 技术 | 说明 |
|------|------|------|
| **框架** | fbc-starter (Axum) | HTTP + gRPC 双协议 + OpenAPI |
| **数据库** | MySQL 8.0 | sqlx + sqlxplus ORM |
| **缓存** | Redis | 组织树缓存等 |
| **搜索** | Meilisearch | 通讯录全文搜索 |
| **内部通信** | gRPC (Tonic) | 提供组织查询服务 + 调用 ms-identity |
| **服务发现** | Nacos | 注册/发现/负载均衡 |
| **API 文档** | Utoipa + Swagger UI | 自动生成 OpenAPI 3.0 文档 |

---

## 🏗 架构设计

```
┌──────────────────────────────────────────────────────┐
│                     ms-team                           │
│                                                      │
│  ┌────────────┐  ┌────────────┐  ┌────────────────┐  │
│  │ HTTP API   │  │ gRPC Server│  │ Swagger UI     │  │
│  │ (RESTful)  │  │ (内部服务)  │  │ (OpenAPI 文档) │  │
│  └─────┬──────┘  └─────┬──────┘  └───────┬────────┘  │
│        │               │                 │            │
│        ▼               ▼                 ▼            │
│  ┌──────────────────────────────────────────────────┐ │
│  │                Service Layer                     │ │
│  │  ├ OrganizationService                           │ │
│  │  ├ DepartmentService                             │ │
│  │  ├ PositionService                               │ │
│  │  ├ EmployeeService                               │ │
│  │  └ ContactsService  ◄── Meilisearch + Permission │ │
│  └────────────────┬─────────────────────────────────┘ │
│                   │                                   │
│  ┌────────────────▼─────────────────────────────────┐ │
│  │                Repository Layer                  │ │
│  └────────────────┬─────────────────────────────────┘ │
│                   ▼                                   │
│        ┌──────────┐   ┌─────────────┐                 │
│        │  MySQL   │   │ Meilisearch │                 │
│        └──────────┘   └─────────────┘                 │
└──────────────────────────────────────────────────────┘
         │                        ▲
         │ gRPC                   │ gRPC
         ▼                        │
   ┌──────────┐             ┌──────────┐
   │ms-identity│            │ ms-im     │
   │(用户/权限) │            │(组织人员查询)│
   └──────────┘             └──────────┘
```

---

## 📁 项目结构

```
ms-team/
├── src/
│   ├── main.rs              # 入口 — Server::run 启动
│   ├── config.rs            # 业务配置
│   ├── error.rs             # 错误定义
│   ├── state.rs             # 应用状态
│   ├── router.rs            # HTTP 路由（含 OpenAPI 文档）
│   ├── middleware/          # 中间件
│   │   └── auth.rs          # 认证/权限中间件
│   ├── client/              # 外部服务客户端
│   │   └── identity.rs      # ms-identity gRPC 客户端
│   ├── grpc/                # gRPC 服务端
│   │   └── mod.rs           # 组织查询 gRPC 服务
│   └── modules/             # 🧩 业务模块
│       ├── organization/    # 组织模块
│       │   ├── handler.rs
│       │   ├── service.rs
│       │   ├── repository.rs
│       │   └── model/
│       ├── department/      # 部门模块
│       │   ├── handler.rs
│       │   ├── service.rs
│       │   ├── repository.rs
│       │   └── model/
│       ├── position/        # 岗位模块
│       │   ├── handler.rs
│       │   ├── service.rs
│       │   ├── repository.rs
│       │   └── model/
│       ├── employee/        # 员工模块
│       │   ├── handler.rs
│       │   ├── service.rs
│       │   ├── repository.rs
│       │   └── model/
│       └── contacts/        # 📖 通讯录模块
│           ├── handler.rs          # 通讯录 6 个接口
│           ├── service.rs          # 聚合查询 + 搜索降级
│           ├── model/dto.rs        # 通讯录专用 DTO
│           ├── permission/         # 权限引擎（可插拔）
│           │   ├── port.rs         # trait 定义
│           │   └── default.rs      # 全开放实现
│           └── search/             # 搜索引擎（可插拔）
│               ├── port.rs         # trait 定义
│               └── adapter.rs      # Meilisearch 适配器
├── docs/
│   ├── contacts-design.md          # 通讯录架构设计文档
│   ├── ENTERPRISE_DESIGN.md        # 企业级设计文档
│   ├── DEVELOPMENT_PLAN_DETAIL.md  # 开发计划
│   ├── API_SPECIFICATION.md        # RESTful API 规范
│   └── gRPC_INTERFACE_DEFINITION.md # gRPC 接口定义
├── Dockerfile               # Docker 构建（含 Swagger UI）
└── Cargo.toml
```

---

## 🚀 快速开始

### 前置条件

- Rust 1.80+
- MySQL 8.0+
- Redis 7.0+
- Nacos 2.0+
- Meilisearch（通讯录搜索，可选）

### 配置与运行

```bash
# 1. 构建
cargo build -p ms-team

# 2. 运行
cargo run -p ms-team
```

服务启动后访问 Swagger UI：

```
http://localhost:30101/swagger-ui
```

---

## ⚙️ 配置说明

| 变量 | 默认值 | 说明 |
|------|--------|------|
| `APP__SERVER__PORT` | `30101` | 服务端口 |
| `APP__LOG__LEVEL` | `info` | 日志级别 |
| `APP__DATABASE__URL` | — | MySQL 连接串 |
| `APP__REDIS__URL` | — | Redis 连接串 |
| `APP__NACOS__SERVER_ADDRS` | — | Nacos 地址 |
| `APP__NACOS__SERVICE_NAME` | `ms-team` | 注册服务名 |
| `APP__CONTACTS__MEILISEARCH_URL` | — | Meilisearch 地址 |
| `APP__CONTACTS__MEILISEARCH_API_KEY` | — | Meilisearch API Key |

---

## 📚 API 文档

> 完整的 API 文档可通过 Swagger UI 在线浏览，以下为核心接口概览。

### RESTful API

服务提供以 `/api/v1/team` 为基础路径的 RESTful API，认证方式为 JWT Token。

#### 组织管理

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/api/v1/team/organizations` | 创建组织 |
| GET | `/api/v1/team/organizations/{id}` | 获取组织详情 |
| PUT | `/api/v1/team/organizations/{id}` | 更新组织 |
| DELETE | `/api/v1/team/organizations/{id}` | 删除组织 |
| GET | `/api/v1/team/organizations` | 组织列表 |
| GET | `/api/v1/team/organizations/tree` | 获取组织树 |

#### 部门管理

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/api/v1/team/departments` | 创建部门 |
| GET | `/api/v1/team/departments/{id}` | 获取部门详情 |
| PUT | `/api/v1/team/departments/{id}` | 更新部门 |
| DELETE | `/api/v1/team/departments/{id}` | 删除部门 |
| GET | `/api/v1/team/departments` | 部门列表 |
| GET | `/api/v1/team/departments/tree` | 获取部门树 |
| GET | `/api/v1/team/departments/roots` | 根部门列表 |
| GET | `/api/v1/team/departments/{id}/children` | 子部门列表 |

#### 岗位管理

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/api/v1/team/positions` | 创建岗位 |
| GET | `/api/v1/team/positions/{id}` | 获取岗位详情 |
| PUT | `/api/v1/team/positions/{id}` | 更新岗位 |
| DELETE | `/api/v1/team/positions/{id}` | 删除岗位 |
| GET | `/api/v1/team/positions` | 岗位列表 |

#### 员工管理

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/api/v1/team/employees` | 创建员工 |
| GET | `/api/v1/team/employees/{id}` | 获取员工详情 |
| PUT | `/api/v1/team/employees/{id}` | 更新员工 |
| DELETE | `/api/v1/team/employees/{id}` | 删除员工 |
| GET | `/api/v1/team/employees` | 员工列表 |
| POST | `/api/v1/team/employees/{id}/departments` | 添加员工到部门 |
| DELETE | `/api/v1/team/employees/{id}/departments/{dept_id}` | 从部门移除员工 |
| POST | `/api/v1/team/employees/{id}/positions` | 添加员工岗位 |
| DELETE | `/api/v1/team/employees/{id}/positions/{pos_id}` | 移除员工岗位 |

#### 通讯录

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/api/v1/team/contacts/entry` | 通讯录入口 |
| GET | `/api/v1/team/contacts/departments/{dept_id}` | 部门展开 |
| GET | `/api/v1/team/contacts/employees/{id}` | 联系人详情 |
| GET | `/api/v1/team/contacts/search` | 全局搜索 |
| GET | `/api/v1/team/contacts/departments/{dept_id}/members` | 部门成员分页 |

#### 管理接口

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/api/v1/team/admin/search/rebuild` | 重建搜索索引 |

### gRPC 接口

- **服务名**: `OrganizationService`
- **数据格式**: Protobuf

详细接口定义见 [gRPC 接口文档](docs/gRPC_INTERFACE_DEFINITION.md)。

---

## 🔗 与其他服务的关系

| 服务 | 协议 | 关系说明 |
|------|------|----------|
| **ms-identity** | gRPC | 双向 — 查询用户权限 / 接收权限校验 |
| **ms-im** | gRPC | 被调用 — 提供组织人员信息查询 |
| **ms-notify** | Kafka | 投递组织变更通知 |

---

## 📄 许可证

MIT OR Apache-2.0