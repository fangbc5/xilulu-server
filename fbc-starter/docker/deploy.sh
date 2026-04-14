#!/usr/bin/env bash
# 中间件一键启动菜单
# 在 docker 目录下执行: ./deploy.sh

set -e
cd "$(dirname "$0")"

if [[ ! -f .env ]]; then
  cp -n .env.example .env 2>/dev/null || true
  echo "已使用默认配置；若需修改端口或密码，请编辑 .env 后重新执行。"
fi
[[ -f .env ]] && set -a && source .env && set +a

echo "请选择要启动的服务（输入序号并回车）："
echo "  1) Redis           (docker-compose-redis.yml)"
echo "  2) MySQL           (docker-compose-mysql.yml)"
echo "  3) PostgreSQL      (docker-compose-postgres.yml)"
echo "  4) rnacos          (docker-compose-rnacos.yml)"
echo "  5) ms-websocket    (docker-compose-ms-websocket.yml)"
echo "  0) 退出"
read -r choice

compose_cmd() {
  local file=$1
  echo "正在使用 $file 启动服务..."
  docker compose -f "$file" up -d
}

case "$choice" in
  1)
    compose_cmd "docker-compose-redis.yml"
    echo "Redis:       localhost:${REDIS_PORT:-6379}"
    ;;
  2)
    compose_cmd "docker-compose-mysql.yml"
    echo "MySQL:       localhost:${MYSQL_PORT:-3306}"
    ;;
  3)
    compose_cmd "docker-compose-postgres.yml"
    echo "PostgreSQL:  localhost:${POSTGRES_PORT:-5432}"
    ;;
  4)
    compose_cmd "docker-compose-rnacos.yml"
    echo "rnacos HTTP: localhost:${RNACOS_HTTP_PORT:-8848}"
    ;;
  5)
    compose_cmd "docker-compose-ms-websocket.yml"
    echo "ms-websocket: ws://localhost:${APP__SERVER__PORT:-30001}/ws"
    ;;
  0)
    echo "已取消。"
    exit 0
    ;;
  *)
    echo "无效选择。"
    exit 1
    ;;
esac

echo "查看状态: docker compose -f 相应文件 ps"
echo "查看日志: docker compose -f 相应文件 logs -f"
echo "停止:     docker compose -f 相应文件 down"
