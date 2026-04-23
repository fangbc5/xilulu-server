# Plan — 对标阿里云 OSS 的 RESTful API 设计方案

## 一、API 总览

资源路径 = `/{bucket}/{object_key}`，HTTP 动词 = 操作类型，与阿里云 OSS / AWS S3 保持一致。

| HTTP 方法 | 路由 | 说明 |
|-----------|------|------|
| `POST` | `/oss/signature/upload` | 获取上传预签名 URL |
| `POST` | `/oss/signature/download` | 获取下载预签名 URL |
| `POST` | `/oss/signature/share` | 获取长效分享链接（JWT，无有效期上限） |
| `GET` | `/oss/share/{token}` | 长效分享链接访问入口（302 重定向） |
| | | |
| `PUT` | `/oss/{bucket}/{*key}` | **PutObject** — 预签名上传 |
| `POST` | `/oss/{bucket}/{*key}` | **PostObject** — 上传完成确认 |
| `POST` | `/oss/{bucket}/{*key}?uploads` | **InitiateMultipartUpload** — 初始化分片上传 |
| `POST` | `/oss/{bucket}/{*key}?uploadId=` | **CompleteMultipartUpload** — 完成分片上传 |
| `GET` | `/oss/{bucket}/{*key}` | **GetObject** — 下载原文件（302 → S3） |
| `GET` | `/oss/{bucket}/{*key}?x-oss-process=image/...` | **GetObject** — 图片实时处理（302 → imgproxy） |
| `GET` | `/oss/{bucket}/{*key}?x-oss-process=video/...` | **GetObject** — 视频截帧产物查询（302） |
| `GET` | `/oss/{bucket}/{*key}?x-oss-process=style/...` | **GetObject** — Style 预设处理（302） |
| `GET` | `/oss/{bucket}/{*key}?uploadId=` | **ListParts** — 查询已上传分片（断点续传） |
| `HEAD` | `/oss/{bucket}/{*key}` | **HeadObject** — 获取文件元数据 |
| `DELETE` | `/oss/{bucket}/{*key}` | **DeleteObject** — 删除文件 |
| `DELETE` | `/oss/{bucket}/{*key}?uploadId=` | **AbortMultipartUpload** — 取消分片上传 |

---

## 二、完整 API 规范

### 2.0 Signature — 统一签名服务

**对标阿里云**：STS 临时凭证 + 服务端签名直传 + 长效分享链接

阿里云 OSS 中，客户端需要签名才能操作私有 Bucket。我们的 `ms-oss` 扮演的是签名颁发中心的角色。
本接口是一个**独立的签名获取端点**，统一服务于上传签名、下载签名、长效分享链接三种场景。

```
POST /oss/signature
```

#### 2.0.1 获取上传签名

**请求体**：

```json
{
    "method": "put",
    "bucket": "public",
    "key": "avatar/2026/04/uuid.jpg",
    "content_type": "image/jpeg",
    "scene": "avatar",
    "size": 102400
}
```

**ms-oss 内部行为**：
1. 校验 scene 规则（文件类型白名单、大小限制）
2. 调用 `OssProvider::presign_put` 生成 S3 预签名 URL（默认 1 小时有效）
3. 异步写入 `file_meta` 审计记录

**响应** `200 OK`：

```json
{
    "code": 0,
    "data": {
        "url": "http://rustfs:9000/public/avatar/...?X-Amz-Signature=...",
        "object_key": "avatar/2026/04/uuid.jpg",
        "method": "PUT",
        "expires_in": 3600
    }
}
```

#### 2.0.2 获取下载签名

**请求体**：

```json
{
    "method": "get",
    "bucket": "chat-media",
    "key": "chat_file/2026/04/report.pdf",
    "expires_in": 600
}
```

**ms-oss 内部行为**：
1. 校验调用者权限（Token）
2. 调用 `OssProvider::presign_get`，生成短效 S3 预签名 URL（默认 10 分钟）

**响应** `200 OK`：

```json
{
    "code": 0,
    "data": {
        "url": "http://rustfs:9000/chat-media/chat_file/...?X-Amz-Signature=...",
        "method": "GET",
        "expires_in": 600
    }
}
```

#### 2.0.3 获取长效分享链接

