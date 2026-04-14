# ms-identity-admin Docker 镜像（Node 构建 + Nginx 运行）
# 构建: docker compose build（在 ms-identity-admin 目录下）
# syntax=docker/dockerfile:1

# ===== Stage 1: 构建 =====
FROM node:20-alpine AS builder
WORKDIR /app

# 安装 pnpm
RUN corepack enable && corepack prepare pnpm@latest --activate

# 先复制依赖文件，利用缓存
COPY package.json pnpm-lock.yaml ./
RUN pnpm install --frozen-lockfile

# 复制源码并构建
COPY . .
RUN pnpm vite build

# ===== Stage 2: Nginx 运行 =====
FROM nginx:alpine

# 复制构建产物
COPY --from=builder /app/dist /usr/share/nginx/html

# 复制 Nginx 配置
COPY nginx.conf /etc/nginx/conf.d/default.conf

EXPOSE 80

CMD ["nginx", "-g", "daemon off;"]
