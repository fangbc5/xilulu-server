#!/bin/bash
# ============================================================================
# SQL Migration Executor
# Purpose: 一键执行所有数据库迁移脚本
# Usage: bash run_migrations.sh [--dev|--prod] [--dry-run]
# ============================================================================

set -e  # 任何命令失败时停止执行

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 配置
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DB_HOST="${DB_HOST:-localhost}"
DB_PORT="${DB_PORT:-3306}"
DB_USER="${DB_USER:-root}"
DB_PASSWORD="${DB_PASSWORD:-}"
DB_NAME="${DB_NAME:-ms_team}"
ENVIRONMENT="${1:-dev}"
DRY_RUN="${2:---execute}"

# 迁移脚本列表
MIGRATIONS=(
    "V001_organization_enhancements.sql"
    "V002_employee_enhancements.sql"
    "V003_employee_department_enhancements.sql"
    "V004_create_location_table.sql"
    "V005_create_secondment_table.sql"
    "V006_init_organization_path_level.sql"
    "V007_init_employee_primary_dept.sql"
)

# ============================================================================
# 函数定义
# ============================================================================

print_header() {
    echo -e "${BLUE}========================================${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}========================================${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

# 检查 MySQL 连接
check_mysql_connection() {
    print_header "检查 MySQL 连接"
    
    if ! command -v mysql &> /dev/null; then
        print_error "mysql 命令未找到，请先安装 MySQL Client"
        exit 1
    fi
    
    export MYSQL_PWD="$DB_PASSWORD"
    mysql -h "$DB_HOST" -P "$DB_PORT" -u "$DB_USER" -e "SELECT 1" > /dev/null 2>&1
    local result=$?
    unset MYSQL_PWD
    
    if [ $result -eq 0 ]; then
        print_success "MySQL 连接成功"
    else
        print_error "MySQL 连接失败"
        print_warning "请检查以下参数："
        echo "  DB_HOST=$DB_HOST"
        echo "  DB_PORT=$DB_PORT"
        echo "  DB_USER=$DB_USER"
        echo "  DB_NAME=$DB_NAME"
        exit 1
    fi
}

# 执行迁移脚本
run_migration() {
    local script_name="$1"
    local script_path="$SCRIPT_DIR/$script_name"
    
    if [ ! -f "$script_path" ]; then
        print_error "脚本不存在：$script_path"
        return 1
    fi
    
    echo ""
    print_header "执行：$script_name"
    
    if [ "$DRY_RUN" == "--dry-run" ]; then
        print_warning "（干运行模式，仅显示脚本内容）"
        head -20 "$script_path"
        echo "..."
    else
        export MYSQL_PWD="$DB_PASSWORD"
        mysql -h "$DB_HOST" -P "$DB_PORT" -u "$DB_USER" "$DB_NAME" < "$script_path"
        local result=$?
        unset MYSQL_PWD
        
        if [ $result -eq 0 ]; then
            print_success "$script_name 执行成功"
        else
            print_error "$script_name 执行失败"
            return 1
        fi
    fi
}

# 备份数据库
backup_database() {
    if [ "$ENVIRONMENT" == "prod" ]; then
        print_header "开始备份数据库"
        
        BACKUP_FILE="${SCRIPT_DIR}/../backups/backup_$(date +'%Y%m%d_%H%M%S').sql"
        mkdir -p "${SCRIPT_DIR}/../backups"
        
        export MYSQL_PWD="$DB_PASSWORD"
        mysqldump -h "$DB_HOST" -P "$DB_PORT" -u "$DB_USER" \
            --single-transaction --quick "${DB_NAME}" > "${BACKUP_FILE}"
        unset MYSQL_PWD
        
        if [ $? -eq 0 ]; then
            print_success "数据库备份成功：$BACKUP_FILE"
        else
            print_error "数据库备份失败"
            exit 1
        fi
    fi
}

# 显示版本信息
show_version_info() {
    echo ""
    print_header "版本信息"
    echo "当前版本：V007"
    echo "发布日期：2026-02-10"
    echo "迁移脚本数：${#MIGRATIONS[@]}"
    echo ""
    echo "包含的迁移："
    for i in "${!MIGRATIONS[@]}"; do
        echo "  $((i+1)). ${MIGRATIONS[$i]}"
    done
}

# ============================================================================
# 主程序
# ============================================================================

main() {
    print_header "数据库迁移工具 v1.0"
    
    echo ""
    echo "配置信息："
    echo "  环境：$ENVIRONMENT"
    echo "  主机：$DB_HOST:$DB_PORT"
    echo "  用户：$DB_USER"
    echo "  数据库：$DB_NAME"
    echo "  模式：$([ "$DRY_RUN" == "--dry-run" ] && echo '干运行' || echo '正式执行')"
    
    # 检查连接
    check_mysql_connection
    
    # 显示版本信息
    show_version_info
    
    # 备份（仅生产环境）
    if [ "$ENVIRONMENT" == "prod" ]; then
        print_warning "生产环境模式 - 将执行数据库备份"
        read -p "确定要继续吗？(yes/no) " -n 3 -r
        echo
        if [[ ! $REPLY =~ ^[Yy][Ee][Ss]$ ]]; then
            print_warning "操作已取消"
            exit 0
        fi
        backup_database
    fi
    
    # 执行所有迁移
    print_header "开始执行迁移脚本"
    
    FAILED_MIGRATIONS=()
    
    for migration in "${MIGRATIONS[@]}"; do
        if ! run_migration "$migration"; then
            FAILED_MIGRATIONS+=("$migration")
        fi
    done
    
    # 汇总结果
    echo ""
    print_header "迁移结果汇总"
    
    if [ ${#FAILED_MIGRATIONS[@]} -eq 0 ]; then
        print_success "所有迁移都执行成功！"
        echo ""
        echo "后续步骤："
        echo "  1. 验证数据库结构：DESC organization;"
        echo "  2. 运行验证查询"
        echo "  3. 更新应用代码"
        echo "  4. 重启应用服务"
    else
        print_error "有 ${#FAILED_MIGRATIONS[@]} 个迁移执行失败："
        for failed in "${FAILED_MIGRATIONS[@]}"; do
            echo "  - $failed"
        done
        echo ""
        print_warning "请检查错误信息并重试"
        exit 1
    fi
}

# 运行主程序
main "$@"
