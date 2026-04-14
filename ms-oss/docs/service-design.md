# ms-oss 服务规划文档

## 1. 服务定位

ms-oss 是 HuLa 微服务体系中的**文件中台服务**，提供统一的文件存储抽象层。

### 核心原则

- **后端只签名，前端直传** — 文件流不经过微服务，避免带宽和内存瓶颈
- **多厂商透明切换** — 通过 `OssProvider` trait 适配不同 OSS 厂商
- **统一文件处理** — 水印、缩略图、格式转换等在一处管控

---

## 2. 系统架构

```
                      ① 请求预签名 URL
        前端/APP ─────────────────────→ ms-oss ──→ 数据库（元数据）
           │                              │
           │  ② 返回预签名 URL             │  Provider 适配层
           │←─────────────────────────────│
           │                              ↓
           │  ③ 直传文件          ┌────────┼────────┐
           │──────────────────→  │        │        │
           │                   RustFS  阿里OSS  腾讯COS
           │
           │  ④ 上传完成回调
           │──────────────────→ ms-oss ──→ 触发文件处理（水印等）
```

### 与其他微服务的关系

| 微服务 | 交互方式 | 场景 |
|--------|----------|------|
| ms-im | HTTP/gRPC | 聊天文件/图片上传签名 |
| ms-identity | HTTP/gRPC | 用户头像上传签名 |
| ms-ai | HTTP/gRPC | AI 生成内容存储 |
| ms-team | HTTP/gRPC | 组织 Logo 等资源 |
| ms-gateway | 反向代理 | 统一入口 |

---

## 3. 模块划分

```
ms-oss/
├── src/
│   ├── main.rs               # 入口
│   ├── config.rs             # OssConfig（Provider、凭据、Bucket）
│   ├── error.rs              # OssError 错误类型
│   ├── state.rs              # OssState（Provider 实例、DB 连接）
│   ├── router.rs             # HTTP 路由注册
│   ├── provider/             # 多厂商适配层
│   │   ├── mod.rs            # OssProvider trait
│   │   ├── s3_compat.rs      # S3 兼容实现（RustFS/MinIO/AWS）
│   │   ├── aliyun.rs         # 阿里云 OSS 原生 SDK（计划中）
│   │   └── tencent.rs        # 腾讯 COS 原生 SDK（计划中）
│   ├── handler/              # HTTP Handler 层
│   │   ├── mod.rs
│   │   ├── presign.rs        # 预签名 URL 签发
│   │   ├── callback.rs       # 上传完成回调
│   │   └── metadata.rs       # 文件元数据 CRUD
│   ├── service/              # 业务逻辑层
│   │   ├── mod.rs
│   │   ├── sign_service.rs   # 签名逻辑（路径策略、权限校验）
│   │   ├── file_service.rs   # 文件元数据管理
│   │   └── process_service.rs# 文件后处理（水印、缩略图）
│   ├── repository/           # 数据访问层
│   │   ├── mod.rs
│   │   └── file_meta_repo.rs # file_meta 表 CRUD
│   └── model/                # 数据模型
│       ├── entity/           # 数据库实体
│       └── dto/              # 请求/响应 DTO
├── docs/                     # 文档
├── .env                      # 环境配置
├── .aiproject/               # AI 开发规范
└── Cargo.toml
```

---

## 4. API 规划

### 4.1 预签名接口

```
POST /oss/presign/upload
```
请求：
```json
{
  "bucket": "hula-chat",
  "filename": "photo.jpg",
  "content_type": "image/jpeg",
  "scene": "chat_image"
}
```
响应：
```json
{
  "code": 0,
  "data": {
    "upload_url": "https://oss.example.com/hula-chat/2026/03/uuid.jpg?X-Amz-...",
    "object_key": "2026/03/uuid.jpg",
    "expires_in": 3600
  }
}
```

```
POST /oss/presign/download
```
请求：
```json
{
  "bucket": "hula-chat",
  "object_key": "2026/03/uuid.jpg"
}
```
响应：
```json
{
  "code": 0,
  "data": {
    "download_url": "https://oss.example.com/hula-chat/2026/03/uuid.jpg?X-Amz-...",
    "expires_in": 3600
  }
}
```

### 4.2 上传回调

```
POST /oss/callback
```
上传完成后前端通知后端，触发元数据入库和文件后处理。

### 4.3 文件元数据

```
GET    /oss/files/:id         # 查询文件信息
DELETE /oss/files/:id         # 删除文件
GET    /oss/files?scene=xxx   # 按场景查询文件列表
```

---

## 5. 多厂商适配策略

### Provider Trait

```rust
#[async_trait]
pub trait OssProvider: Send + Sync {
    async fn presign_put(&self, bucket, key, content_type, expires) -> Result<PresignedUrl>;
    async fn presign_get(&self, bucket, key, expires) -> Result<PresignedUrl>;
    async fn delete_object(&self, bucket, key) -> Result<()>;
    async fn head_object(&self, bucket, key) -> Result<ObjectMeta>;
}
```

### 厂商支持计划

| 厂商 | 实现方式 | 状态 |
|------|---------|------|
| RustFS / MinIO | S3 兼容（aws-sdk-s3） | ✅ 已实现 |
| AWS S3 | S3 兼容（aws-sdk-s3） | ✅ 已实现 |
| 阿里云 OSS | S3 兼容模式 | 📋 计划中 |
| 腾讯 COS | S3 兼容模式 | 📋 计划中 |

阿里云 OSS 和腾讯 COS 均支持 S3 兼容模式，只需通过 `S3CompatProvider` 配置不同的 endpoint 即可，无需单独实现。

---

## 6. 文件路径策略

按场景自动生成 Object Key：

```
{scene}/{year}/{month}/{uuid}.{ext}

示例：
chat_image/2026/03/550e8400-e29b-41d4-a716-446655440000.jpg
avatar/2026/03/550e8400-e29b-41d4-a716-446655440001.png
ai_content/2026/03/550e8400-e29b-41d4-a716-446655440002.mp3
```

---

## 7. Bucket 规划

| Bucket | 用途 | 访问策略 |
|--------|------|----------|
| `wula-avatar` | 用户头像 | Public Read |
| `wula-chat` | IM 聊天文件 | Private（预签名） |
| `wula-ai` | AI 生成内容 | Private |
| `wula-public` | 公共静态资源 | Public Read |

---

## 8. 文件处理规划（Phase 2）

| 功能 | 触发方式 | 说明 |
|------|---------|------|
| 图片水印 | 上传回调后异步处理 | 支持文字/图片水印 |
| 缩略图生成 | 上传回调后异步处理 | 多尺寸缩略图 |
| PDF 水印 | 上传回调后异步处理 | 文字水印 |
| 文件格式校验 | 签名前校验 | 白名单机制 |
| 文件大小限制 | 签名前校验 | 按场景配置 |
