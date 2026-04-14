# ms-team - 企业级组织管理服务

ms-team 是 hula-server 微服务架构中的组织管理模块，负责企业组织架构管理、人员信息管理、部门管理等功能。该模块是企业级应用的核心组件，支持多租户、高并发、高可用等特性。

## 架构概述

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Frontend      │    │   API Gateway   │    │  ms-team│
│                 │◄──►│                 │◄──►│                 │
└─────────────────┘    └─────────────────┘    │  • Organization │
                                              │  • Department   │
┌─────────────────┐    ┌─────────────────┐    │  • Position     │
│   ms-identity   │    │   ms-notify     │    │  • Employee     │
│                 │◄──►│                 │◄──►│                 │
│  • User Auth    │    │  • Notifications│    │  • Cache       │
│  • RBAC         │    │                 │    │  • Events      │
└─────────────────┘    └─────────────────┘    └─────────────────┘
                             ▲
                             │
                    ┌─────────────────┐
                    │    Kafka        │
                    │                 │
                    │  • Event Bus    │
                    └─────────────────┘
```

## 核心功能

### 1. 组织管理
- 组织创建、编辑、删除
- 组织树结构展示
- 组织类型管理

### 2. 部门管理
- 部门创建、编辑、删除
- 部门树结构展示
- 部门负责人设置
- 部门员工统计

### 3. 岗位管理
- 岗位创建、编辑、删除
- 岗位分类管理
- 岗位层级管理

### 4. 员工管理
- 员工信息管理
- 员工与部门、岗位关系管理
- 员工状态管理
- 员工权限范围查询

### 5. 权限与安全
- 多租户数据隔离
- 基于Casbin的权限控制
- 数据权限范围控制
- 操作审计日志

## 技术栈

- **语言**: Rust
- **Web框架**: Axum + Tokio (异步高性能)
- **数据库**: MySQL (事务支持、数据一致性)
- **缓存**: Redis (高性能缓存、会话存储)
- **RPC框架**: gRPC (高效内部服务通信)
- **消息队列**: Kafka (事件驱动架构)
- **权限框架**: 与 ms-identity 集成的 Casbin 权限控制
- **序列化**: Protobuf + JSON
- **日志追踪**: Tracing + 分布式链路追踪

## 快速开始

### 环境准备

1. 安装 Rust
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup default stable
```

2. 启动依赖服务
```bash
# 启动 MySQL, Redis, Kafka, Nacos
docker-compose up -d
```

### 构建和运行

1. 构建项目
```bash
cd /path/to/hula-server
cargo build -p ms-team
```

2. 运行服务
```bash
cargo run -p ms-team
```

3. 或使用环境变量运行
```bash
RUST_LOG=debug cargo run -p ms-team
```

## API 接口

服务提供 RESTful API 和 gRPC 接口两种访问方式：

### RESTful API
- 基础路径: `/api/v1`
- 认证: JWT Token
- 数据格式: JSON

### gRPC 接口
- 服务名称: `OrganizationService`
- 数据格式: Protobuf
- 认证: 通过请求头传递

详细 API 接口规范请参见 [API_SPECIFICATION.md](./docs/API_SPECIFICATION.md)

## 配置说明

服务支持以下环境变量配置：

- `DATABASE_URL`: 数据库连接字符串
- `REDIS_URL`: Redis 连接地址
- `KAFKA_BOOTSTRAP_SERVERS`: Kafka 服务器地址
- `NACOS_ADDR`: Nacos 服务器地址
- `SERVER_PORT`: 服务端口，默认 8080
- `GRPC_PORT`: gRPC 端口，默认 50051

## 文档

- [企业级设计文档](./docs/ENTERPRISE_DESIGN.md): 详细的企业级设计说明
- [开发计划](./docs/DEVELOPMENT_PLAN_DETAIL.md): 详细的开发计划和任务分解
- [API 接口规范](./docs/API_SPECIFICATION.md): RESTful API 接口详细说明
- [gRPC 接口定义](./docs/gRPC_INTERFACE_DEFINITION.md): gRPC 接口详细定义

## 部署

### Docker 部署

```Dockerfile
FROM rust:1.70 as builder

WORKDIR /app
COPY . .
RUN cargo build --release -p ms-team

FROM debian:buster-slim
RUN apt-get update && apt-get install -y ca-certificates
COPY --from=builder /app/target/release/ms-team /usr/local/bin/
CMD ["ms-team"]
```

### Kubernetes 部署

参考 [k8s-deployment.yaml](./k8s-deployment.yaml) 文件进行 Kubernetes 部署。

## 贡献

欢迎提交 Issue 和 Pull Request 来帮助我们改进 ms-team 模块。

## 许可证

本项目遵循 [MIT 许可证](../../LICENSE)。