**对标阿里云**：突破 S3 预签名 7 天上限，通过业务 JWT 实现任意有效期的分享。

**请求体**：

```json
{
    "method": "share",
    "bucket": "public",
    "key": "docs/guide.pdf",
    "expires_in": 31536000,
    "x_oss_process": "image/resize,m_fill,w_300,h_300"
}
```

| 字段 | 说明 |
|------|------|
| `expires_in` | 有效期（秒），无上限（如 3 年 = 94608000，5 年 = 157680000） |
| `x_oss_process` | 可选，绑定在分享链接中的处理参数（防止外部篡改加工内容） |

**ms-oss 内部行为**：
1. 将 `bucket`、`key`、`expires_at`、`x_oss_process` 打包为 JWT Payload
2. 使用服务端 Secret 签名
3. 返回长效访问 URL

**响应** `200 OK`：

```json
{
    "code": 0,
    "data": {
        "url": "https://domain/oss/share/eyJhbGciOiJIUzI1NiJ9...",
        "expires_in": 31536000
    }
}
```

**长效链接访问流程**：

```
GET /oss/share/{jwt_token}
```

1. `ms-oss` 解码并验证 JWT（校验签名、有效期）
2. 从 JWT 中提取 `bucket`/`key`/`x_oss_process`
3. 根据内容走标准的 GetObject 302 分发流程（原文件 → S3 签名 / 图片 → imgproxy）
4. 若 JWT 无效或过期 → 返回 `403 Forbidden`

> **设计要点**：长效链接的 URL 中不暴露任何 bucket/key 信息（全部封装在 JWT 里），
> 外部用户无法从 URL 反推存储路径，也无法篡改处理参数。

---

### 2.1 PutObject — 预签名上传

**对标阿里云**: `PUT /{ObjectName}`

```
PUT /oss/{bucket}/{*key}
```

**请求头**：

| Header | 必须 | 说明 |
|--------|-----|------|
| `Content-Type` | 是 | 文件 MIME 类型 |
| `x-oss-meta-original-name` | 否 | 原始文件名 |
| `x-oss-meta-scene` | 是 | 业务场景（`avatar`/`chat_image`/`chat_video` 等） |
| `Content-Length` | 否 | 文件大小（字节），用于预校验 |

**ms-oss 内部行为**：
1. 校验 scene 规则（文件类型、大小限制）
2. 调用 `OssProvider::presign_put` 生成预签名 URL
3. 异步写入 `file_meta` 审计记录
4. 返回签名信息

**响应** `200 OK`：

```json
{
    "code": 0,
    "data": {
        "upload_url": "http://rustfs:9000/bucket/key?X-Amz-Signature=...",
        "object_key": "avatar/2026/04/uuid.jpg",
        "expires_in": 3600
    }
}
```

> **PutObject vs Signature(put) 的区别**：
> - `PUT /oss/{bucket}/{key}` 适用于客户端已知完整路径的场景（REST 语义更纯粹）
> - `POST /oss/signature` 适用于客户端只知 scene + 文件名，由服务端生成路径的场景（更灵活）
> - 两者内部逻辑相同，属于同一能力的两种入口形式

---

### 2.2 PostObject — 上传完成确认 / 分片上传

**对标阿里云**: `POST /{ObjectName}?uploads` / Callback 机制

```
POST /oss/{bucket}/{*key}
```

根据 Query 参数区分行为：

#### 2.2.1 上传完成确认（无特殊 Query）

```
POST /oss/{bucket}/{*key}
```

**ms-oss 内部行为**：
1. 根据 `{bucket}/{key}` 调用 `OssProvider::head_object` 验证文件确实存在于存储中
2. 对比文件大小是否一致
3. 异步更新 `file_meta` 审计记录（`status=1`）
4. 检测 `content_type`，若为视频类型则向 Kafka 发送 `sys.media.task.submit` 触发异步处理

**响应** `200 OK`：

```json
{
    "code": 0,
    "data": {
        "bucket": "public",
        "key": "avatar/2026/04/uuid.jpg",
        "content_type": "image/jpeg",
        "size": 102400
    }
}
```

#### 2.2.2 InitiateMultipartUpload — 初始化分片上传

**对标阿里云**: `POST /{ObjectName}?uploads`

