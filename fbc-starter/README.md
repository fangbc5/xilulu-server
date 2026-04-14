# fbc-starter

[![Crates.io](https://img.shields.io/crates/v/fbc-starter.svg)](https://crates.io/crates/fbc-starter)
[![Documentation](https://docs.rs/fbc-starter/badge.svg)](https://docs.rs/fbc-starter)
[![License](https://img.shields.io/crates/l/fbc-starter.svg)](LICENSE)

一个基于 Rust 和 Axum 的生产级 Web 服务器启动器，提供了开箱即用的基础功能和最佳实践。

## ✨ 功能特性

### 核心功能

- ✅ **Web 框架**: 基于 Axum 的高性能异步 Web 框架
- ✅ **配置管理**: 使用 `.env` 文件和环境变量进行配置（支持 `dotenvy`）
- ✅ **日志系统**: 集成 tracing 和 tracing-subscriber，支持结构化日志，默认 UTC+8 时区
- ✅ **CORS 支持**: 可配置的跨域资源共享
- ✅ **请求压缩**: 自动 Gzip 压缩响应
- ✅ **健康检查**: 提供符合 Prometheus 标准的 `/health` 端点
- ✅ **优雅关闭**: 支持 Ctrl+C 和 SIGTERM 信号
- ✅ **错误处理**: 统一的错误处理机制
- ✅ **中间件**: 请求追踪、日志记录等
- ✅ **上下文路径**: 支持可配置的服务上下文路径

### 可选特性

- 🔌 **gRPC 支持**: 基于 Tonic 的 gRPC 服务集成
- 🗄️ **数据库**: PostgreSQL/MySQL/SQLite 支持（基于 SQLx）
- 📦 **Redis 缓存**: Redis 连接池和缓存键构建器
- 🔍 **服务发现**: Nacos 服务注册、发现和订阅
- ⚖️ **负载均衡**: 基于 Nacos 的服务发现和轮询负载均衡
- 📨 **消息队列**: Kafka 生产者和消费者支持

## 🚀 快速开始

### 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
fbc-starter = "0.1.5"

# 或启用所需的特性
fbc-starter = { version = "0.1.5", features = ["postgres", "redis", "nacos", "grpc", "kafka"] }
```

### 基础使用

使用新的闭包配置方式：

```rust
use fbc_starter::Server;
use axum::{routing::get, Router};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    Server::run(|builder| {
        // 创建 HTTP 路由
        let routes = Router::new()
            .route("/api/hello", get(|| async { "Hello, World!" }));

        builder.http_router(routes)
    })
    .await
}
```

### gRPC 服务集成

```rust
use fbc_starter::Server;
use tonic::{transport::Server as TonicServer, Request, Response, Status};

// 定义 gRPC 服务
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    Server::run(|builder| {
        // 配置 HTTP 路由
        let http_routes = axum::Router::new()
            .route("/", axum::routing::get(|| async { "HTTP + gRPC Server" }));

        // 配置 gRPC 服务
        #[cfg(feature = "grpc")]
        let builder = builder.grpc_router(
            tonic::transport::Server::builder()
                .add_service(your_grpc_service)
        );

        builder.http_router(http_routes)
    })
    .await
}
```

### Kafka 消息处理

```rust
use fbc_starter::{Server, KafkaMessageHandler, Message};
use async_trait::async_trait;
use std::sync::Arc;

// 定义消息处理器
struct MyMessageHandler;

#[async_trait]
impl KafkaMessageHandler for MyMessageHandler {
    fn topics(&self) -> Vec<String> {
        vec!["my-topic".to_string()]
    }

    async fn handle(&self, message: Message) {
        tracing::info!("收到消息: topic={}, data={:?}", message.topic, message.data);
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    Server::run(|builder| {
        let routes = axum::Router::new()
            .route("/", axum::routing::get(|| async { "Kafka Server" }));

        #[cfg(feature = "consumer")]
        let builder = builder.with_kafka_handler(Arc::new(MyMessageHandler));

        builder.http_router(routes)
    })
    .await
}
```

## 📖 配置

### 环境变量配置

创建 `.env` 文件（参考 `.env.example`）：

```env
# ================================
# 服务器配置
# ================================
APP__SERVER__ADDR=0.0.0.0
APP__SERVER__PORT=3000
# APP__SERVER__CONTEXT_PATH=/api

# ================================
# 日志配置
# ================================
APP__LOG__LEVEL=info
APP__LOG__JSON=false

# ================================
# CORS 配置
# ================================
APP__CORS__ALLOWED_ORIGINS=*
APP__CORS__ALLOWED_METHODS=GET,POST,PUT,DELETE,PATCH,OPTIONS
APP__CORS__ALLOWED_HEADERS=*
APP__CORS__ALLOW_CREDENTIALS=false
```

### 数据库配置（mysql/postgres/sqlite 特性）

```env
# PostgreSQL
APP__DATABASE__URL=postgres://user:password@localhost:5432/dbname
# MySQL
# APP__DATABASE__URL=mysql://user:password@localhost:3306/dbname
# SQLite
# APP__DATABASE__URL=sqlite://data.db

APP__DATABASE__MAX_CONNECTIONS=100
APP__DATABASE__MIN_CONNECTIONS=10
```

### Redis 配置（redis 特性）

```env
APP__REDIS__URL=redis://127.0.0.1:6379
# APP__REDIS__PASSWORD=your_password
APP__REDIS__POOL_SIZE=10
```

### Nacos 配置（nacos 特性）

```env
APP__NACOS__SERVER_ADDRS=127.0.0.1:8848
APP__NACOS__SERVICE_NAME=my-service
APP__NACOS__NAMESPACE=public
APP__NACOS__GROUP_NAME=DEFAULT_GROUP

# 订阅的服务列表（支持多种格式）
# 方式 1: JSON 数组
APP__NACOS__SUBSCRIBE_SERVICES=["im-server","user-service"]
# 方式 2: 逗号分隔
# APP__NACOS__SUBSCRIBE_SERVICES=im-server,user-service
```

### Kafka 配置（kafka 特性）

```env
APP__KAFKA__BROKERS=localhost:9092

# Producer 配置（producer 特性）
APP__KAFKA__PRODUCER__RETRIES=3
APP__KAFKA__PRODUCER__ENABLE_IDEMPOTENCE=true
APP__KAFKA__PRODUCER__ACKS=all

# Consumer 配置（consumer 特性）
APP__KAFKA__CONSUMER__GROUP_ID=my-consumer-group
APP__KAFKA__CONSUMER__ENABLE_AUTO_COMMIT=true
# 订阅的主题列表
APP__KAFKA__CONSUMER__TOPICS=["topic1","topic2"]
```

## 🔧 特性说明

### 核心特性

- `default`: 无额外特性，仅包含基础 HTTP 服务
- `grpc`: 启用 gRPC 服务支持（基于 Tonic）
- `mysql` / `postgres` / `sqlite`: 启用对应数据库支持（SQLx）
- `redis`: 启用 Redis 缓存支持
- `nacos`: 启用 Nacos 服务注册与发现
- `balance`: 启用负载均衡（依赖 nacos 特性）
- `kafka`: 启用 Kafka 基础支持
- `producer`: 启用 Kafka 生产者（依赖 kafka 特性）
- `consumer`: 启用 Kafka 消费者（依赖 kafka 特性）

### 组合使用

```toml
# 完整功能
[dependencies]
fbc-starter = { version = "0.1.5", features = ["postgres", "redis", "grpc", "nacos", "balance", "producer", "consumer"] }

# 仅 HTTP + 数据库（示例使用 postgres，可按需改为 mysql/sqlite）
[dependencies]
fbc-starter = { version = "0.1.5", features = ["postgres"] }

# HTTP + gRPC
[dependencies]
fbc-starter = { version = "0.1.5", features = ["grpc"] }

# 微服务（服务发现 + 负载均衡 + 消息队列）
[dependencies]
fbc-starter = { version = "0.1.5", features = ["nacos", "balance", "producer", "consumer"] }
```

## 📚 使用示例

### 访问共享状态

```rust
use fbc_starter::Server;
use axum::{routing::get, Router, Extension};
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    Server::run(|builder| {
        let app_state = builder.app_state().clone();

        let routes = Router::new()
            .route("/db", get(db_handler))
            .with_state(app_state);

        builder.http_router(routes)
    })
    .await
}

async fn db_handler(
    axum::extract::State(state): axum::extract::State<Arc<fbc_starter::AppState>>
) -> &'static str {
    #[cfg(feature = "database")]
    if let Ok(pool) = state.database() {
        // 使用数据库连接池
    }

    "OK"
}
```

### 使用缓存键构建器

```rust
use fbc_starter::cache::{CacheKeyBuilder, SimpleCacheKeyBuilder, ValueType};

let builder = SimpleCacheKeyBuilder::new("user")
    .with_prefix("dev")
    .with_tenant("0000")
    .with_modular("authority")
    .with_field("id")
    .with_value_type(ValueType::Obj);

let cache_key = builder.key(&[&1u64]);
// 生成: "dev:0000:authority:user:id:obj:1"
```

### 使用 Kafka 发送消息

```rust
use fbc_starter::{Server, MessageProducer};
use axum::{routing::post, Router, Json, extract::State};
use std::sync::Arc;

async fn send_message(
    State(state): State<Arc<fbc_starter::AppState>>,
    Json(payload): Json<serde_json::Value>,
) -> Result<String, String> {
    #[cfg(feature = "producer")]
    if let Ok(producer) = state.message_producer() {
        producer.send("my-topic", payload)
            .await
            .map_err(|e| e.to_string())?;
        return Ok("Message sent".to_string());
    }

    Err("Producer not available".to_string())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    Server::run(|builder| {
        let app_state = builder.app_state().clone();
        let routes = Router::new()
            .route("/send", post(send_message))
            .with_state(app_state);

        builder.http_router(routes)
    })
    .await
}
```

### 使用 gRPC 客户端（负载均衡）

```rust
#[cfg(feature = "balance")]
use fbc_starter::get_load_balancer;

// 获取服务的负载均衡器
let lb = get_load_balancer("im-server");

// 获取一个实例的 endpoint
if let Some(endpoint) = lb.next_endpoint() {
    // 创建 gRPC 客户端
    let channel = tonic::transport::Channel::from_shared(endpoint)?
        .connect()
        .await?;

    let mut client = YourGrpcClient::new(channel);
    let response = client.your_method(request).await?;
}
```

## 🎯 API 端点

### 系统端点

- `GET /` - 欢迎信息和版本信息

  ```json
  {
    "name": "fbc-starter",
    "version": "0.1.5",
    "status": "running"
  }
  ```

- `GET /health` - 健康检查（符合 Prometheus 标准）
  ```json
  {
    "status": "healthy"
  }
  ```

## 🏗️ 项目结构建议

```
your-project/
├── Cargo.toml
├── .env                    # 环境配置（不提交到 Git）
├── .env.example            # 配置示例
├── src/
│   ├── main.rs            # 应用入口
│   ├── handlers/          # HTTP 处理器
│   │   ├── mod.rs
│   │   └── user.rs
│   ├── grpc/              # gRPC 服务定义
│   │   ├── mod.rs
│   │   └── service.rs
│   ├── kafka/             # Kafka 消息处理器
│   │   ├── mod.rs
│   │   └── handlers.rs
│   └── model/             # 数据模型
│       ├── mod.rs
│       ├── dto/
│       ├── vo/
│       └── entity/
└── proto/                 # gRPC proto 文件
    └── service.proto
```

## 📝 完整示例

查看示例项目：

- [im-server](../im-server) - gRPC 健康检查服务
- [ws-server](../ws-server) - WebSocket + Kafka 消息服务

## 🔍 配置项详细说明

### 服务器配置

| 配置项                      | 说明       | 默认值    | 示例        |
| --------------------------- | ---------- | --------- | ----------- |
| `APP__SERVER__ADDR`         | 监听地址   | `0.0.0.0` | `127.0.0.1` |
| `APP__SERVER__PORT`         | 监听端口   | `3000`    | `8080`      |
| `APP__SERVER__CONTEXT_PATH` | 上下文路径 | 无        | `/api`      |

### 日志配置

| 配置项            | 说明      | 默认值  | 可选值                                    |
| ----------------- | --------- | ------- | ----------------------------------------- |
| `APP__LOG__LEVEL` | 日志级别  | `info`  | `trace`, `debug`, `info`, `warn`, `error` |
| `APP__LOG__JSON`  | JSON 格式 | `false` | `true`, `false`                           |

### Nacos 配置

| 配置项                           | 说明                         | 默认值                 |
| -------------------------------- | ---------------------------- | ---------------------- |
| `APP__NACOS__SERVER_ADDRS`       | Nacos 服务器地址（逗号分隔） | -                      |
| `APP__NACOS__SERVICE_NAME`       | 服务名称                     | -                      |
| `APP__NACOS__SERVICE_PORT`       | 服务端口                     | 同 `APP__SERVER__PORT` |
| `APP__NACOS__NAMESPACE`          | 命名空间                     | `public`               |
| `APP__NACOS__GROUP_NAME`         | 分组名称                     | `DEFAULT_GROUP`        |
| `APP__NACOS__WEIGHT`             | 服务权重                     | `1.0`                  |
| `APP__NACOS__SUBSCRIBE_SERVICES` | 订阅的服务列表               | `[]`                   |

### Kafka 配置

| 配置项                                     | 说明              | 默认值           |
| ------------------------------------------ | ----------------- | ---------------- |
| `APP__KAFKA__BROKERS`                      | Kafka Broker 地址 | `localhost:9092` |
| `APP__KAFKA__PRODUCER__RETRIES`            | 生产者重试次数    | `3`              |
| `APP__KAFKA__PRODUCER__ENABLE_IDEMPOTENCE` | 启用幂等性        | `true`           |
| `APP__KAFKA__PRODUCER__ACKS`               | ACK 策略          | `all`            |
| `APP__KAFKA__CONSUMER__GROUP_ID`           | 消费者组 ID       | -                |
| `APP__KAFKA__CONSUMER__ENABLE_AUTO_COMMIT` | 自动提交 offset   | `true`           |
| `APP__KAFKA__CONSUMER__TOPICS`             | 订阅的主题列表    | `[]`             |

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

## 📄 许可证

本项目采用 MIT 或 Apache-2.0 双许可证。

## 🙏 致谢

- [Axum](https://github.com/tokio-rs/axum) - 高性能 Web 框架
- [Tonic](https://github.com/hyperium/tonic) - gRPC 框架
- [Tokio](https://tokio.rs/) - 异步运行时
- [Tracing](https://github.com/tokio-rs/tracing) - 结构化日志
- [SQLx](https://github.com/launchbadge/sqlx) - 异步 SQL 工具包
- [Nacos Rust Client](https://github.com/nacos-group/nacos-sdk-rust) - Nacos 服务发现客户端
- [rdkafka](https://github.com/fede1024/rust-rdkafka) - Kafka 客户端
