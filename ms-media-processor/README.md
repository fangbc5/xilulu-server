# ms-media-processor — 企业级媒体处理服务

> 异步媒体处理中台，支持视频抽帧、转码、HLS 切片、图片裁剪等能力。

---

## 📋 目录

- [快速开始](#-快速开始)
- [环境要求](#-环境要求)
- [初始化步骤](#-初始化步骤)
- [配置说明](#-配置说明)
- [开发](#-开发)
- [Docker 部署](#-docker-部署)
- [架构说明](#-架构说明)
- [API / 消息协议](#-api--消息协议)

---

## 🚀 快速开始

```bash
# 1. 复制环境变量配置
cp .env.example .env

# 2. 初始化数据库（见下方详细步骤）

# 3. 创建 Kafka Topic（见下方详细步骤）

# 4. 启动开发服务
dx serve  # 如果使用 dx
# 或
cargo run -p ms-media-processor
```

---

## 📦 环境要求

| 依赖 | 版本 | 说明 |
|------|------|------|
| Rust | 1.80+ | 编程语言 |
| FFmpeg | 6.0+ | 视频/音频处理引擎 |
| MySQL | 8.0+ | 任务持久化 |
| Kafka | 3.0+ | 消息队列 |
| MinIO / S3 | 最新 | 对象存储 |
| Redis | 7.0+ | 缓存 |
| Nacos | 2.0+ | 服务注册与配置中心 |

### FFmpeg 安装

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

---

## 🔧 初始化步骤

### 1. 创建数据库

```sql
CREATE DATABASE IF NOT EXISTS `fbc_media`
  DEFAULT CHARACTER SET utf8mb4
  DEFAULT COLLATE utf8mb4_unicode_ci;
```

### 2. 执行建表脚本

```bash
mysql -u root -p fbc_media < docs/sql/init.sql
```

建表脚本位于 [`docs/sql/init.sql`](docs/sql/init.sql)，包含以下表：

| 表名 | 说明 |
|------|------|
| `media_task` | 媒体处理任务主表（含状态机、乐观锁、优先级） |
| `media_task_output` | 任务产物表（一对多，如 HLS 的多个切片） |

### 3. 创建 Kafka Topic

```bash
# 进入 Kafka 容器
docker exec -it fbc-kafka bash

# 创建 Topic（或使用 kafka-topics.sh）
kafka-topics.sh --create --bootstrap-server localhost:9092 \
  --topic sys.media.task.submit \
  --partitions 6 \
  --replication-factor 1

kafka-topics.sh --create --bootstrap-server localhost:9092 \
  --topic sys.media.task.completed \
  --partitions 3 \
  --replication-factor 1

kafka-topics.sh --create --bootstrap-server localhost:9092 \
  --topic sys.media.task.dlq \
  --partitions 1 \
  --replication-factor 1
```

**Topic 说明：**

| Topic | 方向 | 分区数 | 说明 |
|-------|------|--------|------|
| `sys.media.task.submit` | **入** | 6 | 业务服务提交处理任务，分区数=最大消费者并行度 |
| `sys.media.task.completed` | **出** | 3 | 任务完成通知（含产物路径、耗时） |
| `sys.media.task.dlq` | **出** | 1 | 死信队列（超过 3 次重试的失败任务） |

### 4. 确认 MinIO Bucket

确保以下 Bucket 已创建（通常由 `ms-oss` 服务管理）：

```bash
# 通过 mc 客户端
mc mb minio/user-uploads     # 用户上传的原始文件
```

### 5. 配置环境变量

```bash
cp .env.example .env
# 编辑 .env，填入实际的数据库、Kafka、MinIO 连接信息
```

---

## ⚙️ 配置说明

所有配置通过环境变量注入，`APP__` 前缀由 fbc-starter 框架自动加载：

### 基础配置（fbc-starter 自动加载）

| 变量 | 默认值 | 说明 |
|------|--------|------|
| `APP__SERVER__ADDR` | `0.0.0.0` | 监听地址 |
| `APP__SERVER__PORT` | `30105` | 服务端口 |
| `APP__LOG__LEVEL` | `info` | 日志级别 |
| `APP__DATABASE__MYSQL__URL` | — | MySQL 连接串 |
| `APP__KAFKA__BROKERS` | — | Kafka Broker 地址 |
| `APP__REDIS__URL` | — | Redis 连接串 |
| `APP__NACOS__SERVER_ADDRS` | — | Nacos 地址 |
| `APP__NACOS__SERVICE_NAME` | `ms-media-processor` | 服务注册名 |

### 业务配置（本服务特有）

| 变量 | 默认值 | 说明 |
|------|--------|------|
| `OSS__ENDPOINT` | `http://127.0.0.1:9000` | MinIO/S3 内网端点 |
| `OSS__PUBLIC_ENDPOINT` | 同 `OSS__ENDPOINT` | MinIO/S3 公网端点 |
| `OSS__REGION` | `us-east-1` | S3 Region |
| `OSS__ACCESS_KEY` | `minioadmin` | S3 Access Key |
| `OSS__SECRET_KEY` | `minioadmin` | S3 Secret Key |
| `MEDIA_WORK_DIR` | `/tmp/media_work` | FFmpeg 临时工作目录 |

---

## 💻 开发

```bash
# 本地运行
cargo run -p ms-media-processor

# 编译检查
cargo check -p ms-media-processor

# 运行测试
cargo test -p ms-media-processor
```

---

## 🐳 Docker 部署

### 前置条件

确保以下基础设施已通过 `fbc-starter/docker` 启动：

```bash
cd fbc-starter/docker
docker compose -f docker-compose-mysql.yml up -d
docker compose -f docker-compose-redis.yml up -d
docker compose -f docker-compose-kafka.yml up -d
docker compose -f docker-compose-rustfs.yml up -d
docker compose -f docker-compose-rnacos.yml up -d
```

### 部署服务

```bash
# 方式一：使用部署脚本
chmod +x deploy.sh && ./deploy.sh

# 方式二：手动部署
cp .env.example .env.docker
# 编辑 .env.docker（将 127.0.0.1 改为 host.docker.internal）
docker compose up -d --build
```

### Docker 环境变量注意事项

`.env.docker` 中需要将 `127.0.0.1` 替换为 `host.docker.internal`：

```env
APP__DATABASE__MYSQL__URL=mysql://root:root@host.docker.internal:3306/fbc_media
APP__KAFKA__BROKERS=host.docker.internal:9092
APP__REDIS__URL=redis://host.docker.internal:6379
OSS__ENDPOINT=http://host.docker.internal:9000
```

> **注意**：本服务 Dockerfile 使用 `alpine:3.21`（而非 `scratch`），因为需要系统级的 FFmpeg 依赖。

---

## 🏗 架构说明

```
Kafka (submit)
     │
     ▼
┌─────────────┐     ┌──────────┐     ┌──────────────────┐
│ Handler     │────▶│ Service  │────▶│ Processor (策略)   │
│ (反序列化)   │     │ (编排)    │     │ ├ VideoSnapshot   │
└─────────────┘     └────┬─────┘     │ ├ VideoTranscode  │
                         │           │ ├ VideoHls        │
                         ▼           │ └ ...             │
                    ┌──────────┐     └──────────────────┘
                    │Repository│
                    │ (MySQL)  │
                    └──────────┘
```

**分层职责：**

| 层 | 文件 | 职责 |
|----|------|------|
| Handler | `kafka/handler.rs` | 消息反序列化 → 调 Service |
| Service | `modules/media/service.rs` | 编排全流程：抢占→下载→处理→上传→通知 |
| Repository | `modules/media/repository.rs` | 乐观锁抢占、原子状态更新 |
| Processor | `modules/media/processor/*.rs` | 策略模式：每种任务类型一个处理器 |

---

## 📡 API / 消息协议

### 提交任务（Kafka 入站）

**Topic**: `sys.media.task.submit`

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

**支持的 task_type：**

| task_type | 说明 | 状态 |
|-----------|------|------|
| `VIDEO_SNAPSHOT` | 视频截图 | ✅ 已实现 |
| `VIDEO_TRANSCODE` | 视频转码 | ✅ 已实现 |
| `VIDEO_HLS` | HLS 切片 | 🔜 Phase 2 |
| `IMAGE_RESIZE` | 图片裁剪 | ✅ 已实现 |
| `IMAGE_WATERMARK` | 图片水印 | ✅ 已实现 |
| `AUDIO_EXTRACT` | 音频提取 | ✅ 已实现 |

### 任务完成（Kafka 出站）

**Topic**: `sys.media.task.completed`（或 `callback_topic`）

```json
{
    "task_id": "550e8400-e29b-41d4-a716-446655440000",
    "status": "DONE",
    "task_type": "VIDEO_SNAPSHOT",
    "original_source": "videos/original/abc123.mp4",
    "result": {
        "primary_key": "_derivative/videos_original_abc123.mp4_thumb.jpg",
        "outputs": [
            {
                "key": "_derivative/videos_original_abc123.mp4_thumb.jpg",
                "output_type": "thumbnail",
                "content_type": "image/jpeg",
                "size": null
            }
        ]
    },
    "processing_time_ms": 1234
}
```

---

## 📚 更多文档

- [媒体平台规划文档](docs/media-platform-plan.md) — 完整的架构设计、HLS 方案、分期计划
- [数据库 DDL](docs/sql/init.sql) — 建表脚本
