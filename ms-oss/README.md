# ms-oss

统一文件中台微服务 — HuLa 微服务体系的对象存储抽象层。

## 功能特性

- 🔐 **预签名 URL 签发** — 后端只做签名，前端直传 OSS，不经过微服务
- 🔌 **多厂商适配** — 通过 `OssProvider` trait 统一 RustFS / MinIO / AWS S3 / 阿里云 OSS / 腾讯 COS
- 🖼️ **统一文件处理** — 水印、缩略图、格式校验一处管控
- 📋 **文件元数据管理** — 记录文件信息，支持按场景查询

## 技术栈

- **框架**: [fbc-starter](https://github.com/fangbc5/fbc-starter) (Axum)
- **S3 SDK**: aws-sdk-s3（兼容所有 S3 协议厂商）
- **数据库**: MySQL（通过 sqlx + sqlxplus）
- **缓存**: Redis
- **注册中心**: Nacos

## 快速开始

### 前置条件

- Rust 1.80+
- MySQL 8.0+
- Redis 7+
- RustFS / MinIO（或其他 S3 兼容存储）

### 配置

复制并修改 `.env` 文件：

```bash
# 核心 OSS 配置
OSS__PROVIDER=rustfs
OSS__ENDPOINT=http://127.0.0.1:9000
OSS__ACCESS_KEY=rustfsadmin
OSS__SECRET_KEY=rustfsadmin
OSS__DEFAULT_BUCKET=hula
```

### 运行

```bash
cargo run --package ms-oss
```

## 项目结构

```
ms-oss/
├── src/
│   ├── main.rs          # 入口（Server::run 启动）
│   ├── config.rs        # OssConfig
│   ├── error.rs         # OssError
│   ├── state.rs         # OssState
│   ├── router.rs        # HTTP 路由
│   └── provider/        # 多厂商适配层
│       ├── mod.rs       # OssProvider trait
│       └── s3_compat.rs # S3 兼容实现
├── docs/
│   └── service-design.md # 服务规划文档
├── .aiproject/          # AI 开发规范（P0-P9）
├── .env                 # 环境配置
└── Cargo.toml
```

## API 概览

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/oss/presign/upload` | 获取预签名上传 URL |
| POST | `/oss/presign/download` | 获取预签名下载 URL |
| POST | `/oss/callback` | 上传完成回调 |
| GET | `/oss/files/:id` | 查询文件元数据 |
| DELETE | `/oss/files/:id` | 删除文件 |

> 详细 API 设计见 [docs/service-design.md](docs/service-design.md)

## 架构设计

```
前端 → ms-oss（签名） → 返回预签名 URL → 前端直传 OSS
                  ↓
        OssProvider trait
         ├── S3CompatProvider（RustFS / MinIO / AWS）
         ├── AliyunProvider（计划中）
         └── TencentProvider（计划中）
```

## 开发规范

本项目遵循 fbc-starter 微服务开发标准，详见 `.aiproject/` 目录。

## License

MIT OR Apache-2.0
