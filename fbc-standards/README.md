# fbc-standards — fbc-starter 微服务开发规范分发工具

> 为使用 fbc-starter 框架的微服务项目一键初始化 AI 编码工具的开发规范。

## 快速使用

```bash
# 1. 克隆本项目
git clone https://github.com/fangbc5/fbc-standards.git /tmp/fbc-standards

# 2. 在你的微服务项目中运行初始化脚本
bash /tmp/fbc-standards/init.sh /path/to/your/ms-xxx

# 3. 交互式选择你使用的 AI 编码工具
# 脚本会自动生成 .aiproject/ 规范 + 对应 AI 工具的配置文件
```

## 支持的 AI 工具

| # | 工具 | 生成的文件 |
|---|------|-----------|
| 1 | Cursor | `.cursorrules` + `.cursor/rules/fbc-starter.md` |
| 2 | GitHub Copilot | `.github/copilot-instructions.md` |
| 3 | Gemini | `.gemini/settings.json` + `.gemini/styleguide.md` |
| 4 | Claude Code | `CLAUDE.md` |
| 5 | Windsurf | `.windsurfrules` |
| 6 | Cline | `.clinerules` |
| 7 | Antigravity | `.agents/rules/` |

## 规范结构 (P0–P9)

```
.aiproject/
├── README.md            # 规范总览 · 场景速查 · 检查清单
├── P0-product.md        # 项目基础 · 依赖管理 · 项目结构 · 启动模式
├── P1-architecture.md   # 分层架构 · 数据层 · 配置管理
├── P2-code-style.md     # 命名规范 · 代码风格 · 文档注释
├── P3-api.md            # 响应格式 · 错误码 · gRPC · Kafka
├── P4-security.md       # 安全实践 · HTTP 安全 · 数据安全
├── P5-testing.md        # 测试策略 · 分层测试 · 测试规范
├── P6-deploy.md         # Docker 容器化 · CI/CD · 环境管理
├── P7-observability.md  # 日志规范 · 结构化日志 · 链路追踪
├── P8-performance.md    # 性能优化 · 缓存策略 · 连接复用
└── P9-ops.md            # 健康检查 · 数据库迁移 · 运维实践
```

## 规范更新

当 `.aiproject/` 中的规范更新后，只需在目标项目中重新运行 `init.sh`（选择覆盖即可）。
AI 工具的规则文件只是引用 `.aiproject/` 的指向文件，无需额外更新。
