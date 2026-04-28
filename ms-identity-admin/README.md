# ms-identity-admin — 管理后台 (前端)

> 配合微服务接口的运营管理仪表盘，基于 Vue 3 + Vite + TypeScript + Naive UI 构建。

## 📋 目录

- [功能概述](#-功能概述)
- [技术栈](#️-技术栈)
- [项目结构](#-项目结构)
- [快速开始](#-快速开始)
- [环境变量](#️-环境变量)
- [Docker 部署](#-docker-部署)
- [与后端服务的关系](#-与后端服务的关系)

---

## ✨ 功能概述

- 👥 **用户管理** — 用户列表、创建、编辑、删除、角色分配
- 🏠 **租户管理** — 租户 CRUD、应用关联管理
- 🔐 **权限管理** — 角色/资源/应用管理界面
- 🏢 **组织管理** — 组织架构、部门、岗位管理
- 📊 **数据看板** — 系统概览与运营数据统计
- ⚙️ **系统配置** — 运营参数配置界面

---

## 🛠️ 技术栈

| 类目 | 技术 | 说明 |
|------|------|------|
| **框架** | Vue 3 | 渐进式 JavaScript 框架 |
| **构建工具** | Vite | 下一代前端构建工具 |
| **类型系统** | TypeScript | 类型安全 |
| **UI 组件库** | Naive UI | Vue 3 组件库 |
| **状态管理** | Pinia | Vue 状态管理 |
| **路由** | Vue Router | SPA 路由管理 |
| **HTTP 客户端** | Axios | API 请求 |
| **CSS 框架** | Tailwind CSS | 实用优先的 CSS |

---

## 📁 项目结构

```
ms-identity-admin/
├── src/
│   ├── api/              # 📡 API 接口封装
│   ├── components/       # 🧩 通用组件
│   ├── layout/           # 📐 布局组件
│   ├── pages/            # 📄 页面组件
│   ├── router/           # 🔀 路由配置
│   ├── store/            # 📦 Pinia 状态管理
│   ├── utils/            # 🔧 工具函数
│   ├── types/            # 📝 TypeScript 类型定义
│   ├── App.vue           # 根组件
│   └── main.ts           # 应用入口
├── .env.example          # 环境变量模板
├── Dockerfile            # Docker 构建 (Nginx)
├── package.json          # 依赖配置
└── vite.config.ts        # Vite 配置
```

---

## 🚀 快速开始

### 前置条件

- Node.js 18+
- npm 或 yarn

### 安装与运行

```bash
# 进入目录
cd ms-identity-admin

# 安装依赖
npm install

# 启动开发服务器
npm run dev

# 构建生产版本
npm run build

# 预览生产构建
npm run preview
```

---

## ⚙️ 环境变量

复制 `.env.example` 为 `.env` 并配置：

| 变量 | 默认值 | 说明 |
|------|--------|------|
| `VITE_PROXY_URL` | `http://localhost:30002` | 后端 API 代理地址 |
| `VITE_PORT` | `5174` | 开发服务器端口 |
| `VITE_BASE_URL` | `/` | 基础路径 |

---

## 🐳 Docker 部署

### 构建与运行

```bash
# 独立构建
cd ms-identity-admin
docker build -t ms-identity-admin:latest .

# 或通过 docker-compose（在 xilulu-server 根目录）
docker compose up -d --build ms-identity-admin
```

### 部署说明

- 使用 Nginx 静态托管
- Docker 容器映射端口 `5174:80`
- 通过 `.env.docker` 配置后端 API 地址

---

## 🔗 与后端服务的关系

| 服务 | 协议 | 关系说明 |
|------|------|----------|
| **ms-auth** | HTTP | 管理员登录认证 |
| **ms-identity** | HTTP | 用户/租户/权限管理 API |
| **ms-team** | HTTP | 组织架构管理 API |

---

## 📄 许可证

MIT OR Apache-2.0
