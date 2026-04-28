# ms-content

统一内容管理中台微服务 — Xilulu 微服务体系的内容创作与检索核心引擎。

## 功能特性

- 📄 **多内容形态支持** — 统一管理文章、动态、评论等异构内容
- 🧱 **Block DSL 正文** — 采用结构化块级 DSL，适配全平台原生渲染
- 🕸️ **内容关系图谱** — 维护“内容-内容”多维关系（评论树、引用、收藏关联）
- 🔍 **全文搜索引擎** — 基于 Meilisearch 提供毫秒级全文检索与聚合分析
- 📑 ** OpenAPI 文档集成** — 内置 Utoipa + Swagger UI 开发者文档

## 技术栈

- **框架**: `fbc-starter` (Tokio + Axum)
- **数据库**: MySQL 8.0 (sqlx + sqlxplus ORM)
- **缓存**: Redis (排行榜、热点数据)
- **搜索引擎**: Meilisearch
- **发现中心**: Nacos

## 快速开始

在 `ms-content` 目录下，复制环境变量模板：
```bash
cp .env.example .env
```
执行编译和运行：
```bash
cargo run --package ms-content
```

## API 文档

服务启动后，可以通过访问 `http://localhost:30106/swagger-ui` (或你映射的部署端口) 查询全套基于 OpenAPI 的可视化交互式接口文档。
