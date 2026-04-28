# ms-websocket — WebSocket 实时通讯网关

> 高性能 WebSocket 网关，所有实时通信能力的骨干长连接承载层，集成音视频信令与消息推送。

## 📋 目录

- [功能特性](#-功能特性)
- [技术栈](#️-技术栈)
- [架构设计](#-架构设计)
- [项目结构](#-项目结构)
- [快速开始](#-快速开始)
- [配置说明](#️-配置说明)
- [WebSocket 协议](#-websocket-协议)
- [与其他服务的关系](#-与其他服务的关系)

---

## ✨ 功能特性

### 连接管理

- 🔌 **大规模并发连接** — 基于 Tokio 异步多线程引擎，单机万级连接承载
- 🛡️ **长连接鉴权** — WebSocket 握手阶段注入 JWT / Token 校验
- 🏓 **心跳保活** — 可配置的心跳间隔与超时检测，时间轮算法高效管理
- 📱 **设备管理** — 按用户+设备维度管理会话，支持同设备防重连策略
- 🔄 **会话恢复** — 断线自动恢复机制，保障消息不丢失

### 消息推送

- 🔀 **内外网双向映射** — 前端请求 → 内部 Kafka/gRPC；内部 Kafka → 客户端推送
- 📨 **精准路由** — 基于 Redis 的分布式路由表，精确定位用户所在节点
- 🎯 **多种推送模式** — 单播、房间广播、全站广播

### 音视频通话

- 📞 **音视频信令** — 完整的呼叫/接听/拒绝/超时信令流
- 🖥️ **屏幕共享** — 屏幕共享信令控制
- 🎙️ **媒体控制** — 静音/取消静音/开关摄像头等信令
- 🏠 **房间管理** — 创建/加入/退出/关闭通话房间
- 📶 **网络质量监控** — 实时网络质量上报与分发

### 消息处理

- ✅ **消息确认 (ACK)** — 消息送达确认机制
- ⌨️ **正在输入** — 对方输入状态实时同步
- 📖 **已读回执** — 消息已读状态实时同步
- 🔐 **扫码登录** — 扫码成功事件推送

---

## 🛠️ 技术栈

| 类目 | 技术 | 说明 |
|------|------|------|
| **框架** | fbc-starter (Axum) | HTTP + WebSocket |
| **WebSocket** | Axum WS + Tokio | 异步 WebSocket 处理 |
| **缓存** | Redis + DashMap | 分布式路由表 + 本地会话表 |
| **消息队列** | Kafka | 消费者 + 生产者双向 |
| **内部通信** | gRPC | 调用 ms-identity 验证 Token |
| **服务发现** | Nacos | 注册/发现/负载均衡 |
| **会话状态** | Nacos Session Registry | 在线状态注册 |

---

## 🏗 架构设计

```
    客户端 (Dioxus App / 浏览器)
         │  WebSocket
         ▼
┌─────────────────────────────────────┐
│           ms-websocket              │
│  ┌───────────────────────────┐      │
│  │ WebSocket Handler         │      │
│  │  ├ 握手鉴权 (Token 校验)    │      │
│  │  └ 协议解析 (JSON Frame)   │      │
│  └───────────┬───────────────┘      │
│              ▼                      │
│  ┌───────────────────────────┐      │
│  │ Message Processor Chain   │      │
│  │  ├ Heartbeat Processor    │      │
│  │  ├ ACK Processor          │      │
│  │  ├ Read Processor         │      │
│  │  ├ Typing Processor       │      │
│  │  ├ Video Call Processor   │      │
│  │  │  ├ Call Request        │      │
│  │  │  ├ Call Response       │      │
│  │  │  ├ Media Control       │      │
│  │  │  ├ Screen Sharing      │      │
│  │  │  └ Room Admin          │      │
│  │  └ Default Processor      │      │
│  └───────────┬───────────────┘      │
│              ▼                      │
│  ┌───────────────────────────┐      │
│  │ Service Layer             │      │
│  │  ├ PushService            │ ←── Kafka Consumer
│  │  ├ VideoChatService       │      │
│  │  ├ RoomMetadataService    │      │
│  │  ├ RoomTimeoutService     │      │
│  │  └ SessionRecoveryService │      │
│  └───────────┬───────────────┘      │
│              │                      │
│  ┌───────────┴───────────────┐      │
│  │ SessionManager            │      │
│  │  ├ 本地会话表 (DashMap)     │      │
│  │  ├ 心跳时间轮 (TimingWheel)│      │
│  │  └ Nacos 在线注册          │      │
│  └───────────────────────────┘      │
│              │                      │
│  ┌───────────┴───────────────┐      │
│  │ Message Router            │      │
│  │  ├ Redis 路由表            │ ──── 多节点寻址
│  │  └ Kafka Producer         │ ──── 跨节点消息转发
│  └───────────────────────────┘      │
└─────────────────────────────────────┘
```

---

## 📁 项目结构

```
ms-websocket/
├── src/
│   ├── main.rs              # 入口
│   ├── lib.rs               # 库入口（模块导出）
│   ├── config.rs            # WsConfig — WebSocket 配置
│   ├── error.rs             # 错误定义
│   ├── state.rs             # WsState — 全局状态
│   ├── types.rs             # 类型别名
│   ├── cache/               # 📦 Redis 缓存键构建器
│   │   ├── local_router_cache.rs      # 本地路由缓存
│   │   ├── router_cache_key_builder.rs# 分布式路由表键
│   │   ├── presence_cache_key_builder.rs  # 在线状态键
│   │   ├── room_metadata_cache_key_builder.rs # 通话房间元数据
│   │   ├── user_rooms_cache_key_builder.rs    # 用户房间列表
│   │   ├── video_rooms_cache_key_builder.rs   # 视频房间列表
│   │   └── ...
│   ├── enums/               # 枚举定义
│   │   ├── ws_push_type.rs  # 推送类型枚举
│   │   ├── ws_req_type.rs   # 请求类型枚举
│   │   └── call_response_status.rs # 通话响应状态
│   ├── grpc/                # gRPC 客户端
│   │   └── client.rs        # ms-identity gRPC 调用
│   ├── kafka/               # Kafka 消费者
│   │   └── consumer/
│   │       ├── push_handler.rs       # 消息推送处理
│   │       ├── msg_login_handler.rs  # 登录消息处理
│   │       └── scan_success_handler.rs # 扫码成功处理
│   ├── model/               # 数据模型
│   │   ├── dto/             # 数据传输对象
│   │   ├── entity/          # 实体（Room 等）
│   │   ├── vo/              # 视图对象（各类消息体）
│   │   └── ws_base_resp.rs  # WebSocket 统一响应
│   ├── routes/              # HTTP 路由
│   │   └── test_push.rs     # 测试推送接口
│   ├── service/             # 🧩 业务服务
│   │   ├── push_service.rs          # 消息推送服务
│   │   ├── video_chat_service.rs    # 音视频通话服务
│   │   ├── room_metadata_service.rs # 房间元数据管理
│   │   ├── room_timeout_service.rs  # 房间超时检测
│   │   └── session_recovery_service.rs # 会话恢复服务
│   └── websocket/           # 🔌 WebSocket 核心
│       ├── handler.rs        # WS 连接处理
│       ├── session_manager.rs# 会话管理器
│       ├── timing_wheel.rs   # 心跳时间轮
│       ├── nacos_session_registry.rs # Nacos 在线注册
│       ├── processor/        # 消息处理器链
│       │   ├── message_chain.rs   # 处理器责任链
│       │   ├── heartbeat.rs       # 心跳处理
│       │   ├── ack.rs             # 消息确认
│       │   ├── read.rs            # 已读回执
│       │   ├── typing.rs          # 输入状态
│       │   └── meet/              # 音视频信令
│       │       ├── video_call.rs  # 呼叫/接听/拒绝
│       │       ├── video.rs       # 视频信令
│       │       ├── media_control.rs # 媒体控制
│       │       ├── room_admin.rs  # 房间管理
│       │       └── ...
│       └── router/           # 消息路由
│           └── message_router_service.rs # 分布式消息路由
├── .env.example
├── Dockerfile
└── Cargo.toml
```

---

## 🚀 快速开始

### 前置条件

- Rust 1.80+
- Redis 7.0+
- Kafka 3.0+
- Nacos 2.0+

### 配置与运行

```bash
# 1. 复制环境变量
cp ms-websocket/.env.example ms-websocket/.env

# 2. 运行
cargo run -p ms-websocket
```

---

## ⚙️ 配置说明

| 变量 | 默认值 | 说明 |
|------|--------|------|
| `APP__SERVER__PORT` | `30201` | 服务端口 |
| `WRITE_CHANNEL_CAP` | `1024` | 写通道缓冲区大小 |
| `HEARTBEAT_INTERVAL_SECS` | `20` | 心跳间隔（秒） |
| `HEARTBEAT_TIMEOUT_SECS` | `60` | 心跳超时时间（秒） |
| `ALLOW_MULTI_SESSION_PER_DEVICE` | `false` | 同设备是否允许多会话 |
| `APP__REDIS__URL` | — | Redis 连接串 |
| `APP__KAFKA__BROKERS` | — | Kafka Broker 地址 |
| `APP__NACOS__SERVER_ADDRS` | — | Nacos 地址 |
| `APP__NACOS__SERVICE_NAME` | `ms-websocket` | 注册服务名 |

---

## 📡 WebSocket 协议

### 连接建立

```
wss://host:30201/ws?token=<JWT_TOKEN>&device=<DEVICE_ID>
```

### 消息帧格式

所有消息使用 JSON 格式，统一结构：

```json
{
  "type": "<消息类型>",
  "data": { ... }
}
```

### 请求类型 (Client → Server)

| Type | 说明 |
|------|------|
| `HEARTBEAT` | 心跳包 |
| `ACK` | 消息确认 |
| `READ` | 已读回执 |
| `TYPING` | 正在输入 |
| `CALL_REQUEST` | 发起通话 |
| `CALL_RESPONSE` | 通话应答 |
| `CALL_END` | 结束通话 |
| `MEDIA_CONTROL` | 媒体控制（静音/关摄像头） |
| `SCREEN_SHARING` | 屏幕共享 |

### 推送类型 (Server → Client)

| Type | 说明 |
|------|------|
| `MESSAGE` | 新消息推送 |
| `ONLINE_NOTIFY` | 用户上线通知 |
| `CALL_REQUEST` | 来电通知 |
| `CALL_ACCEPTED` | 通话已接听 |
| `CALL_REJECTED` | 通话已拒绝 |
| `CALL_TIMEOUT` | 通话超时 |
| `ROOM_CLOSED` | 通话房间关闭 |
| `SCAN_SUCCESS` | 扫码登录成功 |

---

## 🔗 与其他服务的关系

| 服务 | 协议 | 关系说明 |
|------|------|----------|
| **ms-im** | Kafka | 双向 — 转发上行消息 / 接收下行推送 |
| **ms-identity** | gRPC | 单向调用 — Token 验证、用户信息查询 |
| **ms-notify** | Kafka | 投递离线推送事件（用户不在线时） |

---

## 📄 许可证

MIT OR Apache-2.0