适用于大文件（>5MB）的分片上传场景，如视频、大文档等。

```
POST /oss/{bucket}/{*key}?uploads
```

**请求头**：

| Header | 必须 | 说明 |
|--------|-----|------|
| `Content-Type` | 是 | 最终文件的 MIME 类型 |
| `x-oss-meta-scene` | 是 | 业务场景 |
| `x-oss-meta-original-name` | 否 | 原始文件名 |
| `x-oss-meta-total-size` | 否 | 文件总大小（字节） |
| `x-oss-meta-part-size` | 否 | 每个分片的大小（字节，默认 5MB） |

**ms-oss 内部行为**：
1. 校验 scene 规则
2. 调用底层 S3 `CreateMultipartUpload` 获取 `upload_id`
3. 异步写入 `file_meta` 审计记录
4. 根据 `total_size` 和 `part_size` 计算分片数量
5. 为每个分片批量生成 `presign_put` URL

**响应** `200 OK`：

```json
{
    "code": 0,
    "data": {
        "upload_id": "2~abc123def456...",
        "object_key": "chat_video/2026/04/big-video.mp4",
        "part_count": 6,
        "part_urls": [
            { "part_number": 1, "upload_url": "http://rustfs:9000/...?partNumber=1&uploadId=2~abc123...&X-Amz-Signature=..." },
            { "part_number": 2, "upload_url": "http://rustfs:9000/...?partNumber=2&uploadId=2~abc123...&X-Amz-Signature=..." },
            { "part_number": 3, "upload_url": "http://rustfs:9000/...?partNumber=3&..." },
            { "part_number": 4, "upload_url": "http://..." },
            { "part_number": 5, "upload_url": "http://..." },
            { "part_number": 6, "upload_url": "http://..." }
        ],
        "expires_in": 7200
    }
}
```

**前端使用流程**：
```javascript
// 1. 初始化分片上传
const { upload_id, part_urls } = await api.post(
    `/oss/${bucket}/${key}?uploads`,
    { headers: { 'x-oss-meta-scene': 'chat_video', 'x-oss-meta-total-size': file.size } }
)

// 2. 并发上传每个分片（客户端自行切片）
const partSize = 5 * 1024 * 1024 // 5MB
const parts = []
for (const { part_number, upload_url } of part_urls) {
    const start = (part_number - 1) * partSize
    const end = Math.min(start + partSize, file.size)
    const blob = file.slice(start, end)
    const resp = await fetch(upload_url, { method: 'PUT', body: blob })
    parts.push({ part_number, etag: resp.headers.get('ETag') })
}

// 3. 完成分片上传
await api.post(`/oss/${bucket}/${key}?uploadId=${upload_id}`, { parts })
```

---

#### 2.2.3 CompleteMultipartUpload — 完成分片上传

**对标阿里云**: `POST /{ObjectName}?uploadId=xxx`

```
POST /oss/{bucket}/{*key}?uploadId=2~abc123def456
```

**请求体**：

```json
{
    "parts": [
        { "part_number": 1, "etag": "\"a54357...\"" },
        { "part_number": 2, "etag": "\"b68912...\"" },
        { "part_number": 3, "etag": "\"c23456...\"" }
    ]
}
```

**ms-oss 内部行为**：
1. 调用底层 S3 `CompleteMultipartUpload`（传入 parts 列表）
2. 调用 `head_object` 获取最终文件大小
3. 异步更新 `file_meta` 审计记录
4. 若为视频类型 → 向 Kafka 发送 `sys.media.task.submit`

**响应** `200 OK`：

```json
{
    "code": 0,
    "data": {
        "bucket": "chat-media",
        "key": "chat_video/2026/04/big-video.mp4",
        "content_type": "video/mp4",
        "size": 31457280
    }
}
```

---

#### 2.2.4 AbortMultipartUpload — 取消分片上传

**对标阿里云**: `DELETE /{ObjectName}?uploadId=xxx`

当上传中途取消或失败时，必须调用此接口清理已上传的分片碎片，释放存储空间。

```
DELETE /oss/{bucket}/{*key}?uploadId=2~abc123def456
```

**ms-oss 内部行为**：
1. 调用底层 S3 `AbortMultipartUpload`
2. 异步更新 `file_meta` 审计记录

