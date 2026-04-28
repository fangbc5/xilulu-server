# Xilulu Server

> 基于 Rust 语言构建的高性能分层微服务后端系统，提供即时通讯、身份认证、团队管理、内容中台、对象存储、媒体处理、消息通知及 AI 代理等全栈企业级能力。

## 📋 目录

- [项目简介](#项目简介)
- [核心技术栈](#️-核心技术栈)
- [微服务架构](#-微服务架构与模块划分)
- [基础设施依赖](#-基础设施依赖)
- [快速开始](#-快速开始)
- [部署指南](#-部署指南)
- [开发规范](#-开发规范)
- [项目结构](#-项目结构)

---

## 项目简介

Xilulu Server 是一套完整的微服务后端工程，底层基于 `Tokio` 异步运行时与 `Axum` Web 框架，依靠自研的 [`fbc-starter`](fbc-starter/README.md) 基础核心组件提供统一的微服务治理与底层支撑。工程采用 **Cargo Virtual Workspace**（虚拟工作区）模式管理，所有微服务共享依赖版本与构建配置。

### 核心设计理念

- 🏗️ **分层架构** — 严格遵循 `Handler → Service → Repository` 单向调用链
- 🔒 **类型安全** — 利用 Rust 类型系统保障数据流安全，`R<T>` 统一 HTTP 响应封装
- ⚡ **高性能** — Tokio 异步运行时 + 零成本抽象，单服务支撑万级并发
- 🧩 **模块解耦** — 各微服务独立部署、独立数据库，通过 gRPC / Kafka 协同

---

## 🛠️ 核心技术栈

| 类目 | 技术选型 |
|------|----------|
| **编程语言** | Rust 1.80+ (Edition 2021) |
| **异步运行时** | Tokio (Full Features) |
| **Web 框架** | Axum 0.8 + Utoipa OpenAPI 自动文档 |
| **数据库** | MySQL 8.0 — sqlx + [sqlxplus](https://github.com/fangbc5/sqlx-plus) (自研 ORM 宏增强) |
| **缓存** | Redis 7 — deadpool-redis 连接池 |
| **消息队列** | Kafka (rdkafka) — 异步事件驱动 |
| **服务治理** | [fbc-starter](https://github.com/fangbc5/fbc-starter) (自研) — 依赖注入、配置、服务注册 |
| **配置中心** | Nacos — 服务注册/发现/配置下发 |
| **内部通信** | gRPC (Tonic) — 强类型内部调用 |
| **实时通信** | WebSocket — 高频双向长连接 |
| **对象存储** | RustFS / MinIO (S3 兼容) |
| **图像处理** | Imgproxy — 动态图像处理代理 |
| **搜索引擎** | Meilisearch — 毫秒级全文检索 |
| **任务调度** | XXL-JOB / Ratchjob — 分布式定时任务 |
| **认证框架** | Sa-Token (Rust 移植版) — Token 管理 |
| **容器化** | Docker + fbc-builder + scratch 三阶构建 |

---

## 🧩 微服务架构与模块划分

### 架构全景

```
┌─────────────┐     ┌──────────────┐     ┌──────────────┐
│  Dioxus App │────▶│  ms-auth     │────▶│ ms-identity  │
│  (客户端)    │     │  (认证鉴权)   │     │ (身份中心)    │
└─────┬───────┘     └──────────────┘     └──────────────┘
      │                                         │
      │  WebSocket                         gRPC │
      ▼                                         ▼
┌─────────────┐     ┌──────────────┐     ┌──────────────┐
│ms-websocket │◀───▶│  ms-im       │────▶│  ms-team     │
│ (WS 网关)   │     │  (即时通讯)   │     │  (组织团队)   │
└─────────────┘     └──────┬───────┘     └──────────────┘
                           │ Kafka
      ┌────────────────────┼────────────────────┐
      ▼                    ▼                    ▼
┌──────────────┐   ┌──────────────┐    ┌──────────────┐
│  ms-notify   │   │  ms-oss      │───▶│ms-media-proc │
│  (消息通知)   │   │ (对象存储)    │    │ (媒体处理)    │
└──────────────┘   └──────────────┘    └──────────────┘
                                              │
              ┌──────────────┐    ┌────────────┘
              │  ms-content  │    │
              │  (内容中台)   │    ▼
              └──────────────┘   FFmpeg / S3
```

### 服务端口映射

| 模块 | 端口 | 描述 | 状态 |
|:-----|:-----|:-----|:-----|
| [**ms-identity**](ms-identity/README.md) | `30001` | 身份中心 — 用户/租户/权限/RBAC 管理、gRPC 内部服务 | ✅ 可用 |
| [**ms-auth**](ms-auth/README.md) | `30002` | 认证鉴权 — 登录/注册/验证码/多租户 Token 管理 | ✅ 可用 |
| [**ms-oss**](ms-oss/README.md) | `30003` | 对象存储 — 预签名上传/下载、文件元数据、动态图像处理 | ✅ 可用 |
| [**ms-team**](ms-team/README.md) | `30101` | 组织团队 — 部门/岗位/员工/组织架构树 | ✅ 可用 |
| [**ms-im**](ms-im/README.md) | `30102` | 即时通讯 — 私聊/群聊/好友/消息持久化与漫游 | ✅ 可用 |
| [**ms-notify**](ms-notify/README.md) | `30104` | 消息通知 — APNs/FCM 推送、短信、邮件、飞书/钉钉/企业微信 | ✅ 可用 |
| [**ms-media-processor**](ms-media-processor/README.md) | `30105` | 媒体处理 — 视频截图/转码、图片缩放/水印、音频提取 | ✅ 可用 |
| [**ms-content**](ms-content/README.md) | `30106` | 内容中台 — Block DSL 正文、内容关系图谱、Meilisearch 全文检索 | ✅ 可用 |
| [**ms-websocket**](ms-websocket/README.md) | `30201` | WebSocket 网关 — 实时消息推送、心跳保活、音视频信令 | ✅ 可用 |
| [**ms-ai**](ms-ai/README.md) | 规划中 | AI 代理 — LLM 对接、智能体编排 | 🔜 开发中 |
| [**ms-identity-admin**](ms-identity-admin/README.md) | `5174` | 管理后台 (Vue 3 前端) — 运营管理仪表盘 | ✅ 可用 |

### 基础框架层

| 模块 | 描述 |
|:-----|:-----|
| [**fbc-starter**](fbc-starter/README.md) | 微服务底层基建 — 依赖注入、`R<T>` 响应封装、`AppError` 错误处理、Nacos 注册/配置、数据库/Redis 管理 |
| [**fbc-standards**](fbc-standards/README.md) | 开发规范分发工具 — 一键初始化 `.aiproject/` 规范体系（支持 Cursor/Copilot/Gemini/Claude 等） |
| [**sqlxplus**](https://github.com/fangbc5/sqlx-plus) | 自研 ORM 宏增强 — 基于 sqlx 的实体派生宏、查询构建器、自动时间戳管理 |

---

## 🏢 基础设施依赖

系统运行依赖以下中间件基础设施：

| 中间件 | 用途 | 版本要求 |
|--------|------|----------|
| **MySQL** | 业务数据持久化 | 8.0+ |
| **Redis** | 缓存/Token 存储/会话管理 | 7.0+ |
| **Nacos** | 服务注册/发现/配置中心 | 2.0+ |
| **Kafka** | 异步消息队列/事件总线 | 3.0+ |
| **RustFS / MinIO** | S3 兼容对象存储 | 最新版 |
| **Meilisearch** | 全文搜索引擎 (ms-content) | 最新版 |
| **FFmpeg** | 视频/音频处理 (ms-media-processor) | 6.0+ |
| **Imgproxy** | 动态图像处理 (ms-oss) | 最新版 |
| **Ratchjob** | XXL-JOB 兼容调度器 (ms-content) | 最新版 |

> 所有中间件的 Docker Compose 配置文件位于 `fbc-starter/docker/` 目录下，可按需单独拉起。

---

## 🚀 快速开始

### 1. 环境准备

```bash
# 安装 Rust 工具链
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup default stable  # Rust 1.80+

# 确认版本
rustc --version
cargo --version
```

### 2. 启动基础设施

```bash
# 进入中间件目录，按需启动
cd fbc-starter/docker

docker compose -f docker-compose-mysql.yml up -d
docker compose -f docker-compose-redis.yml up -d
docker compose -f docker-compose-kafka.yml up -d
docker compose -f docker-compose-rnacos.yml up -d
docker compose -f docker-compose-rustfs.yml up -d
```

### 3. 配置环境变量

```bash
# 以 ms-auth 为例，复制并编辑环境配置
cp ms-auth/.env.example ms-auth/.env
# 编辑 .env，填入实际的数据库、Redis、Nacos 连接信息
```

### 4. 本地运行

```bash
# 启动单个微服务
cargo run -p ms-auth

# 热重载开发模式（需安装 cargo-watch）
cargo install cargo-watch
cargo watch -x 'run -p ms-auth'

# 编译检查（不运行）
cargo check -p ms-auth

# 运行测试
cargo test -p ms-auth
```

---

## 🐳 部署指南

### 前置条件

1. 安装 [Docker Engine](https://www.docker.com/) + [Docker Compose](https://docs.docker.com/compose/)
2. 创建 Docker 外部网络：
   ```bash
   docker network create fbc-network
   ```
3. 构建基础构建镜像（首次）：
   ```bash
   docker build -f docker/Dockerfile.builder -t fbc-builder:latest .
   ```

### 配置 Docker 环境变量

为每个微服务准备 `.env.docker` 文件：

```bash
# 复制模板后，将 127.0.0.1 替换为 host.docker.internal
cp ms-identity/.env.example ms-identity/.env.docker
# 编辑连接地址...
```

> **关键差异**：Docker 环境中需将所有 `127.0.0.1` / `localhost` 地址替换为 `host.docker.internal`。

### 一键构建与部署

```bash
# 在 xilulu-server 根目录执行

# 构建并启动所有服务
docker compose up -d --build

# 仅构建特定服务
docker compose up -d --build ms-auth

# 查看服务状态
docker compose ps

# 查看某服务日志
docker compose logs -f ms-auth

# 停止所有服务
docker compose down
```

### 构建流程说明

项目采用 **三阶构建** 策略，以最小化运行镜像体积：

```
fbc-builder:latest (Alpine + Rust + 系统依赖)
      ↓ cargo chef prepare
  Planner (生成 recipe.json)
      ↓ cargo chef cook + cargo build
  Builder (编译 release 二进制)
      ↓ COPY binary
  scratch (零体积运行镜像)
```

> **例外**：`ms-media-processor` 因依赖系统级 FFmpeg，运行镜像使用 `alpine:3.21` 而非 `scratch`。

---

## 📖 开发规范

本项目具有严格的架构规范约束，相关规范文件位于 `.aiproject/` 目录：

| 优先级 | 文件 | 内容 |
|--------|------|------|
| **P0** | [P0-product.md](.aiproject/P0-product.md) | 依赖管理、项目结构、启动模式 |
| **P1** | [P1-architecture.md](.aiproject/P1-architecture.md) | 分层架构、数据层、配置管理 |
| **P2** | [P2-code-style.md](.aiproject/P2-code-style.md) | 命名规范、代码风格、文档注释 |
| **P3** | [P3-api.md](.aiproject/P3-api.md) | HTTP 响应格式、错误码、gRPC、Kafka 协议 |
| **P4** | [P4-security.md](.aiproject/P4-security.md) | 安全实践、HTTP 安全 |
| **P5** | [P5-testing.md](.aiproject/P5-testing.md) | 测试策略、分层测试 |
| **P6** | [P6-deploy.md](.aiproject/P6-deploy.md) | Docker 部署、CI/CD |
| **P7** | [P7-observability.md](.aiproject/P7-observability.md) | 日志规范、链路追踪 |
| **P8** | [P8-performance.md](.aiproject/P8-performance.md) | 性能优化、缓存策略 |
| **P9** | [P9-ops.md](.aiproject/P9-ops.md) | 健康检查、数据库迁移、运维 |

### 核心约束速查

```
✅ Server::run 闭包启动          — 禁止手动初始化运行时
✅ Handler → Service → Repository — 严格单向调用链
✅ R<T> 统一 HTTP 响应            — 禁止裸露散装结构
✅ AppError 工厂方法              — 禁止通用/系统级报错
✅ sqlxplus 实体宏                — 字段 Option<T>
✅ CacheKeyBuilder 构建缓存键     — 规范化 Redis Key
✅ tracing 结构化日志             — 禁止 println!
✅ fbc-builder + scratch 部署     — 禁止 debian-slim / distroless
✅ 中文注释                       — 项目全局使用中文注释
```

---

## 📁 项目结构

```
xilulu-server/
├── Cargo.toml                  # 工作区根配置（统一依赖版本）
├── Cargo.lock                  # 锁定依赖版本
├── docker-compose.yml          # 生产部署编排
├── docker/
│   └── Dockerfile.builder      # fbc-builder 统一构建镜像
├── .aiproject/                 # P0-P9 开发规范体系
├── .agents/rules/              # AI 编码工具规则
│
├── fbc-starter/                # 🔧 微服务基础框架
├── fbc-standards/              # 📋 开发规范分发工具
│
├── ms-identity/                # 🆔 身份中心服务
├── ms-auth/                    # 🔐 认证鉴权服务
├── ms-oss/                     # 📂 对象存储服务
├── ms-team/                    # 🏢 组织团队服务
├── ms-im/                      # 💬 即时通讯服务
├── ms-notify/                  # 🔔 消息通知服务
├── ms-media-processor/         # 🎬 媒体处理服务
├── ms-content/                 # 📝 内容中台服务
├── ms-websocket/               # 🔌 WebSocket 网关
├── ms-ai/                      # 🤖 AI 代理服务 (开发中)
└── ms-identity-admin/          # 🖥️ 管理后台前端
```

### 微服务标准目录结构

每个 `ms-*` 微服务遵循统一的分层目录模板：

```
ms-xxx/
├── src/
│   ├── main.rs          # 入口 — Server::run 闭包启动
│   ├── config.rs        # 业务配置 — 从 .env / Nacos 加载
│   ├── error.rs         # 错误定义 — 基于 AppError 工厂
│   ├── state.rs         # 应用状态 — 聚合所有 Service
│   ├── router.rs        # HTTP 路由 — Axum Router 定义
│   ├── modules/         # 业务模块 — 按领域划分
│   │   └── xxx/
│   │       ├── handler.rs      # Handler 层 — HTTP 请求处理
│   │       ├── service.rs      # Service 层 — 业务逻辑编排
│   │       ├── repository.rs   # Repository 层 — 数据库 CRUD
│   │       └── model/
│   │           ├── dto.rs      # 数据传输对象 (请求/响应)
│   │           └── entity.rs   # 数据库实体 (sqlxplus 宏)
│   ├── kafka/           # Kafka 消费/生产者 (可选)
│   ├── grpc/            # gRPC 服务/客户端 (可选)
│   └── cache/           # Redis 缓存键构建器 (可选)
├── docs/                # 服务级文档
├── .env.example         # 环境变量模板
├── Dockerfile           # Docker 构建文件
├── Cargo.toml           # 服务依赖配置
└── README.md            # 服务说明文档
```

---

`xilulu-server` © 2026. All rights reserved.
