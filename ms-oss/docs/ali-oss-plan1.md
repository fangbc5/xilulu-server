我想增强我的ms-oss服务，希望其能够成为像阿里云oss一样的服务。能够处理上传的图片/视频缩略图能力，能对图片进行裁剪 。还有就是能够像阿里云oss一样获取长期的下载签名，请给我一套合理的企业级生产设计

# 企业级 OSS 服务架构增强方案

针对您提出的需求：
1. **动态图片处理**（缩略图、裁剪）
2. **视频处理**（截取首帧视频缩略图）
3. **长期下载签名**（突破 S3/MinIO 原生 V4 签名最长 7 天的限制）

基于 `fbc-starter` 和当前 `ms-oss` 的架构，我为您设计了如下 **核心原理上对标阿里云 OSS** 的企业级生产环境实现方案。

## 架构核心思想

阿里云 OSS 的 `x-oss-process` 是动态且极具扩展性的。在基于 MinIO/S3 兼容存储的后端系统中，原生存储服务并不支持这种动态图像处理。因此，我们需要将 `ms-oss` 作为一个**智能代理网关层（Smart Proxy Layer）**：
依靠 `ms-oss` 去接管外部的下载请求，利用 **JWT业务签名代理 + HTTP 302 临时重定向机制 + JIT(Just-In-Time) 按需处理** 来完美解决这些问题，既能实现高级功能，又不会使 Rust 服务的网络出口带宽成为下载瓶颈。

---

## Proposed Changes

### 1. 动态图片与视频处理 (x-oss-process)
**策略：按需处理 (JIT) 结合衍生文件缓存**
我们将实现一个专用的下载路由，可以接收类似 `?x-oss-process=image/resize,w_200/crop,w_200,h_200` 或 `video/snapshot,t_1` 的参数。

* **流程设计**：
  1. 请求到达 `ms-oss` 下载接口，解析 `x-oss-process` 参数。
  2. 根据原文件名及处理参数，计算出一个衍生文件 Key（例如：`_derivative/user/avatar.jpg@resize_w200_crop_w200_h200.jpg`）。
  3. `ms-oss` 向 MinIO 查询该衍生文件是否存在。
  4. **如果存在**：直接生成该衍生文件的*短期* MinIO 原生下载 URL，通过 HTTP 302 重定向给客户端。（流量由 MinIO 直接承载）。
  5. **如果不存在**：
     * 从 MinIO 下载原文件到 `ms-oss` 内存或临时目录。
     * **图片场景**：调用 Rust 原生的 `image` crate 进行缩放、裁剪处理。
     * **视频场景**：通过 `std::process::Command` 调用系统 `ffmpeg` 提取指定时间的视频帧。
     * 将处理后的结果字节流作为一个新的 Object 上传到 MinIO 的 `_derivative/` 路径下，做永久或定期缓存。
     * 生成新文件的短期 URL，HTTP 302 返回给客户端。

### 2. 长期安全的下载签名 (Long-term URL)
**策略：业务 JWT 验签 + 临时 S3 签名转换**
AWS S3 协议底层硬性限制了预签名 URL 最高只有 7 天的有效期。企业级做法是签发自己的**业务长期 Token**。

* **流程设计**：
  1. 新增 API `POST /api/v1/oss/signature/long-term`。请求包含要分享的 `bucket`、`key` 和期望的过期时间（例如半年、1年）。
  2. `ms-oss` 生成一个包含这些信息的 JWT Token，由服务端的 Secret 签名保证防篡改。
  3. 分享给客户端的链接变更为：`https://您的域名/api/v1/oss/download/{bucket}/{key}?token=<JWT>`。
  4. 任意外部用户访问该链接，`ms-oss` 后端利用 Secret 校验 JWT Token 是否合法且未过期。
  5. 若校验通过，`ms-oss` 对 MinIO 实时生成一个只有 **10 分钟有效期的原生预签名 URL**。
  6. 通过 HTTP 302 将用户引导向这个 MinIO 真实临时地址下载文件。

---

### 需要修改的文件模块

---

#### [MODIFY] ms-oss/Cargo.toml
* 增加 `image = "0.24"` 及其相关 features 依赖，用于纯 Rust 实现的高效图片缩放裁剪处理。
* 增加 `jsonwebtoken = "9.3"`，用于长期签名 Token 的签发与验证。

#### [MODIFY] ms-oss/Dockerfile
* 在容器构建时，注入 `RUN apt-get update && apt-get install -y ffmpeg`。这是针对视频抽取首帧的最佳企业级实践，比在 Rust 内嵌复杂的 c-bindings 更加安全和易于维护。

#### [NEW] ms-oss/src/modules/file/process.rs (重新实现原有虚构的逻辑)
* 实现 `ImageProcessor` 结构体：负责解析 `image/resize,w_100,h_100,m_fill` 和 `image/crop` 的处理链。
* 实现 `VideoProcessor` 结构体：负责调用 OS 级别的 `ffmpeg -i <input> -ss <time> -vframes 1 <output>` 获取缩略图首帧。

#### [MODIFY] ms-oss/src/modules/file/handler.rs & router.rs
* `GET /api/v1/oss/download/:bucket/*key`：核心的代理与处理分发网关端点
* `POST /api/v1/oss/signature/long-term`：获取长期签名的端点

#### [MODIFY] ms-oss/src/modules/file/service.rs
* 新增处理逻辑：`generate_long_term_signature` 签发 JWT
* 新增处理逻辑：`handle_download_proxy` 处理下载、验签、x-oss-process 检查、临时重定向核心逻辑。

## User Review Required

> [!IMPORTANT]
> 1. **视频处理要求**：由于视频抽帧需要用到 `ffmpeg`，我们需要修改您的 `ms-oss` 的 Dockerfile，把系统层的 `ffmpeg` 库打包进镜像。您目前的镜像基础环境支持此操作吗？（默认基于 Debian/Ubuntu Slim 是一定支持的）。
> 2. **存储结构规范**：对于所有由原图处理生成的附加产物（如缩略图），我计划将其上传回 OSS 的以 `_derivative/` 开头的不可见目录中作为二级缓存，避免每次访问重复计算，这在 OSS 级海量文件下是被认可的。

请确认此实施方案，如果方案符合您的企业级要求，请点击 Approve，我将立刻开始对相关代码进行重构与开发。