**响应** `204 No Content`

---

#### 2.2.5 ListParts — 查询已上传分片

**对标阿里云**: `GET /{ObjectName}?uploadId=xxx`

断点续传场景：客户端上传中途断开后恢复时，先查询哪些分片已上传成功，避免重复上传。

```
GET /oss/{bucket}/{*key}?uploadId=2~abc123def456
```

**ms-oss 内部行为**：
1. 调用底层 S3 `ListParts`
2. 返回已成功上传的分片列表

**响应** `200 OK`：

```json
{
    "code": 0,
    "data": {
        "upload_id": "2~abc123def456",
        "parts": [
            { "part_number": 1, "etag": "\"a54357...\"", "size": 5242880 },
            { "part_number": 2, "etag": "\"b68912...\"", "size": 5242880 }
        ],
        "next_part_number": 3
    }
}
```

**断点续传前端实现**：
```javascript
// 恢复上传：先查已完成的分片
const { parts: doneParts } = await api.get(`/oss/${bucket}/${key}?uploadId=${upload_id}`)
const doneSet = new Set(doneParts.map(p => p.part_number))

// 只上传未完成的分片
for (const { part_number, upload_url } of part_urls) {
    if (doneSet.has(part_number)) continue  // 跳过已完成
    // ... 上传逻辑
}
```

---

### 2.3 GetObject — 下载 / 图片处理 / 视频产物查询

**对标阿里云**: `GET /{ObjectName}` + `?x-oss-process=...`

```
GET /oss/{bucket}/{*key}
```

**这是整个系统的核心分发引擎。**

#### 场景 A：原文件下载（无 `x-oss-process`）

```
GET /oss/public/docs/report.pdf
```

**响应** `302 Found`：

```
HTTP/1.1 302 Found
Location: http://rustfs:9000/public/docs/report.pdf?X-Amz-Signature=...&X-Amz-Expires=600
```

ms-oss 耗时：< 1ms（纯签名计算，无 DB 查询）

#### 场景 B：图片实时处理（`x-oss-process=image/...`）

```
GET /oss/public/avatar/user1.jpg?x-oss-process=image/resize,m_fill,w_128,h_128/format,webp
```

**ms-oss 内部流程**：
1. 解析 `x-oss-process` → `image/resize,m_fill,w_128,h_128/format,webp`
2. 翻译为 imgproxy 指令 → `rs:fill:128:128`，格式后缀 `@webp`
3. 拼装 imgproxy 路径 → `/rs:fill:128:128/plain/s3://public/avatar/user1.jpg@webp`
4. HMAC-SHA256 签名 → `/{signature}/rs:fill:128:128/plain/s3://...@webp`
5. 组装完整 URL → `http://nginx-cdn:8085/{signature}/rs:fill:128:128/plain/s3://...@webp`

**响应** `302 Found`：

```
HTTP/1.1 302 Found
Location: http://nginx-cdn:8085/{signature}/rs:fill:128:128/plain/s3://public/avatar/user1.jpg@webp
```

ms-oss 耗时：< 1ms（纯计算，无 DB 查询）

#### 场景 C：视频截帧产物查询（`x-oss-process=video/...`）

```
GET /oss/chat-media/chat_video/2026/04/vid.mp4?x-oss-process=video/snapshot,t_0
```

**ms-oss 内部流程**：
1. 解析 `x-oss-process` → `video/snapshot,t_0`
2. 查 DB `file_meta`（通过 `file_key` + `bucket`）获取 `thumbnail_key`
3. 若 `thumbnail_key` 存在 → 为其生成 presigned GET URL → `302`
4. 若 `thumbnail_key` 为空（处理中/失败）→ 返回占位图 URL 或 `404`

#### 场景 D：Style 预设

```
GET /oss/public/avatar/user1.jpg?x-oss-process=style/avatar_small
```

**ms-oss 内部流程**：
1. 在内存 Style 映射表中查找 `avatar_small` → `image/resize,m_fill,w_64,h_64/format,webp`
2. 展开后走场景 B 的逻辑

---

### 2.4 HeadObject — 获取元数据

**对标阿里云**: `HEAD /{ObjectName}`

```
HEAD /oss/{bucket}/{*key}
```

