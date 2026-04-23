# Xilulu Server

Xilulu Server 是一款基于 Rust 语言编写的高性能、分层架构微服务后端工程。本项目提供了一整套微服务治理系统，核心业务涵盖即时通讯 (IM)、身份认证、团队管理、通知下发及 AI 接口代理等模块。系统底层基于 `Tokio` 异步运行时与 `Axum` Web 框架，并依靠自研的 `fbc-starter` 基础核心组件提供微服务治理与底层支撑。

## 🛠️ 核心技术栈

*   **编程语言**: Rust
*   **异步运行时**: Tokio
*   **Web 框架**: Axum
*   **数据库框架**: sqlx + sqlxplus (原生异步支持与宏能力增强)
*   **微服务中间件**: fbc-starter（自研基础框架，提供依赖注入与服务治理）
*   **通信协议**: gRPC (内部服务间高效调用), WebSocket (客户端实时推送)
*   **基础设施与中间件**: Nacos (服务注册与统一配置中心), MySQL 8, Redis (分布式缓存), Kafka (异步消息队列)
*   **容器化与部署**: Docker, Docker Compose

## 📦 微服务架构与模块划分

项目采用 Cargo Virtual Workspace (虚拟工作区) 模式管理，核心分为基础框架层与业务微服务层：

### ⚙️ 基础框架层
*   **`fbc-starter`**：微服务底层基建，提供了完整的依赖注入支持、结构化的业务错误处理 (`AppError`)、统一的结构化 HTTP 响应封装 (`R<T>`)、Nacos 服务注册/发现与配置下发逻辑、数据库池和 Redis 缓存控制机制等。

### 🧩 业务微服务层

各微服务在结构逻辑上保持相互独立，由统一的网关或内部 gRPC 调用配合协同。

| 模块目录 | 映射端口 | 服务描述说明 |
| :--- | :--- | :--- |
| **`ms-auth`** | 30002 | **认证鉴权服务**: 负责登录与注册的统一入口、用户身份基本校验、全局会话 Token 管理派发机制。 |
| **`ms-identity`** | 30001 | **身份中心服务**: 统一的数字身份与账户库、角色权限体系(RBAC)管理、多租户划分及密码安全规范校验。 |
| **`ms-im`** | 30102 | **即时通讯服务**: IM 核心业务支撑。涉及好友申请审批、聊天消息队列持久化、回话历史维护与同步。 |
| **`ms-team`** | 30101 | **组织团队服务**: B 端功能的组织架构及团队管理，维护部门结构树以及组织人事数据生命周期。 |
| **`ms-notify`** | 30104 | **消息通知服务**: 处理站外触达(APNs、FCM等 Push 推送)、短信系统接入与邮件群发的网关抽象层。 |
| **`ms-oss`** | 30003 | **对象存储服务**: 提供头像、多媒体资源的上传策略下发以及云端文件存取的统一认证代理机制。 |
| **`ms-media-processor`** | 30105 | **媒体处理服务**: 企业级异步媒体处理中台，支持视频抽帧截图、转码、HLS 自适应码率切片、图片裁剪水印等能力。 |
| **`ms-websocket`** | 30201 | **WebSocket 网关**: 维护客户端的 TCP 长连接，对接消息回推分发流系统，保障消息高频实时送达。 |
| **`ms-ai`** | 规划中 | **AI 代理服务**: 实验性地接入大型语言模型，并开放对接客户端的智能化功能接口与业务集成流。 |
| **`ms-identity-admin`**| 5174 | **管理后台(前端)**: 配合微服务接口的管理人员仪表盘与系统运营配置界面 (静态网页/Nginx环境)。 |

## 🚀 部署与运行指南

本项目高度依赖 Docker 容器化编排实现快速部署：

### 1. 环境前置要求

在进行部署前，你需要确保以下几项已准备就绪：
- 最新版本的 [Docker Engine](https://www.docker.com/) 和 [Docker Compose](https://docs.docker.com/compose/) 安装完毕。
- 全局中间件基础设施已启动并可通过网络访问 (包含 MySQL 8、Redis、Nacos、Kafka 等服务)。
- 若在本地测试基础设施，可以前往 `fbc-starter/docker` 按需拉起配套环境。

### 2. 建立 Docker 外部网络

所有应用容器需要与中间件通信，因此必须确保在宿主机上已创建名称为 `fbc-network` 的 Docker 网桥。
```bash
docker network create fbc-network
```

### 3. 环境配置准备

为所有待运行的业务微服务模块准备各自独立的隔离配置。请参考 `.env.docker.example`，在对应的每个 `ms-*` 模块根目录添加 `.env.docker` 文件并配置相关数据库连接与 Nacos 路由地址等变量。

```bash
# 以 ms-identity 为例进行配置复制：
cp ms-identity/.env.example ms-identity/.env.docker
# 接着使用编辑器补齐其中的网络连接凭据...
```

### 4. 一键构建与发布堆栈（推荐）

回到 `xilulu-server` 项目根目录，执行以下 Compose 命令，可以依靠一份 `docker-compose.yml`将全部服务端一次性启动与组装。

```bash
# 在后台进行无感构建并拉起整个服务端矩阵
docker compose up -d --build

# 仅更新/构建某个特定服务 (比如当只修改了 ms-auth 时)
docker compose up -d --build ms-auth
```

**实用的运维命令参考:**
```bash
# 检查堆栈中各服务的运行状态及端口占用
docker compose ps

# 实时监听特定微服务的标准运行输出日志
docker compose logs -f ms-websocket

# 销毁所有本项目的容器实力及网络连接
docker compose down
```

### 5. 本地开发调试模式

如果在开发阶段，更推荐通过 `cargo` 来单独运行当前正处于调试的服务。前提是请预先在相应模块中添加符合开发环境直连中间件IP 的 `.env`。

```bash
# 启动认证微服务
cargo run -p ms-auth

# 或者配合 cargo-watch 可以做到热重载以提高开发效率
cargo watch -x 'run -p ms-auth'
```

## 📖 核心代码编写规范

本项目具有严谨的架构规范约束，在给本项目提交代码或者撰写新微服务时，务必提前阅读项目根目录下 `.aiproject/` 内的核心准则：
- **严格遵循架构分层**：要求逻辑流向清晰 `Handler(处理路由/Controller)` -> `Service(承接业务逻辑)` -> `Repository(控制持久化/数据库通信)`。
- **数据结构安全**：对客户端的接口响应都必须由 `fbc_starter::R<T>` 进行封箱包装，不得对外裸露散装结构。
- **统一错误工厂**：通过 `fbc_starter::errors::AppError` 下的约定工厂方法来返回处理结果及错误码，不允许滥用通用或系统级报错。
- **禁止本地硬编码**：核心参数项（如各类外站 Token 配置）务必集成至 Nacos 管理台，由基础配置组件统一读取注入使用。

---
`xilulu-server` © 2026. All rights reserved.
