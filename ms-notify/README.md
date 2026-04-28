# ms-notify — 多端消息通知服务

> 全站消息通知与站外触达系统，整合 APNs / FCM / 短信 / 邮件 / 企业 IM 等多种推送渠道。

## 📋 目录

- [功能特性](#-功能特性)
- [技术栈](#️-技术栈)
- [架构设计](#-架构设计)
- [项目结构](#-项目结构)
- [快速开始](#-快速开始)
- [配置说明](#️-配置说明)
- [推送渠道](#-推送渠道)
- [Kafka 消息协议](#-kafka-消息协议)
- [与其他服务的关系](#-与其他服务的关系)

---

## ✨ 功能特性

### 推送渠道

- 📱 **移动端系统推送** — APNs (iOS) + FCM (Android) 原生推送
- 📧 **邮件推送** — SMTP 邮件发送（支持 HTML 模板）
- 💬 **短信推送** — 阿里云 SMS 短信验证码/通知
- 🔗 **飞书**  — Webhook 机器人消息推送
- 🔗 **钉钉** — Webhook 签名机器人推送
- 🔗 **企业微信** — Webhook 消息推送

### 核心能力

- 🔔 **站内通知聚合** — 点赞/评论/提及等业务行为统一生成通知
- 🌙 **免打扰模式** — 根据用户设置、时段控制推送频次
- 📊 **推送日志** — 全渠道推送记录落库（notify_log），支持追溯与分析
- 🔌 **渠道适配层** — 通过 Adapter 模式统一不同渠道的 API 差异
- ⚡ **Kafka 驱动** — 完全事件驱动架构，异步消费推送请求

---

## 🛠️ 技术栈

| 类目 | 技术 | 说明 |
|------|------|------|
| **框架** | fbc-starter (Axum) | HTTP 服务 |
| **数据库** | MySQL 8.0 | 推送日志持久化 |
| **缓存** | Redis | 免打扰状态/Token 缓存 |
| **消息队列** | Kafka (消费者) | 接收推送事件 |
| **内部通信** | gRPC | 调用 ms-identity 查询用户信息 |
| **服务发现** | Nacos | 注册/发现/负载均衡 |
| **邮件** | Lettre | Rust 原生 SMTP 客户端 |
| **短信** | 阿里云 SMS API | 签名 + HTTP 调用 |
| **APNs** | HTTP/2 + JWT | 苹果推送通知服务 |
| **FCM** | Firebase Admin SDK | Google 推送服务 |

---

## 🏗 架构设计

```
        Kafka 事件源
  ┌─────────┬─────────┬─────────┐
  │ms-auth  │ms-im    │ms-content│
  │(验证码)  │(离线消息) │(互动通知) │
  └────┬────┴────┬────┴────┬────┘
       │         │         │
       ▼         ▼         ▼
  ┌──────────────────────────────┐
  │        ms-notify             │
  │  ┌──────────────────────┐    │
  │  │ Kafka Consumer       │    │
  │  │ ├ NotificationHandler│    │
  │  │ └ OfflinePushHandler │    │
  │  └──────────┬───────────┘    │
  │             ▼                │
  │  ┌──────────────────────┐    │
  │  │ 决策引擎              │    │
  │  │ • 免打扰检查          │    │
  │  │ • 频次控制            │    │
  │  │ • 渠道路由            │    │
  │  └──────────┬───────────┘    │
  │             ▼                │
  │  ┌──────────────────────┐    │
  │  │ 渠道适配层 (Adapters)  │    │
  │  │ ├ EmailAdapter       │    │
  │  │ ├ SmsAdapter         │    │
  │  │ ├ ApnsAdapter        │    │
  │  │ ├ FcmAdapter         │    │
  │  │ ├ FeishuAdapter      │    │
  │  │ ├ DingdingAdapter    │    │
  │  │ └ WechatAdapter      │    │
  │  └──────────┬───────────┘    │
  │             ▼                │
  │  ┌──────────────────────┐    │
  │  │ NotifyLog (日志落库)   │    │
  │  └──────────────────────┘    │
  └──────────────────────────────┘
```

---

## 📁 项目结构

```
ms-notify/
├── src/
│   ├── main.rs              # 入口 — Server::run 启动
│   ├── config.rs            # NotifyConfig — 多渠道配置
│   ├── error.rs             # 错误定义
│   ├── router.rs            # HTTP 路由（健康检查等）
│   ├── adapters/            # 📡 推送渠道适配层
│   │   ├── mod.rs
│   │   ├── email.rs         # 邮件适配器 (Lettre SMTP)
│   │   ├── sms.rs           # 短信适配器 (阿里云 SMS)
│   │   ├── apns.rs          # APNs 推送适配器
│   │   ├── fcm.rs           # FCM 推送适配器
│   │   ├── feishu.rs        # 飞书 Webhook
│   │   ├── dingding.rs      # 钉钉 Webhook
│   │   └── wechat.rs        # 企业微信 Webhook
│   ├── handlers/            # HTTP 处理器
│   ├── kafka/               # Kafka 消费者
│   │   ├── mod.rs
│   │   ├── notification_handler.rs    # 通知消息处理器
│   │   └── offline_push_handler.rs    # 离线推送处理器
│   ├── models/              # 通知模型
│   │   ├── channel.rs       # 推送渠道枚举
│   │   ├── message.rs       # 消息结构体
│   │   └── notification.rs  # 通知实体
│   ├── modules/
│   │   ├── notify_log/      # 推送日志模块
│   │   │   ├── entity.rs    # 日志实体 (notify_log 表)
│   │   │   ├── repository.rs# 日志数据访问
│   │   │   └── service.rs   # 日志服务
│   │   └── push/            # 推送服务
│   │       └── service.rs   # PushService — 统一推送入口
│   └── client/              # 外部服务客户端
├── .env.example             # 环境变量模板（含全渠道配置）
├── Dockerfile               # Docker 构建
└── Cargo.toml
```

---

## 🚀 快速开始

### 前置条件

- Rust 1.80+
- MySQL 8.0+
- Redis 7.0+
- Kafka 3.0+
- Nacos 2.0+

### 配置与运行

```bash
# 1. 复制环境变量
cp ms-notify/.env.example ms-notify/.env

# 2. 编辑 .env，配置需要启用的推送渠道

# 3. 运行
cargo run -p ms-notify
```

---

## ⚙️ 配置说明

### 基础配置

| 变量 | 默认值 | 说明 |
|------|--------|------|
| `APP__SERVER__PORT` | `30104` | 服务端口 |
| `APP__LOG__LEVEL` | `info` | 日志级别 |
| `APP__KAFKA__BROKERS` | — | Kafka Broker 地址 |
| `APP__NACOS__SERVER_ADDRS` | — | Nacos 地址 |
| `APP__NACOS__SERVICE_NAME` | `ms-notify` | 注册服务名 |

### 邮件配置

| 变量 | 说明 |
|------|------|
| `APP__NOTIFY__EMAIL__SMTP_SERVER` | SMTP 服务器地址 |
| `APP__NOTIFY__EMAIL__SMTP_USER` | SMTP 用户名 |
| `APP__NOTIFY__EMAIL__SMTP_PASS` | SMTP 密码 |
| `APP__NOTIFY__EMAIL__SMTP_PORT` | SMTP 端口 (默认 587) |

### 短信配置（阿里云）

| 变量 | 说明 |
|------|------|
| `APP__NOTIFY__SMS__ENDPOINT` | API 端点 |
| `APP__NOTIFY__SMS__ACCESS_KEY_ID` | AccessKey ID |
| `APP__NOTIFY__SMS__ACCESS_KEY_SECRET` | AccessKey Secret |
| `APP__NOTIFY__SMS__SIGN_NAME` | 短信签名 |
| `APP__NOTIFY__SMS__TEMPLATE_CODE` | 模板 Code |

### APNs 推送配置

| 变量 | 说明 |
|------|------|
| `APP__NOTIFY__APNS__P8_CERT_PATH` | .p8 私钥文件路径 |
| `APP__NOTIFY__APNS__TEAM_ID` | Apple 团队 ID |
| `APP__NOTIFY__APNS__KEY_ID` | 密钥 ID |
| `APP__NOTIFY__APNS__TOPIC` | App Bundle ID |

### FCM 推送配置

| 变量 | 说明 |
|------|------|
| `APP__NOTIFY__FCM__SERVICE_ACCOUNT_JSON_PATH` | Service Account JSON 路径 |
| `APP__NOTIFY__FCM__PROJECT_ID` | GCP 项目 ID |

### 企业 IM 配置

| 变量 | 说明 |
|------|------|
| `APP__NOTIFY__FEISHU__WEBHOOK` | 飞书机器人 Webhook URL |
| `APP__NOTIFY__DINGDING__WEBHOOK` | 钉钉机器人 Webhook URL |
| `APP__NOTIFY__DINGDING__SECRET` | 钉钉签名密钥 |
| `APP__NOTIFY__WECHAT__WEBHOOK` | 企业微信 Webhook URL |

---

## 📡 推送渠道

| 渠道 | 适用场景 | 状态 |
|------|----------|------|
| **APNs** | iOS 设备离线推送 | ✅ 已实现 |
| **FCM** | Android 设备离线推送 | ✅ 已实现 |
| **Email (SMTP)** | 邮箱验证码、系统通知 | ✅ 已实现 |
| **阿里云 SMS** | 短信验证码、安全告警 | ✅ 已实现 |
| **飞书 Webhook** | 运维告警、团队通知 | ✅ 已实现 |
| **钉钉 Webhook** | 运维告警、团队通知 | ✅ 已实现 |
| **企业微信 Webhook** | 运维告警、团队通知 | ✅ 已实现 |

---

## 📨 Kafka 消息协议

### 接收事件

| 来源服务 | 事件类型 | 说明 |
|----------|----------|------|
| **ms-auth** | 验证码发送 | 短信/邮箱验证码投递 |
| **ms-im** | 离线消息推送 | 用户离线时触发 APNs/FCM 推送 |
| **ms-content** | 互动通知 | 点赞/评论/提及触发站内通知 |

---

## 🔗 与其他服务的关系

| 服务 | 协议 | 关系说明 |
|------|------|----------|
| **ms-auth** | Kafka | 接收验证码发送请求 |
| **ms-im** | Kafka | 接收离线消息推送请求 |
| **ms-content** | Kafka | 接收互动行为通知 |
| **ms-identity** | gRPC | 查询用户设备 Token / 联系信息 |

---

## 📄 许可证

MIT OR Apache-2.0