**响应** `200 OK`（无 Body，信息在 Header 中）：

```
HTTP/1.1 200 OK
Content-Type: image/jpeg
Content-Length: 102400
x-oss-meta-original-name: my-avatar.jpg
x-oss-meta-scene: avatar
x-oss-meta-uploader-id: 5001
x-oss-meta-thumbnail-key: _derivative/avatar_thumb.jpg
Last-Modified: Wed, 23 Apr 2026 08:30:00 GMT
```

**ms-oss 内部行为**：查 DB `file_meta`，将字段映射为 `x-oss-meta-*` 响应头。

---

### 2.5 DeleteObject — 删除文件

**对标阿里云**: `DELETE /{ObjectName}`

```
DELETE /oss/{bucket}/{*key}
```

**ms-oss 内部行为**：
1. 调用 `OssProvider::delete_object` 删除存储中的真实文件
2. 查 DB，若存在关联的 `thumbnail_key`，同时删除缩略图
3. 异步更新 `file_meta` 审计记录

**响应** `204 No Content`（对标阿里云，无论文件是否存在均返回 204）

---

## 三、`x-oss-process` 参数翻译对照表

### 3.1 图片处理 `image/...`

#### resize 缩放

| 阿里云语法 | 含义 | imgproxy 翻译 |
|-----------|------|-------------|
| `resize,m_lfit,w_300,h_200` | 等比限宽高 | `rs:fit:300:200` |
| `resize,m_fill,w_300,h_200` | 等比裁剪填充 | `rs:fill:300:200` |
| `resize,m_fixed,w_300,h_200` | 强制拉伸 | `rs:force:300:200` |
| `resize,m_mfit,w_300,h_200` | 等比缩放至少一边 | `rs:fill-down:300:200` |
| `resize,w_300` | 仅宽 | `rs:fit:300:0` |
| `resize,h_200` | 仅高 | `rs:fit:0:200` |
| `resize,p_50` | 百分比 | `rs:fit:50p:50p` |

#### crop 裁剪

| 阿里云语法 | 含义 | imgproxy 翻译 |
|-----------|------|-------------|
| `crop,w_300,h_200,g_center` | 中心裁剪 | `c:300:200:ce` |
| `crop,w_300,h_200,g_nw` | 左上裁剪 | `c:300:200:nowe` |
| `crop,w_300,h_200,x_10,y_20` | 坐标裁剪 | `c:300:200:nowe:10:20` |

#### quality 质量

| 阿里云语法 | imgproxy 翻译 |
|-----------|-------------|
| `quality,q_90` | `q:90` |
| `quality,Q_85` | `q:85` |

#### format 格式

| 阿里云语法 | imgproxy 翻译 |
|-----------|-------------|
| `format,jpg` | 后缀 `@jpg` |
| `format,png` | 后缀 `@png` |
| `format,webp` | 后缀 `@webp` |

#### 管道示例

```
阿里云:    ?x-oss-process=image/resize,m_fill,w_300,h_300/quality,q_85/format,webp
imgproxy:  /{sign}/rs:fill:300:300/q:85/plain/s3://{bucket}/{key}@webp
```

### 3.2 视频处理 `video/...`

| 阿里云语法 | 含义 | ms-oss 行为 |
|-----------|------|-----------|
| `video/snapshot,t_1000` | 截取第1秒帧 | 查 DB thumbnail_key → 302 |
| `video/snapshot,t_0,f_jpg` | 首帧 jpg | 同上 |
| `video/snapshot,t_1000,w_800,h_600` | 截帧+缩放 | 查到 thumbnail_key → 通过 imgproxy 二次 resize → 302 |

### 3.3 Style 预设 `style/...`

| 阿里云语法 | 展开为 |
|-----------|-------|
| `style/avatar_small` | `image/resize,m_fill,w_64,h_64/format,webp` |
| `style/avatar_medium` | `image/resize,m_fill,w_128,h_128/format,webp` |
| `style/avatar_large` | `image/resize,m_fill,w_256,h_256/format,webp` |
| `style/chat_thumb` | `image/resize,m_lfit,w_480/quality,q_85` |
| `style/chat_preview` | `image/resize,m_lfit,w_1200/quality,q_90` |
| `style/video_cover` | `video/snapshot,t_0,f_jpg` |

