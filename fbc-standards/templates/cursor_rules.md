# fbc-starter 微服务开发规范

你正在开发一个基于 fbc-starter 框架的 Rust 微服务。
在生成、修改或审查代码时，你**必须严格遵守**本项目 `.aiproject/` 目录下的规范体系。

## 规范文件索引

请在编码前阅读以下文件（P0 最高优先级，P9 最低）：

- `.aiproject/README.md` — 规范总览 + 场景速查 + 检查清单
- `.aiproject/P0-product.md` — 依赖管理、项目结构、启动模式（**必须遵守**）
- `.aiproject/P1-architecture.md` — 分层架构、数据层、配置管理
- `.aiproject/P2-code-style.md` — 命名规范、导入顺序、文档注释
- `.aiproject/P3-api.md` — 响应格式、错误码体系、gRPC、Kafka
- `.aiproject/P4-security.md` — 安全实践、HTTP 安全、数据安全
- `.aiproject/P5-testing.md` — 测试策略、分层测试
- `.aiproject/P6-deploy.md` — Docker 容器化部署、CI/CD
- `.aiproject/P7-observability.md` — 日志规范、链路追踪
- `.aiproject/P8-performance.md` — 性能优化、缓存策略
- `.aiproject/P9-ops.md` — 健康检查、数据库迁移、运维

## 核心规则

1. `Server::run` 启动，禁止手动初始化
2. `Handler → Service → Repository` 单向调用
3. `R<T>` 包装所有 HTTP 响应，`AppError` 工厂方法
4. `sqlxplus` 实体宏，字段 `Option<T>`
5. `CacheKeyBuilder` 构建缓存键，禁止手动拼接
6. `tracing` 日志，禁止 `println!`
7. `fbc-builder` + `scratch` 部署（禁止 `debian-slim` / `distroless`）
8. 中文注释
