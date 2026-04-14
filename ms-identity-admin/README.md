# MS Identity Admin Console

运营管理后台，基于 Vue 3 + Vite + TypeScript + Naive UI。

## 技术栈

- **Vue 3** - 渐进式 JavaScript 框架
- **Vite** - 下一代前端构建工具
- **TypeScript** - 类型安全的 JavaScript
- **Naive UI** - Vue 3 组件库
- **Pinia** - Vue 状态管理
- **Vue Router** - 路由管理
- **Axios** - HTTP 客户端
- **Tailwind CSS** - 实用优先的 CSS 框架

## 开发

```bash
# 安装依赖
npm install

# 启动开发服务器
npm run dev

# 构建生产版本
npm run build

# 预览生产构建
npm run preview
```

## 项目结构

```
src/
├── api/              # API 接口封装
├── components/       # 通用组件
├── layout/           # 布局组件
├── pages/            # 页面组件
├── router/           # 路由配置
├── store/            # Pinia 状态管理
├── utils/            # 工具函数
├── types/            # TypeScript 类型定义
├── App.vue           # 根组件
└── main.ts           # 应用入口
```

## 环境变量

复制 `.env.example` 为 `.env` 并配置：

- `VITE_PROXY_URL`: API 代理地址（默认：http://localhost:30002）
- `VITE_PORT`: 开发服务器端口（默认：5174）
- `VITE_BASE_URL`: 基础路径（默认：/）