---

## 四、前端使用对比

### 旧方案（繁琐）

```javascript
// 1. 先调一个接口获取预签名
const { upload_url } = await api.post('/api/v1/oss/presign/upload', { ... })
// 2. 上传文件
await fetch(upload_url, { method: 'PUT', body: file })
// 3. 再调一个接口确认
await api.post('/api/v1/oss/callback', { ... })
// 4. 展示下载链接
const { download_url } = await api.post('/api/v1/oss/presign/download', { object_key })
```

### 新方案（简洁如阿里云）

```javascript
// 1. 获取签名（语义清晰：PUT 就是要上传）
const { upload_url } = await api.put(`/oss/${bucket}/${key}`, {
    headers: { 'Content-Type': 'image/jpeg', 'x-oss-meta-scene': 'avatar' }
})
// 2. 上传文件
await fetch(upload_url, { method: 'PUT', body: file })
// 3. 确认上传（POST 到同一路径）
await api.post(`/oss/${bucket}/${key}`)

// 展示图片 —— 直接用路径，浏览器自动跟随302
<img src="/oss/public/avatar/2026/04/uuid.jpg?x-oss-process=style/avatar_small" />
```

---

## 五、路由注册表（Rust axum 视角）

```rust
Router::new()
    // 签名服务
    .route("/oss/signature", post(create_signature))       // 统一签名
    .route("/oss/share/{token}", get(share_redirect))      // 长效分享 302 入口
    // 对象操作（RESTful 核心）
    .route("/oss/{bucket}/*key", put(put_object))           // PutObject
    .route("/oss/{bucket}/*key", post(post_object))         // PostObject (确认/分片初始化/分片完成)
    .route("/oss/{bucket}/*key", get(get_object))           // GetObject (下载/处理/ListParts)
    .route("/oss/{bucket}/*key", head(head_object))         // HeadObject
    .route("/oss/{bucket}/*key", delete(delete_object))     // DeleteObject / AbortMultipart
```

**Query 参数路由决策一览**：

| HTTP方法 | Query 参数 | 行为 |
|---------|-----------|------|
| `POST` | 无 | 上传完成确认 |
| `POST` | `?uploads` | 初始化分片上传 |
| `POST` | `?uploadId=xxx` | 完成分片上传 |
| `GET` | 无 / `?x-oss-process=` | 下载 / 图片处理 |
| `GET` | `?uploadId=xxx` | 查询已上传分片 (ListParts) |
| `DELETE` | 无 | 删除文件 |
| `DELETE` | `?uploadId=xxx` | 取消分片上传 |

> 注意：不再使用 `/api/v1` 前缀，直接用 `/oss/` 前缀，与阿里云 OSS 的路径风格保持一致。

---

## 六、代码架构变更

### 6.1 新增文件

| 文件 | 职责 |
|------|------|
| `src/utils/mod.rs` | 工具模块根 |
| `src/utils/imgproxy.rs` | imgproxy HMAC-SHA256 签名 URL 生成器 |
| `src/utils/oss_process.rs` | `x-oss-process` 参数解析与 imgproxy 翻译引擎 |

### 6.2 修改文件

| 文件 | 变更内容 |
|------|---------|
| `src/router.rs` | **重写**：废弃旧路由，注册 5 个 RESTful 端点 |
| `src/modules/file/handler.rs` | **重写**：实现 `put_object`、`post_object`、`get_object`、`head_object`、`delete_object` |
| `src/modules/file/service.rs` | **重构**：按新 handler 对应新增服务方法 |
| `src/config.rs` | 新增 `imgproxy_key`/`salt`/`base_url`/`styles`/`video_placeholder_url` |
| `src/error.rs` | 新增分发错误码 4510~4513 |
| `Cargo.toml` | 新增 `hmac`、`sha2` 依赖 |

### 6.3 gRPC 接口

gRPC 服务（`grpc.rs`）面向**内部微服务间调用**，保留不变。
外部客户端只通过 HTTP RESTful 接口访问。

---

## 七、安全模型

### 7.1 Bucket 级别访问控制

