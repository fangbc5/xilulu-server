# ms-im — 即时通讯服务

> Xilulu 的核心即时通信枢纽，承担消息的持久化沉淀、漫游下发、社交关系链管理与群聊协调。

## 📋 目录

- [功能特性](#-功能特性)
- [技术栈](#️-技术栈)
- [架构设计](#-架构设计)
- [项目结构](#-项目结构)
- [快速开始](#-快速开始)
- [配置说明](#️-配置说明)
- [API 文档](#-api-文档)
- [Kafka 消息协议](#-kafka-消息协议)
- [与其他服务的关系](#-与其他服务的关系)

---

## ✨ 功能特性

### 消息系统

- 💬 **私聊与群聊** — 支持单聊/群聊双轨消息互传
- 🗄️ **消息持久化** — 全量消息入库 MySQL，支持历史消息查询
- 🔄 **消息漫游** — 基于 Seq 游标的断线增量消息拉取
- 📖 **阅读状态同步** — 已读/未读游标跟踪，多端同步
- 🏷️ **丰富消息类型** — 文本、图片、语音、视频、文件、位置、合并转发等
- ⚡ **消息标记** — 点赞、收藏、置顶等标记能力

### 社交关系链

- 👥 **好友管理** — 好友申请/审批/删除完整流程
- 🚫 **拉黑机制** — 单向/双向拉黑拦截
- 📇 **联系人管理** — 联系人列表与排序、免打扰设置

### 群聊管理

- 🏠 **房间系统** — 创建/加入/退出/解散群聊
- 👑 **角色体系** — 群主/管理员/普通成员分级权限
- 🔕 **群消息控制** — 全员禁言、单人禁言

### 消息同步

- 📡 **增量同步** — 客户端通过 Seq 拉取离线期间的增量消息
- 🔔 **实时推送** — 通过 Kafka 投递至 ms-websocket 网关实时下发

---

## 🛠️ 技术栈

| 类目 | 技术 | 说明 |
|------|------|------|
| **框架** | fbc-starter (Axum) | HTTP + gRPC 双协议 |
| **数据库** | MySQL 8.0 | sqlx + sqlxplus ORM |
| **缓存** | Redis | 联系人免打扰状态缓存等 |
| **消息队列** | Kafka | 消息投递解耦 (生产者 + 消费者) |
| **内部通信** | gRPC (Tonic) | 提供 IM 服务端接口 + 调用 ms-identity |
| **服务发现** | Nacos | 注册/发现/负载均衡 |

---

## 🏗 架构设计

```
客户端 (Dioxus App)
      │
      │ WebSocket
      ▼
┌──────────────┐    Kafka     ┌──────────────┐
│ ms-websocket │◀────────────▶│   ms-im      │
│  (接入网关)   │             │  (IM 核心)    │
└──────────────┘             └──────┬───────┘
                                    │
                    ┌───────────────┼───────────────┐
                    ▼               ▼               ▼
              ┌──────────┐   ┌──────────┐   ┌──────────┐
              │  MySQL   │   │  Redis   │   │  Kafka   │
              │ (持久化)  │   │ (缓存)   │   │ (投递)   │
              └──────────┘   └──────────┘   └──────────┘
                    │
                    ▼ gRPC
              ┌──────────────┐
              │ ms-identity  │
              │ (用户信息查询) │
              └──────────────┘
```

**数据流**：
1. 客户端通过 WebSocket 连接 `ms-websocket`
2. `ms-websocket` 将消息通过 Kafka 投递给 `ms-im`
3. `ms-im` 完成消息清洗、合规校验、持久化入库
4. `ms-im` 通过 Kafka 将消息推送给 `ms-websocket` 做实时下发
5. 需要用户信息时通过 gRPC 调用 `ms-identity`

---

## 📁 项目结构

```
ms-im/
├── src/
│   ├── main.rs              # 入口 — Server::run 启动
│   ├── config.rs            # IM 业务配置
│   ├── error.rs             # 错误定义
│   ├── state.rs             # ImState — 聚合所有 Service
│   ├── router.rs            # HTTP 路由定义
│   ├── cache/               # Redis 缓存键
│   │   └── contact_mute_cache_key_builder.rs  # 联系人免打扰缓存键
│   ├── client/              # 外部服务客户端
│   │   └── identity.rs      # ms-identity gRPC 客户端
│   ├── enums/               # 业务枚举定义
│   │   ├── message_type.rs  # 消息类型 (文本/图片/语音/视频...)
│   │   ├── message_status.rs# 消息状态
│   │   ├── room_type.rs     # 房间类型 (单聊/群聊)
│   │   ├── group_role.rs    # 群角色 (群主/管理员/成员)
│   │   ├── black_type.rs    # 拉黑类型
│   │   └── ...
│   ├── grpc/                # gRPC 服务端
│   │   ├── health.rs        # 健康检查服务
│   │   └── im.rs            # IM gRPC 服务接口
│   ├── kafka/               # Kafka 消息处理
│   │   └── mod.rs           # 消息投递与消费
│   ├── model/               # 全局模型
│   │   ├── dto/             # 公共 DTO
│   │   │   └── summary_info.rs  # 用户摘要信息
│   │   └── entity/          # 公共实体
│   └── modules/             # 🧩 业务模块
│       ├── contact/         # 联系人模块
│       │   ├── handler.rs   # 联系人 HTTP API
│       │   ├── service.rs   # 联系人业务逻辑
│       │   ├── repository.rs# 联系人数据访问
│       │   └── model/       # 联系人 DTO + Entity
│       ├── friend/          # 好友模块
│       │   ├── handler.rs   # 好友申请/审批 API
│       │   ├── service.rs   # 好友关系链逻辑
│       │   ├── repository.rs# 好友数据访问
│       │   └── model/       # 好友 DTO + Entity
│       ├── message/         # 消息模块
│       │   ├── handler.rs   # 消息发送/查询 API
│       │   ├── service.rs   # 消息处理/持久化
│       │   ├── repository.rs# 消息数据访问
│       │   └── model/       # 消息 DTO + Entity
│       ├── room/            # 房间/群聊模块
│       │   ├── handler.rs   # 群管理 API
│       │   ├── service.rs   # 群聊业务逻辑
│       │   ├── repository.rs# 群数据访问
│       │   └── model/       # 群 DTO + Entity
│       └── sync/            # 消息同步模块
│           ├── handler.rs   # 增量同步 API
│           ├── service.rs   # Seq 同步逻辑
│           └── model/       # 同步 DTO
├── .env.example             # 环境变量模板
├── Dockerfile               # Docker 构建（fbc-builder + scratch）
└── Cargo.toml
```

---

## 🚀 快速开始

### 前置条件

- Rust 1.80+
- MySQL 8.0+ (需初始化 IM 数据库)
- Redis 7.0+
- Kafka 3.0+
- Nacos 2.0+

### 配置与运行

```bash
# 1. 复制环境变量
cp ms-im/.env.example ms-im/.env
# 编辑 .env，填入实际连接信息

# 2. 初始化数据库（如有 SQL 脚本）
mysql -u root -p ms_im < ms-im/docs/sql/init.sql

# 3. 运行
cargo run -p ms-im
```

---

## ⚙️ 配置说明

| 变量 | 默认值 | 说明 |
|------|--------|------|
| `APP__SERVER__PORT` | `30102` | 服务端口 |
| `APP__LOG__LEVEL` | `info` | 日志级别 |
| `APP__DATABASE__URL` | — | MySQL 连接串 |
| `APP__DATABASE__MAX_CONNECTIONS` | `100` | 最大连接数 |
| `APP__REDIS__URL` | — | Redis 连接串 |
| `APP__KAFKA__BROKERS` | — | Kafka Broker 地址 |
| `APP__NACOS__SERVER_ADDRS` | — | Nacos 地址 |
| `APP__NACOS__SERVICE_NAME` | `ms-im` | 注册服务名 |
| `APP__NACOS__SUBSCRIBE_SERVICES` | `["ms-identity"]` | 订阅的服务列表 |

---

## 📚 API 文档

### 好友模块

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/api/friend/apply` | 发起好友申请 |
| POST | `/api/friend/approve` | 审批好友申请 |
| DELETE | `/api/friend/:id` | 删除好友 |
| GET | `/api/friend/list` | 好友列表 |
| POST | `/api/friend/black` | 拉黑用户 |

### 消息模块

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/api/message/send` | 发送消息 |
| GET | `/api/message/page` | 分页查询消息 |
| POST | `/api/message/mark` | 消息标记 |

### 联系人模块

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/api/contact/list` | 联系人列表 |
| PUT | `/api/contact/mute` | 免打扰设置 |

### 房间模块

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/api/room/group/create` | 创建群聊 |
| POST | `/api/room/group/invite` | 邀请入群 |
| POST | `/api/room/group/exit` | 退出群聊 |

### 同步模块

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/api/sync/messages` | 增量消息拉取 |

---

## 📡 Kafka 消息协议

| Topic | 方向 | 说明 |
|-------|------|------|
| 消息投递 Topic | **出** | 发送已处理的消息至 ms-websocket 做实时下发 |
| 消息接收 Topic | **入** | 接收来自 ms-websocket 的用户上行消息 |

---

## 🔗 与其他服务的关系

| 服务 | 协议 | 关系说明 |
|------|------|----------|
| **ms-websocket** | Kafka | 双向 — 接收上行消息 / 推送下行消息 |
| **ms-identity** | gRPC | 单向调用 — 查询用户信息 |
| **ms-notify** | Kafka | 单向投递 — 离线消息触发推送通知 |

---

## 📄 许可证

MIT OR Apache-2.0
