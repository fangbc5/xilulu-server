# 终极生产级 OSS 微服务架构设计方案

感谢您的深度指导！您指出的这些痛点（重复消费、CPU打爆、签名泄露、TCP阻塞、调度抢占）正是由业务原型向千万级日活平台演进时必踩的“血坑”。

基于您的反馈，我将底层设计彻底推翻重构为 **完全拥抱高可用、防击穿、异步调度的高阶架构**。

---

## 核心架构蓝图与坑位化解方案

### 1. 媒体处理异步管道（彻底解决视频处理痛点）
**原问题**：非幂等导致重复写、FFmpeg 失败引发无限重试死循环、大小任务抢占。
**终极架构**：
* **去重与状态机 (Idempotency & State Machine)**：
  在数据库中增加 `media_task` 表。设计严格的单向状态机：`INIT -> PROCESSING -> DONE / FAILED`。
  Worker 在拿到 Kafka 消息后，要求必须使用带版本号的乐观锁（`UPDATE media_task SET status = 'PROCESSING', version = version + 1 WHERE id = ? AND status = 'INIT'`）。更新失败直接 Drop（证明在被其他机器处理）。唯一联合主键 `UK(file_id, task_type, version)`。
* **重试与死信队列 (DLQ)**：
  配置 `video-process-retry` 和 `video-process-dlq` Topic。若遇到文件损坏、OOM，Worker 抓取异常后将重试次数写回头，重试超过 3 次直接扔进 DLQ 并告警，同时 DB 标记 `FAILED`，杜绝毒药消息阻塞队列引发 CPU 爆炸。
* **调度与分级 (Scheduler & Priority)**：
  将 Kafka Topic 按复杂度拆分：`topic_fast_track`（仅抽首帧）和 `topic_heavy_track`（长视频转码）。
   甚至依据后续需求，可以为独立的 GPU 节点配置专属消费组（Consumer Group），确保小文件不被大文件阻塞调度。

### 2. 动态图片引擎 防击穿设计 (Imgproxy Shield)
**原问题**：无阻拦的动态访问会把 imgproxy 的 CPU 和 MinIO 的 IO 双双打爆。
**终极架构**：`Client -> CDN (公共网络缓存) -> Nginx Cache Node (网卡级缓存) -> imgproxy -> MinIO`
* **CDN 承载**：热点头像、Banner 的访问 99% 应该在边缘节点终结。
* **Nginx Cache**：在 `docker-compose` 架构内，在 `imgproxy` 前必然挂载一个由 Nginx `proxy_cache` 驱动的极速缓存层（Cache-Control）。
* **行为流**：只有当 CDN 和 Nginx 都发生 MISS 时，请求才会被打入 `imgproxy`，进而请求 MinIO 且只请求一次。处理完即层层回写。

### 3. 长效签名的“相对性”与安全置换
**原问题**：1 年有效期的 URL 一旦泄露，意味着永久被白嫖外链流量，无法审计和撤回。
**终极架构**：**短期物理 URL + 实时置换机制**
企业级业务绝对不存储、不暴露底层的真实长期物理链接。
* **数据库侧**：DB 中持久化存储的应当是抽象的虚地址（如 `oss://group1/avatar/xyz.png`）。
* **展示侧（On-the-fly Sign）**：
  在客户端请求 `GET /api/v1/user/info` 时，User Service 调用 OSS Client 库，**实时将 `oss://...` 映射出只有 5~15 分钟有效期的真实 MinIO/CDN 签名 URL** 并返回给前端。
* **分享侧（302 置换）**：
  如果需要在外部网页分享一个不变更的物理形态链接，提供如 `https://域名/share/v1/file/123` 的 API。该 API 基于 Cookie/Header Auth 控制访问权。验证通过后，**服务端返回 HTTP 302** 并在 Header 的 `Location` 填入刚刚生成的 `5 分钟有效短期签名 URL`。外部下载器跟随跳转。安全性与持久性共存。

### 4. 网关直连的全面废除 (TCP 防耗尽)
**原问题**：如果网关（Gateway）作为代理直接流式地将 MinIO 里的 10GB 数据 Proxy 给用户，网关的连接池瞬间枯竭，整个微服务集群瘫痪。
**终极架构**：**网关仅作 PEP（策略执行点）**
* 网关（或 `ms-oss`）接收到下载或查看请求时，**绝对不接触文件二进制流**。
* 它的指责仅为校验 Token/Cookie/鉴权机制。
* 校验成功后，通过生成 MinIO 预签名链接，回复 `HTTP 302 Redirect`。
* 此后用户的真实的沉重 TCP 长连接直接打向 MinIO 集群的节点或 CDN 节点，网关在几毫秒内毫发无损地功成身退。

---

## 落地演进路线 (Action Plan)

考虑到企业级强度的引入，如果我们在本代码库上实施，我建议按照如下优先级在 `xilulu-server` 中稳步落地：

### Phase 1：重构鉴权与下载流 (目前最急需)
- [ ] 撤销所有业务对 MinIO 的直接流式代理（如果有）。
- [ ] 建立基于 302 Redirect 和 10 分钟短签名的核心分发链路 (`ms-oss` 返回短签 302)。

### Phase 2：防击穿的图片基建上线
- [ ] 调整 `docker-compose.yml`，串联 `Nginx (proxy_cache)` -> `imgproxy`。
- [ ] `ms-oss` 封装统一的转换方法，业务代码请求缩略图时，不再是 `ms-oss` 处理，而是直接生成指向 `Nginx -> imgproxy` 的可信短签名签名 URL。

### Phase 3：视频重载 Worker 的异步管道 (最高难度)
- [ ] 在数据库增加 `media_tasks` 幂等结构。
- [ ] 在 `fbc-starter` 中配置重试与 DLQ 的 Kafka Topic。
- [ ] 编写一个挂载有 ffmpeg 环境的 Rust Worker Task，利用乐观锁安全消耗队列并写入衍生存储。

## 您的确认

这份方案全方位避开了您提到的所有生产级雷区。由于这个整改涉及到跨组件协同（新服务、网关设计变更、数据库幂等结构），您希望我**先专注于 Phase 1（重构建鉴权与短签302分发）**和实现最基础的逻辑，还是先从某个具体痛点（比如在 docker-compose 把 imgproxy 和 Nginx 缓存层搭起来）开始给您输出代码？