| Bucket | 策略 | GET 行为 | PUT/POST/DELETE 行为 |
|--------|------|---------|-------------------|
| 名称包含 `public` / 配置为公开 | Public Read | 无需Token直接302 | 需 Token |
| 其他 | Private | 需 Token | 需 Token |

### 7.2 三层签名保护

```
客户端
  │ Token（sa-token/JWT）
  ▼
ms-oss  ──────→  302 Location 携带两种签名之一:
  │
  ├─ 原文件下载:   S3 Pre-signed URL (X-Amz-Signature, 10分钟)
  └─ 图片处理:     imgproxy HMAC-SHA256 签名 (防篡改)
                        │
                        ▼
                   nginx-cdn → imgproxy → rustfs
```

---

## 八、性能矩阵

| 操作 | 查DB | ms-oss耗时 | 计算节点 |
|------|------|-----------|---------|
| `POST /oss/signature` (put) | ✅ 写入1条 | < 5ms | 无 |
| `POST /oss/signature` (get) | ❌ | < 1ms | 无 |
| `POST /oss/signature` (share) | ❌ | < 1ms（JWT签名） | 无 |
| `GET /oss/share/{jwt}` | ❌ | < 1ms（JWT验签） | 同展开后场景 |
| `PUT` (预签名) | ✅ 写入1条 | < 5ms | 无 |
| `POST` (确认) | ✅ 读+更新 | < 5ms | 无 |
| `POST ?uploads` (初始化分片) | ✅ 写入1条 | < 10ms（批量签名） | 无 |
| `POST ?uploadId=` (完成分片) | ✅ 读+更新 | < 10ms | 无 |
| `GET ?uploadId=` (查分片) | ❌ | < 5ms | 无 |
| `DELETE ?uploadId=` (取消分片) | ✅ 更新1条 | < 5ms | 无 |
| `GET` 原文件 | ❌ | < 1ms | 无（直连rustfs） |
| `GET` image/* | ❌ | < 1ms | imgproxy (Nginx缓存) |
| `GET` video/* | ✅ 查1条 | < 5ms | 无（读取预处理产物） |
| `GET` style/* | ❌ | < 1ms | 同展开后场景 |
| `HEAD` | ✅ 查1条 | < 5ms | 无 |
| `DELETE` | ✅ 读+更新 | < 5ms | 无 |

---

## 九、实施分期

### Phase 1：RESTful 核心引擎 + 签名服务
- [x] 新建 `utils/imgproxy.rs`：HMAC-SHA256 签名算法
- [x] 新建 `utils/oss_process.rs`：解析器 + 翻译器
- [x] 新建 `utils/jwt.rs`：长效分享链接的 JWT 签发与验证
- [x] 重写 `router.rs`：注册 RESTful 端点 + 签名端点 + 分享端点
- [x] 重写 `handler.rs`：`create_signature`/`share_redirect`/`put_object`/`post_object`/`get_object`/`head_object`/`delete_object`
- [x] 重构 `service.rs`：适配新 handler
- [x] 修改 `config.rs`：imgproxy 配置 + Style 预设 + JWT Secret
- [x] 修改 `error.rs`：新增错误码
- [x] 修改 `Cargo.toml`：引入 `hmac`/`sha2`/`jsonwebtoken`

### Phase 2：分片上传完整支持
- [x] `post_object` 中 `?uploads` 分支：初始化分片 + 批量 presign
- [x] `post_object` 中 `?uploadId=` 分支：完成分片合并
- [x] `get_object` 中 `?uploadId=` 分支：查询已上传分片（ListParts）
- [x] `delete_object` 中 `?uploadId=` 分支：取消分片上传（AbortMultipart）
- [x] 扩展 `OssProvider` trait：新增 `create_multipart`/`complete_multipart`/`abort_multipart`/`list_parts`/`presign_upload_part`

### Phase 3：视频异步流整合
- [x] `post_object` 确认逻辑中检测视频 → 发送 Kafka 任务
- [x] `get_object` 中 `video/*` 场景的 DB 查询 + 302 逻辑
- [x] 视频封面 + imgproxy 二次处理的串联

### Phase 4：高级功能
- [ ] Style 系统数据库化 + 管理接口
- [ ] Bucket 级别的 CORS 策略配置
- [ ] 文件版本控制（对标阿里云 Versioning）
