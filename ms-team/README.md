# ms-team — 企业级组织管理服务

> 组织架构与团队管理核心模块，维护部门结构树、岗位体系、员工生命周期及数据权限控制。

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

### 权限与安全

- 🔐 **多租户隔离** — 基于租户 ID 的数据隔离
- 🛡️ **数据权限** — 结合 ms-identity 的 RBAC 权限控制
- 📝 **操作审计** — 关键操作审计日志

---

## 🛠️ 技术栈

| 类目 | 技术 | 说明 |
|------|------|------|
| **框架** | fbc-starter (Axum) | HTTP + gRPC 双协议 |
| **数据库** | MySQL 8.0 | sqlx + sqlxplus ORM |
| **缓存** | Redis | 组织树缓存等 |
| **内部通信** | gRPC (Tonic) | 提供组织查询服务 + 调用 ms-identity |
| **服务发现** | Nacos | 注册/发现/负载均衡 |

---

## 🏗 架构设计

```
┌──────────────────────────────────────────────┐
│                   ms-team                     │
│                                              │
│  ┌────────────┐  ┌────────────┐  ┌────────┐  │
│  │ HTTP API   │  │ gRPC Server│  │ gRPC   │  │
│  │ (RESTful)  │  │ (内部服务)  │  │ Client │  │
│  └─────┬──────┘  └─────┬──────┘  └───┬────┘  │
│        │               │             │        │
│        ▼               ▼             ▼        │
│  ┌──────────────────────────────────────────┐ │
│  │             Service Layer                │ │
│  │  ├ OrganizationService                   │ │
│  │  ├ DepartmentService                     │ │
│  │  ├ PositionService                       │ │
│  │  └ EmployeeService                      │ │
│  └────────────────┬─────────────────────────┘ │
│                   │                           │
│  ┌────────────────▼─────────────────────────┐ │
│  │             Repository Layer             │ │
│  │  ├ OrganizationRepository                │ │
│  │  ├ DepartmentRepository                  │ │
│  │  ├ PositionRepository                    │ │
│  │  └ EmployeeRepository                   │ │
│  └────────────────┬─────────────────────────┘ │
│                   ▼                           │
│             ┌──────────┐                      │
│             │  MySQL   │                      │
│             └──────────┘                      │
└──────────────────────────────────────────────┘
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
│   ├── router.rs            # HTTP 路由
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
│       └── employee/        # 员工模块
│           ├── handler.rs
│           ├── service.rs
│           ├── repository.rs
│           └── model/
├── docs/
│   ├── ENTERPRISE_DESIGN.md         # 企业级设计文档
│   ├── DEVELOPMENT_PLAN_DETAIL.md   # 开发计划
│   ├── API_SPECIFICATION.md         # RESTful API 规范
│   └── gRPC_INTERFACE_DEFINITION.md # gRPC 接口定义
├── Dockerfile               # Docker 构建
└── Cargo.toml
```

---

## 🚀 快速开始

### 前置条件

- Rust 1.80+
- MySQL 8.0+
- Redis 7.0+
- Nacos 2.0+

### 配置与运行

```bash
# 1. 构建
cargo build -p ms-team

# 2. 运行
cargo run -p ms-team
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

---

## 📚 API 文档

### RESTful API

服务提供以 `/api/v1` 为基础路径的 RESTful API，认证方式为 JWT Token。

#### 组织管理

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/api/v1/organizations` | 创建组织 |
| GET | `/api/v1/organizations/:id` | 获取组织详情 |
| PUT | `/api/v1/organizations/:id` | 更新组织 |
| DELETE | `/api/v1/organizations/:id` | 删除组织 |
| GET | `/api/v1/organizations/tree` | 获取组织树 |

#### 部门管理

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/api/v1/departments` | 创建部门 |
| GET | `/api/v1/departments/:id` | 获取部门详情 |
| PUT | `/api/v1/departments/:id` | 更新部门 |
| DELETE | `/api/v1/departments/:id` | 删除部门 |
| GET | `/api/v1/departments/tree` | 获取部门树 |

#### 岗位管理

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/api/v1/positions` | 创建岗位 |
| GET | `/api/v1/positions/:id` | 获取岗位详情 |
| PUT | `/api/v1/positions/:id` | 更新岗位 |
| DELETE | `/api/v1/positions/:id` | 删除岗位 |

#### 员工管理

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/api/v1/employees` | 添加员工 |
| GET | `/api/v1/employees/:id` | 获取员工详情 |
| PUT | `/api/v1/employees/:id` | 更新员工信息 |
| DELETE | `/api/v1/employees/:id` | 删除员工 |
| GET | `/api/v1/employees/list` | 员工列表 |

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