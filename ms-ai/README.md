# ms-ai — AI 代理应用微服务

> Xilulu 平台对接大语言模型的业务网关，探索生成式 AI 在企业级应用中的集成能力。

**🔜 当前状态：开发规划中**

---

## 📋 目录

- [定位与愿景](#-定位与愿景)
- [技术栈](#️-技术栈)
- [数据模型](#-数据模型)
- [项目结构](#-项目结构)
- [快速开始](#-快速开始)
- [发展计划](#-发展计划)

---

## 🎯 定位与愿景

本模块是 Xilulu 生态系统通向 AI 世界的统一代理层，核心职责包括：

- 🤖 **多模型对接** — 无缝接入 OpenAI、Anthropic、Google Gemini 及开源本地大模型
- 💬 **AI 对话** — 具有语境记忆的智能聊天助手
- 📝 **AI 写作** — 智能创作、润色、翻译等文本生成能力
- 🖼️ **AI 绘图** — 文生图、图生图等图像生成
- 🎵 **AI 音乐** — AI 音乐创作与编曲
- 🎬 **AI 视频** — 文本/图片生成视频
- 🧠 **知识库** — 文档向量化存储与知识问答 (RAG)
- 🗺️ **思维导图** — AI 辅助思维导图生成
- 🔧 **工具调用** — 平台内功能的 AI Agent 编排
- 🛡️ **安全过滤** — 敏感内容过滤与合规检查

---

## 🛠️ 技术栈

| 类目 | 技术 | 说明 |
|------|------|------|
| **框架** | fbc-starter (Axum) | HTTP 服务 |
| **数据库** | MySQL 8.0 | sqlx + sqlxplus ORM |
| **缓存** | Redis | 对话缓存/频率限制 |
| **服务发现** | Nacos | 注册/配置中心 |

---

## 📊 数据模型

系统已预设以下实体模型，覆盖 AI 应用的全生命周期：

| 实体 | 表名 | 说明 |
|------|------|------|
| `AiPlatform` | `ai_platform` | AI 平台/提供商配置 |
| `AiModel` | `ai_model` | 模型元信息（名称、能力、计费） |
| `AiApiKey` | `ai_api_key` | API Key 管理（加密存储） |
| `AiChatConversation` | `ai_chat_conversation` | 对话会话记录 |
| `AiChatMessage` | `ai_chat_message` | 对话消息详情 |
| `AiChatRole` | `ai_chat_role` | 角色预设/提示词模板 |
| `AiImage` | `ai_image` | AI 生成图片记录 |
| `AiVideo` | `ai_video` | AI 生成视频记录 |
| `AiMusic` | `ai_music` | AI 音乐创作记录 |
| `AiAudio` | `ai_audio` | AI 语音处理记录 |
| `AiWrite` | `ai_write` | AI 写作任务记录 |
| `AiMindMap` | `ai_mind_map` | AI 思维导图记录 |
| `AiKnowledge` | `ai_knowledge` | 知识库配置 |
| `AiKnowledgeDocument` | `ai_knowledge_document` | 知识库文档 |
| `AiKnowledgeSegment` | `ai_knowledge_segment` | 文档分片/向量化 |
| `AiTool` | `ai_tool` | Agent 工具定义 |
| `AiModelUsageRecord` | `ai_model_usage_record` | 模型使用统计 |

---

## 📁 项目结构

```
ms-ai/
├── src/
│   ├── main.rs              # 入口 — Server::run 启动
│   ├── router.rs            # HTTP 路由定义
│   └── model/               # 数据模型
│       └── entity/          # 实体定义 (17 个实体表)
│           ├── ai_platform.rs
│           ├── ai_model.rs
│           ├── ai_api_key.rs
│           ├── ai_chat_conversation.rs
│           ├── ai_chat_message.rs
│           ├── ai_chat_role.rs
│           ├── ai_image.rs
│           ├── ai_video.rs
│           ├── ai_music.rs
│           ├── ai_audio.rs
│           ├── ai_write.rs
│           ├── ai_mind_map.rs
│           ├── ai_knowledge.rs
│           ├── ai_knowledge_document.rs
│           ├── ai_knowledge_segment.rs
│           ├── ai_tool.rs
│           └── ai_model_usage_record.rs
└── Cargo.toml
```

---

## 🚀 快速开始

```bash
# 目前仅包含实体定义骨架，可直接启动空服务
cargo run -p ms-ai
```

---

## 📅 发展计划

### Phase 1 — 基础对话能力

- [ ] AI 对话 API (SSE 流式响应)
- [ ] 多模型适配层 (OpenAI / Anthropic / 本地模型)
- [ ] 对话历史管理
- [ ] API Key 加密与轮换

### Phase 2 — 多模态生成

- [ ] AI 写作 (文案/翻译/润色)
- [ ] AI 绘图 (文生图)
- [ ] AI 思维导图生成

### Phase 3 — 知识库与 Agent

- [ ] RAG 知识库 (文档上传 → 分片 → 向量化 → 检索)
- [ ] Agent 工具调用编排
- [ ] 平台内搜索集成

### Phase 4 — 高级能力

- [ ] AI 视频/音乐生成
- [ ] 使用量统计与计费
- [ ] 敏感内容过滤与合规

---

## 📄 许可证

MIT OR Apache-2.0
