# fbc-starter 基础中间件 Docker 一键部署

提供 **Redis** 必选，**MySQL 8.0.45** 与 **PostgreSQL 15** 二选一的 Docker 编排。

## 一键部署

在 **docker 目录**下执行，数据库二选一：

```bash
cd docker
cp .env.example .env   # 可选，按需改端口与密码

# 仅 Redis
docker compose up -d

# Redis + MySQL
docker compose --profile mysql up -d

# Redis + PostgreSQL
docker compose --profile postgres up -d
```

或使用脚本（需传入 mysql 或 postgres）：

```bash
./deploy.sh mysql      # Redis + MySQL
./deploy.sh postgres   # Redis + PostgreSQL
./deploy.sh            # 仅 Redis
```

## 服务与端口

| 服务       | 镜像           | 默认端口 | 说明                    |
|------------|----------------|----------|-------------------------|
| Redis      | redis:latest   | 6379     | 必选，持久化 appendonly |
| MySQL      | mysql:8.0.45   | 3306     | 选 `--profile mysql`    |
| PostgreSQL | postgres:15    | 5432     | 选 `--profile postgres` |

## 连接示例

- **Redis**: `redis://127.0.0.1:6379`
- **MySQL**（启用时）: `mysql://root:root@127.0.0.1:3306/app`
- **PostgreSQL**（启用时）: `postgres://postgres:postgres@127.0.0.1:5432/app`

## 常用命令

```bash
cd docker

# 查看状态（若用过 profile，需带上相同 profile）
docker compose --profile mysql ps   # 或 --profile postgres
docker compose ps                    # 仅 Redis

# 查看日志
docker compose logs -f

# 停止（与启动时使用的 profile 一致）
docker compose down                  # 仅 Redis
docker compose --profile mysql down  # 或 --profile postgres

# 停止并删除数据卷
docker compose --profile mysql down -v
```
