# ms-oss — 统一对象存储服务

> 微服务体系的对象存储抽象层，提供预签名直传、文件元数据管理、动态图像处理及多厂商适配能力。

## 📋 目录

- [功能特性](#-功能特性)
- [技术栈](#️-技术栈)
- [架构设计](#-架构设计)
- [项目结构](#-项目结构)
- [快速开始](#-快速开始)
- [配置说明](#️-配置说明)
- [API 文档](#-api-文档)
- [图像处理](#️-动态图像处理)
- [Kafka 消息协议](#-kafka-消息协议)
- [与其他服务的关系](#-与其他服务的关系)

---

## ✨ 功能特性

### 文件管理

- 🔐 **预签名 URL 签发** — 后端仅做签名，前端直传 OSS，不经过微服务中转
- 📤 **简单上传** — 小文件一次性上传
- 📦 **分片上传** — 大文件分片上传（Init → Upload Parts → Complete）
- 📥 **预签名下载** — 安全的带时效性下载链接
- 🗑️ **文件软删除** — 软删除 + 生命周期管理
- 📋 **元数据管理** — 文件信息入库（大小、类型、hash、状态）

### 存储适配

- 🔌 **多厂商适配** — 通过 `OssProvider` trait 统一接口
  - ✅ **S3 兼容** — RustFS / MinIO / AWS S3（已实现）
  - 🔜 **阿里云 OSS** — 计划中
  - 🔜 **腾讯 COS** — 计划中

### 动态图像处理

- 🖼️ **实时处理** — 基于 Imgproxy 的 URL 签名代理
- 📐 **图片缩放** — `x-oss-process=image/resize,w_200,h_200`
- ✂️ **图片裁剪** — 自定义裁剪区域
- 🎨 **格式转换** — JPEG / PNG / WebP / AVIF 互转
- 🔍 **质量控制** — 压缩质量调节
- 🎬 **视频截图** — `x-oss-process=video/snapshot,t_5000` 按需触发

### 安全

- 🔑 **JWT 签名** — Imgproxy URL 签名防盗链
- 🛡️ **上传回调验证** — 防止伪造回调请求
- ⏱️ **链接时效控制** — 预签名 URL 过期时间可配

---

## 🛠️ 技术栈

| 类目 | 技术 | 说明 |
|------|------|------|
| **框架** | fbc-starter (Axum) | HTTP 服务 + OpenAPI 文档 |
| **S3 SDK** | rust-s3 | S3 兼容协议客户端 |
| **数据库** | MySQL 8.0 | 文件元数据持久化 (file_meta 表) |
| **缓存** | Redis | 上传状态缓存 |
| **消息队列** | Kafka | 生产者 + 消费者 |
| **图像处理** | Imgproxy | 动态图像处理代理 |
| **服务发现** | Nacos | 注册/配置中心 |

---

## 🏗 架构设计

### 上传流程

```
客户端                    ms-oss                       S3 (RustFS/MinIO)
  │                         │                              │
  │ 1. 请求预签名 URL       │                              │
  │ ──────────────────────► │                              │
  │                         │ 生成签名                      │
  │ ◄────────────────────── │                              │
  │  返回 presigned URL     │                              │
  │                         │                              │
  │ 2. 直传文件              │                              │
  │ ───────────────────────────────────────────────────────►│
  │                         │                              │
  │                         │ 3. MinIO Event (Kafka)       │
  │                         │ ◄────────────────────────────│
  │                         │ 记录文件元数据                 │
  │                         │                              │
  │ 4. 访问文件              │                              │
  │ ──────────────────────► │                              │
  │                         │ 返回 presigned download URL  │
  │ ◄────────────────────── │                              │
```

### 图像处理流程

```
客户端                    ms-oss                    Imgproxy              S3
  │                         │                          │                   │
  │ GET /file?process=...   │                          │                   │
  │ ──────────────────────► │                          │                   │
  │                         │ 解析 x-oss-process       │                   │
  │                         │ 生成 Imgproxy 签名 URL   │                   │
  │ ◄────────────────────── │                          │                   │
  │  302 → Imgproxy URL    │                          │                   │
  │ ──────────────────────────────────────────────────►│                   │
  │                         │                          │ 拉取原图           │
  │                         │                          │ ──────────────────►│
  │                         │                          │ ◄──────────────────│
  │ ◄─────────────────────────────────────────────────│ 返回处理后图片      │
```

### 多厂商适配

```
ms-oss Service Layer
        │
        ▼
  OssProvider trait
   ├── S3CompatProvider   ← RustFS / MinIO / AWS S3
   ├── AliyunProvider     ← 阿里云 OSS (计划中)
   └── TencentProvider    ← 腾讯 COS (计划中)
```

---

## 📁 项目结构

```
ms-oss/
├── src/
│   ├── main.rs              # 入口 — Server::run 启动
│   ├── lib.rs               # 库入口
│   ├── config.rs            # OssConfig — S3/Imgproxy 配置
│   ├── error.rs             # OssError — 错误定义
│   ├── state.rs             # OssState — 应用状态
│   ├── router.rs            # HTTP 路由（含 OpenAPI 文档）
│   ├── cache.rs             # Redis 缓存键构建
│   ├── provider/            # 🔌 存储厂商适配层
│   │   ├── mod.rs           # OssProvider trait 定义
│   │   └── s3_compat.rs     # S3 兼容实现
│   ├── modules/
│   │   └── file/            # 📂 文件管理模块
│   │       ├── handler.rs   # HTTP API 处理器
│   │       ├── service.rs   # 文件业务逻辑
│   │       ├── repository.rs# 文件元数据访问
│   │       └── model/
│   │           └── dto.rs   # 请求/响应 DTO
│   ├── kafka/               # Kafka 处理器
│   │   ├── minio_event.rs   # MinIO Bucket 事件消费
│   │   └── media_completed.rs # 媒体处理完成回调
│   └── utils/               # 工具模块
│       ├── imgproxy.rs      # Imgproxy URL 签名生成
│       ├── jwt.rs           # JWT 工具
│       └── oss_process.rs   # x-oss-process 参数解析器
├── docs/
│   ├── service-design.md    # 服务规划文档
│   └── sql/
│       └── init.sql         # 建表脚本 (file_meta)
├── .env.example             # 环境变量模板
├── Dockerfile               # Docker 构建
└── Cargo.toml
```

---

## 🚀 快速开始

### 前置条件

- Rust 1.80+
- MySQL 8.0+
- Redis 7.0+
- RustFS / MinIO（或其他 S3 兼容存储）
- Imgproxy（可选，用于动态图像处理）
- Nacos 2.0+

### 配置与运行

```bash
# 1. 复制环境变量
cp ms-oss/.env.example ms-oss/.env

# 2. 初始化数据库
mysql -u root -p ms_oss < ms-oss/docs/sql/init.sql

# 3. 运行
cargo run -p ms-oss
```

---

## ⚙️ 配置说明

### 基础配置

| 变量 | 默认值 | 说明 |
|------|--------|------|
| `APP__SERVER__PORT` | `30003` | 服务端口 |
| `APP__LOG__LEVEL` | `info` | 日志级别 |
| `APP__DATABASE__URL` | — | MySQL 连接串 |

### OSS 配置

| 变量 | 默认值 | 说明 |
|------|--------|------|
| `OSS__PROVIDER` | `rustfs` | 存储提供商 |
| `OSS__ENDPOINT` | `http://127.0.0.1:9000` | S3 内网端点 |
| `OSS__PUBLIC_ENDPOINT` | 同 ENDPOINT | S3 公网端点 |
| `OSS__REGION` | `us-east-1` | S3 Region |
| `OSS__ACCESS_KEY` | — | S3 Access Key |
| `OSS__SECRET_KEY` | — | S3 Secret Key |
| `OSS__DEFAULT_BUCKET` | `public` | 默认 Bucket |
| `OSS__PRESIGN_EXPIRES_SECS` | `3600` | 预签名 URL 有效期(秒) |

### 其他配置

| 变量 | 说明 |
|------|------|
| `APP__REDIS__URL` | Redis 连接串 |
| `APP__KAFKA__BROKERS` | Kafka Broker 地址 |
| `APP__NACOS__SERVER_ADDRS` | Nacos 地址 |
| `APP__NACOS__SERVICE_NAME` | 注册服务名 (`ms-oss`) |

---

## 📚 API 文档

> 服务启动后可访问 OpenAPI 文档查看完整接口详情。

### 文件上传

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/oss/presign/upload` | 获取简单上传预签名 URL |
| POST | `/oss/multipart/init` | 初始化分片上传 |
| POST | `/oss/multipart/presign` | 获取分片预签名 URL |
| POST | `/oss/multipart/complete` | 完成分片上传 |

### 文件下载与访问

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/oss/presign/download` | 获取预签名下载 URL |
| GET | `/oss/files/:bucket/:key` | 获取文件（支持 x-oss-process 动态处理） |

### 文件管理

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/oss/files/:id` | 查询文件元数据 |
| DELETE | `/oss/files/:id` | 删除文件（软删除） |

---

## 🖼️ 动态图像处理

通过 `x-oss-process` 参数触发实时图像处理（由 Imgproxy 执行）：

```bash
# 缩放图片
GET /oss/files/bucket/key?x-oss-process=image/resize,w_200,h_200

# 格式转换
GET /oss/files/bucket/key?x-oss-process=image/format,webp

# 质量控制
GET /oss/files/bucket/key?x-oss-process=image/quality,q_80

# 组合处理
GET /oss/files/bucket/key?x-oss-process=image/resize,w_400/format,webp/quality,q_85

# 视频截图（按需触发，首次可能返回 202）
GET /oss/files/bucket/key?x-oss-process=video/snapshot,t_5000,f_jpg
```

---

## 📨 Kafka 消息协议

| Topic | 方向 | 说明 |
|-------|------|------|
| MinIO Bucket Event | **入** | 监听 MinIO 的文件上传事件，自动记录元数据 |
| `sys.media.task.submit` | **出** | 视频截图请求投递给 ms-media-processor |
| `sys.media.task.completed` | **入** | 接收媒体处理完成通知，同步产物元数据 |

---

## 🔗 与其他服务的关系

| 服务 | 协议 | 关系说明 |
|------|------|----------|
| **ms-media-processor** | Kafka | 双向 — 投递媒体任务 / 接收完成回调 |
| **客户端** | HTTP | 预签名 URL 下发 + 动态图像处理代理 |
| **S3 存储** | S3 协议 | 文件存取 |
| **Imgproxy** | HTTP | 图像处理代理 |

---

## 📄 许可证

MIT OR Apache-2.0
