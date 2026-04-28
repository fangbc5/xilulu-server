# ms-content — 内容中台服务

> 统一内容管理中台，支持多内容形态、Block DSL 结构化正文、内容关系图谱、全文搜索及分布式定时任务。

## 📋 目录

- [功能特性](#-功能特性)
- [技术栈](#️-技术栈)
- [架构设计](#-架构设计)
- [项目结构](#-项目结构)
- [快速开始](#-快速开始)
- [配置说明](#️-配置说明)
- [API 文档](#-api-文档)
- [搜索引擎集成](#-搜索引擎集成)
- [定时任务](#-定时任务)
- [与其他服务的关系](#-与其他服务的关系)

---

## ✨ 功能特性

### 内容管理

- 📄 **多内容形态** — 统一管理文章、动态、评论等异构内容类型
- 🧱 **Block DSL 正文** — 结构化块级 DSL，适配全平台原生渲染
- 🕸️ **内容关系图谱** — 维护"内容-内容"多维关系（评论树、引用、收藏关联）
- 📑 **内容 CRUD** — 完整的创建/读取/更新/删除生命周期

### 搜索与检索

- 🔍 **全文搜索** — 基于 Meilisearch 毫秒级全文检索
- 📊 **聚合分析** — 搜索结果聚合与高亮
- 🔄 **索引同步** — 内容变更自动同步至搜索引擎

### 任务调度

- ⏰ **XXL-JOB 集成** — 通过 Ratchjob（XXL-JOB 兼容）执行分布式定时任务
- 📊 **定时统计** — 内容热度统计、排行榜更新等

### 开发者体验

- 📖 **OpenAPI 文档** — 内置 Utoipa + Swagger UI 可交互式接口文档

---

## 🛠️ 技术栈

| 类目 | 技术 | 说明 |
|------|------|------|
| **框架** | fbc-starter (Axum) | HTTP 服务 + OpenAPI |
| **数据库** | MySQL 8.0 | sqlx + sqlxplus ORM |
| **缓存** | Redis | 排行榜、热点数据缓存 |
| **搜索引擎** | Meilisearch | 毫秒级全文检索 |
| **任务调度** | Ratchjob / XXL-JOB | 分布式定时任务 |
| **服务发现** | Nacos | 注册/配置中心 |

---

## 🏗 架构设计

```
客户端请求
     │
     ▼
┌──────────────────────────────┐
│         ms-content           │
│                              │
│  ┌────────────────────────┐  │
│  │ Handler (HTTP API)     │  │ ◄── Swagger UI
│  │  ├ 内容 CRUD           │  │
│  │  └ 搜索查询            │  │
│  └────────┬───────────────┘  │
│           ▼                  │
│  ┌────────────────────────┐  │
│  │ Service (业务编排)      │  │
│  │  ├ 内容生命周期         │  │
│  │  ├ 关系图谱计算         │  │
│  │  └ 搜索索引同步         │  │
│  └───┬────────────┬───────┘  │
│      │            │          │
│      ▼            ▼          │
│  ┌────────┐  ┌──────────┐   │
│  │  MySQL │  │Meilisearch│   │
│  │  (数据) │  │ (搜索)    │   │
│  └────────┘  └──────────┘   │
│                              │
│  ┌────────────────────────┐  │
│  │ Job Module             │  │ ◄── Ratchjob/XXL-JOB
│  │  └ DemoJob (定时任务)   │  │
│  └────────────────────────┘  │
└──────────────────────────────┘
```

### 搜索架构（端口适配器模式）

```
ContentService
      │
      ▼
  SearchPort (trait)           ← 搜索能力端口抽象
      │
      ▼
  MeilisearchAdapter          ← 具体实现
      │
      ▼
  Meilisearch API
```

---

## 📁 项目结构

```
ms-content/
├── src/
│   ├── main.rs              # 入口 — Server::run 启动 + XXL-JOB 注册
│   ├── config.rs            # ContentConfig — Meilisearch/XXL-JOB 配置
│   ├── error.rs             # 错误定义
│   ├── state.rs             # ContentState — 应用状态
│   ├── router.rs            # HTTP 路由（含 Swagger UI）
│   ├── job/                 # ⏰ 定时任务模块
│   │   ├── mod.rs           # 任务注册入口
│   │   └── demo_job.rs      # 示例任务
│   └── modules/
│       └── content/          # 📝 内容模块
│           ├── handler.rs    # HTTP API 处理器
│           ├── service.rs    # 内容业务逻辑
│           ├── repository.rs # 内容数据访问
│           ├── model/
│           │   ├── dto.rs    # 请求/响应 DTO
│           │   ├── entity.rs # 数据库实体
│           │   └── domain.rs # 领域模型
│           └── search/       # 🔍 搜索集成
│               ├── port.rs   # SearchPort trait (端口)
│               └── adapter.rs# MeilisearchAdapter (适配器)
├── docs/
│   └── sql/
│       └── init.sql          # 建表脚本
├── .env.example              # 环境变量模板
├── Dockerfile                # Docker 构建
└── Cargo.toml
```

---

## 🚀 快速开始

### 前置条件

- Rust 1.80+
- MySQL 8.0+
- Redis 7.0+
- Meilisearch (最新版)
- Nacos 2.0+
- Ratchjob / XXL-JOB (可选，定时任务)

### 配置与运行

```bash
# 1. 复制环境变量
cp ms-content/.env.example ms-content/.env

# 2. 初始化数据库
mysql -u root -p ms_content < ms-content/docs/sql/init.sql

# 3. 运行
cargo run -p ms-content
```

### 访问 API 文档

服务启动后访问 Swagger UI：

```
http://localhost:30106/swagger-ui
```

---

## ⚙️ 配置说明

### 基础配置

| 变量 | 默认值 | 说明 |
|------|--------|------|
| `APP__SERVER__PORT` | `30106` | HTTP 服务端口 |
| `APP__LOG__LEVEL` | `info,ms_content=debug` | 日志级别 |
| `APP__DATABASE__URL` | — | MySQL 连接串 |
| `APP__REDIS__URL` | — | Redis 连接串 |
| `APP__NACOS__SERVER_ADDRS` | — | Nacos 地址 |
| `APP__NACOS__SERVICE_NAME` | `ms-content` | 注册服务名 |

### 搜索引擎配置

| 变量 | 说明 |
|------|------|
| `APP__CONTENT__MEILISEARCH_URL` | Meilisearch 服务地址 |
| `APP__CONTENT__MEILISEARCH_API_KEY` | Master Key |

### 任务调度配置

| 变量 | 说明 |
|------|------|
| `APP__CONTENT__XXL_ADMIN_ADDR` | Ratchjob/XXL-JOB 管理端地址 (API 端口) |
| `APP__CONTENT__XXL_ACCESS_TOKEN` | 访问令牌 |
| `APP__CONTENT__XXL_EXECUTOR_PORT` | 执行器回调端口 (默认 `31106`) |

> ⚠️ **注意**：`XXL_ADMIN_ADDR` 需配置为 API 端口（如 `8725`），而非 Web UI 端口。

---

## 📚 API 文档

> 完整的 API 文档可通过 Swagger UI 在线浏览，以下为核心接口概览。

### 内容管理

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/api/content` | 创建内容 |
| GET | `/api/content/:id` | 获取内容详情 |
| PUT | `/api/content/:id` | 更新内容 |
| DELETE | `/api/content/:id` | 删除内容 |
| GET | `/api/content/list` | 内容列表 |

### 搜索

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/api/content/search` | 全文搜索 |

---

## 🔍 搜索引擎集成

- 采用**端口适配器模式** (Ports & Adapters)，解耦搜索引擎实现
- `SearchPort` trait 定义搜索能力（索引/查询/删除）
- `MeilisearchAdapter` 实现具体的 Meilisearch API 调用
- 未来可轻松替换为 Elasticsearch 等其他引擎

---

## ⏰ 定时任务

- 基于 XXL-JOB / Ratchjob 分布式任务调度
- 任务注册在 `job/` 模块
- 服务启动时自动注册执行器
- 调度器通过 HTTP 回调执行器端口触发任务

---

## 🔗 与其他服务的关系

| 服务 | 协议 | 关系说明 |
|------|------|----------|
| **ms-notify** | Kafka | 投递互动通知（评论/点赞/提及） |
| **ms-oss** | HTTP | 内容中的图片/视频/附件管理 |
| **Meilisearch** | HTTP | 全文搜索索引同步 |
| **Ratchjob** | HTTP | 定时任务调度 |

---

## 📄 许可证

MIT OR Apache-2.0
