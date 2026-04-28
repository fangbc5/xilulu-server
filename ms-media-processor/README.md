# ms-media-processor — 企业级媒体处理服务

> 异步媒体处理中台，支持视频截图/转码、图片缩放/水印、音频提取等能力，基于 Kafka 事件驱动 + FFmpeg 处理引擎。

## 📋 目录

- [功能特性](#-功能特性)
- [技术栈](#️-技术栈)
- [架构设计](#-架构设计)
- [项目结构](#-项目结构)
- [快速开始](#-快速开始)
- [配置说明](#️-配置说明)
- [Kafka 消息协议](#-kafka-消息协议)
- [与其他服务的关系](#-与其他服务的关系)

---

## ✨ 功能特性

### 处理能力

| 能力 | 类型 | 状态 |
|------|------|------|
| 🎬 **视频截图** | `VIDEO_SNAPSHOT` | ✅ 已实现 |
| 🔄 **视频转码** | `VIDEO_TRANSCODE` | ✅ 已实现 |
| 🎵 **音频提取** | `AUDIO_EXTRACT` | ✅ 已实现 |
| 📐 **图片缩放** | `IMAGE_RESIZE` | ✅ 已实现 |
| 💧 **图片水印** | `IMAGE_WATERMARK` | ✅ 已实现 |
| 📺 **HLS 自适应切片** | `VIDEO_HLS` | 🔜 Phase 2 |

### 核心特性

- ⚡ **全异步架构** — Kafka 事件驱动，无阻塞处理流水线
- 🔒 **乐观锁抢占** — 多实例竞争安全，同一任务不会被重复处理
- 🔄 **自动重试** — 失败任务自动重试（最多 3 次），超限进入死信队列
- 📊 **状态机管理** — `PENDING → PROCESSING → DONE / FAILED / DLQ` 完整生命周期
- 🎯 **策略模式** — 每种任务类型对应独立的 Processor 实现
- 📦 **S3 兼容** — 从 MinIO/RustFS/AWS S3 下载原始文件 + 上传处理产物
- 🔔 **完成通知** — 处理完成后通过 Kafka 通知业务方

---

## 🛠️ 技术栈

| 类目 | 技术 | 说明 |
|------|------|------|
| **框架** | fbc-starter (Axum) | HTTP 服务（健康检查） |
| **处理引擎** | FFmpeg 6.0+ | 视频/音频处理 |
| **数据库** | MySQL 8.0 | 任务持久化 + 乐观锁 |
| **缓存** | Redis | 任务状态缓存 |
| **消息队列** | Kafka | 消费者（接收任务）+ 生产者（通知完成） |
| **对象存储** | rust-s3 | S3 兼容协议（MinIO/RustFS/AWS） |
| **服务发现** | Nacos | 注册/配置中心 |

---

## 🏗 架构设计

```
ms-oss / 业务服务
      │
      │ Kafka: sys.media.task.submit
      ▼
┌──────────────────────────────────────────────┐
│             ms-media-processor               │
│                                              │
│  ┌────────────────────────────────────┐      │
│  │ Kafka Handler (反序列化)            │      │
│  └──────────┬─────────────────────────┘      │
│             ▼                                │
│  ┌────────────────────────────────────┐      │
│  │ Service (编排全流程)                │      │
│  │  1. 乐观锁抢占任务 (MySQL)          │      │
│  │  2. 下载源文件 (S3)                 │      │
│  │  3. 路由到对应 Processor            │      │
│  │  4. 上传处理产物 (S3)               │      │
│  │  5. 更新任务状态 (MySQL)            │      │
│  │  6. 发送完成通知 (Kafka)            │      │
│  └──────┬─────────────────────────────┘      │
│         │                                    │
│  ┌──────▼─────────────────────────────┐      │
│  │ Processor (策略模式)                │      │
│  │  ├ VideoSnapshotProcessor  (FFmpeg) │      │
│  │  ├ VideoTranscodeProcessor (FFmpeg) │      │
│  │  ├ AudioExtractProcessor   (FFmpeg) │      │
│  │  ├ ImageResizeProcessor    (内置)   │      │
│  │  └ ImageWatermarkProcessor (内置)   │      │
│  └────────────────────────────────────┘      │
│                                              │
│  ┌─────────────┐  ┌─────────────┐            │
│  │  Repository  │  │  S3 Client  │            │
│  │  (MySQL)     │  │  (MinIO)    │            │
│  └─────────────┘  └─────────────┘            │
└──────────────────────────────────────────────┘
      │
      │ Kafka: sys.media.task.completed / callback_topic
      ▼
ms-oss / 业务服务
```

**任务状态机**：
```
PENDING → PROCESSING → DONE
                    ↘ FAILED (retry < 3)
                       ↘ DLQ (retry >= 3)
```

---

## 📁 项目结构

```
ms-media-processor/
├── src/
│   ├── main.rs              # 入口 — Server::run + Kafka Consumer
│   ├── config.rs            # MediaConfig — S3/FFmpeg 配置
│   ├── error.rs             # 错误定义
│   ├── kafka/               # 📨 Kafka 处理器
│   │   ├── mod.rs           # Consumer 注册
│   │   └── handler.rs       # 任务消息反序列化 + 路由
│   └── modules/
│       └── media/           # 🎬 媒体处理模块
│           ├── service.rs   # 全流程编排
│           ├── repository.rs# 乐观锁 + 任务状态管理
│           ├── s3_client.rs # S3 文件上传/下载
│           ├── model/
│           │   ├── dto.rs   # Kafka 消息 DTO
│           │   ├── entity.rs# media_task / media_task_output 实体
│           │   └── enums.rs # 任务类型/状态枚举
│           └── processor/   # 🎯 处理器（策略模式）
│               ├── mod.rs           # Processor trait
│               ├── video_snapshot.rs # 视频截图 (FFmpeg)
│               ├── video_transcode.rs# 视频转码 (FFmpeg)
│               ├── audio_extract.rs  # 音频提取 (FFmpeg)
│               ├── image_resize.rs   # 图片缩放
│               └── image_watermark.rs# 图片水印
├── docs/
│   ├── sql/init.sql             # 建表脚本
│   └── media-platform-plan.md   # 架构规划文档
├── .env.example                 # 环境变量模板
├── Dockerfile                   # Docker 构建 (alpine:3.21 + FFmpeg)
└── Cargo.toml
```

---

## 🚀 快速开始

### 环境要求

| 依赖 | 版本 | 说明 |
|------|------|------|
| Rust | 1.80+ | 编程语言 |
| FFmpeg | 6.0+ | 视频/音频处理引擎 |
| MySQL | 8.0+ | 任务持久化 |
| Kafka | 3.0+ | 消息队列 |
| MinIO / S3 | 最新 | 对象存储 |
| Redis | 7.0+ | 缓存 |
| Nacos | 2.0+ | 服务发现 |

### 安装 FFmpeg

```bash
# macOS
brew install ffmpeg

# Ubuntu / Debian
sudo apt install ffmpeg

# Alpine (Docker)
apk add --no-cache ffmpeg

# 验证
ffmpeg -version
```

### 初始化步骤

```bash
# 1. 创建数据库
mysql -e "CREATE DATABASE IF NOT EXISTS fbc_media DEFAULT CHARACTER SET utf8mb4"

# 2. 执行建表脚本
mysql -u root -p fbc_media < docs/sql/init.sql

# 3. 创建 Kafka Topic
docker exec -it fbc-kafka kafka-topics.sh --create \
  --bootstrap-server localhost:9092 \
  --topic sys.media.task.submit --partitions 6 --replication-factor 1

docker exec -it fbc-kafka kafka-topics.sh --create \
  --bootstrap-server localhost:9092 \
  --topic sys.media.task.completed --partitions 3 --replication-factor 1

docker exec -it fbc-kafka kafka-topics.sh --create \
  --bootstrap-server localhost:9092 \
  --topic sys.media.task.dlq --partitions 1 --replication-factor 1

# 4. 配置环境变量
cp .env.example .env

# 5. 启动
cargo run -p ms-media-processor
```

---

## ⚙️ 配置说明

### 基础配置

| 变量 | 默认值 | 说明 |
|------|--------|------|
| `APP__SERVER__PORT` | `30105` | 服务端口 |
| `APP__LOG__LEVEL` | `info` | 日志级别 |
| `APP__DATABASE__MYSQL__URL` | — | MySQL 连接串 |
| `APP__REDIS__URL` | — | Redis 连接串 |
| `APP__KAFKA__BROKERS` | — | Kafka Broker 地址 |
| `APP__NACOS__SERVER_ADDRS` | — | Nacos 地址 |
| `APP__NACOS__SERVICE_NAME` | `ms-media-processor` | 注册服务名 |

### S3/MinIO 配置

| 变量 | 默认值 | 说明 |
|------|--------|------|
| `OSS__ENDPOINT` | `http://127.0.0.1:9000` | S3 内网端点 |
| `OSS__PUBLIC_ENDPOINT` | 同 ENDPOINT | S3 公网端点 |
| `OSS__REGION` | `us-east-1` | S3 Region |
| `OSS__ACCESS_KEY` | `minioadmin` | Access Key |
| `OSS__SECRET_KEY` | `minioadmin` | Secret Key |

### FFmpeg 配置

| 变量 | 默认值 | 说明 |
|------|--------|------|
| `MEDIA_WORK_DIR` | `/tmp/media_work` | FFmpeg 临时工作目录 |

---

## 📨 Kafka 消息协议

### 提交任务（入站）

**Topic**: `sys.media.task.submit` (6 分区)

```json
{
    "task_id": "550e8400-e29b-41d4-a716-446655440000",
    "task_type": "VIDEO_SNAPSHOT",
    "source": {
        "bucket": "user-uploads",
        "key": "videos/original/abc123.mp4"
    },
    "parameters": {
        "time_offset_ms": 5000,
        "format": "jpg",
        "quality": 2
    },
    "priority": "high",
    "callback_topic": "biz.video.ready"
}
```

### 任务完成（出站）

**Topic**: `sys.media.task.completed` (3 分区) 或 `callback_topic`

```json
{
    "task_id": "550e8400-e29b-41d4-a716-446655440000",
    "status": "DONE",
    "task_type": "VIDEO_SNAPSHOT",
    "original_source": "videos/original/abc123.mp4",
    "result": {
        "primary_key": "_derivative/videos_original_abc123.mp4_thumb.jpg",
        "outputs": [{
            "key": "_derivative/videos_original_abc123.mp4_thumb.jpg",
            "output_type": "thumbnail",
            "content_type": "image/jpeg",
            "size": null
        }]
    },
    "processing_time_ms": 1234
}
```

### 死信队列

**Topic**: `sys.media.task.dlq` (1 分区) — 超过 3 次重试的失败任务

---

## 🐳 Docker 部署

> ⚠️ 本服务 Dockerfile 使用 `alpine:3.21`（而非 `scratch`），因为需要系统级的 FFmpeg 依赖。

```bash
# 方式一：docker compose（推荐）
docker compose up -d --build ms-media-processor

# 方式二：手动部署
cp .env.example .env.docker
# 编辑 .env.docker（将 127.0.0.1 改为 host.docker.internal）
docker compose up -d --build ms-media-processor
```

---

## 🔗 与其他服务的关系

| 服务 | 协议 | 关系说明 |
|------|------|----------|
| **ms-oss** | Kafka | 双向 — 接收处理任务 / 回调完成通知 |
| **S3 存储** | S3 协议 | 下载源文件 + 上传处理产物 |
| **FFmpeg** | CLI 调用 | 视频/音频处理引擎 |

---

## 📚 更多文档

- [媒体平台规划文档](docs/media-platform-plan.md) — 完整架构设计、HLS 方案、分期计划
- [数据库 DDL](docs/sql/init.sql) — 建表脚本

---

## 📄 许可证

MIT OR Apache-2.0
