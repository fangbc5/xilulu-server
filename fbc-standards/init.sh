#!/bin/bash
# =============================================================================
# fbc-starter 微服务规范初始化 & 同步工具
#
# 用法:
#   bash init.sh [目标目录]           # 交互式初始化单个项目
#   bash init.sh --sync <工具编号>    # 批量同步到所有 ms-* 微服务
#
# 工具编号:
#   1=Cursor  2=Copilot  3=Gemini  4=Claude  5=Windsurf  6=Cline  7=Antigravity  all=全部
#
# 示例:
#   bash init.sh ../ms-im                # 交互式初始化 ms-im
#   bash init.sh --sync 4,7              # 同步 Claude + Antigravity 到所有 ms-*
#   bash init.sh --sync all              # 同步全部工具到所有 ms-*
#   bash init.sh --sync 7 ms-im ms-auth  # 只同步 Antigravity 到指定项目
# =============================================================================

set -e

# ---------- 颜色定义 ----------
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# ---------- 脚本所在目录（用于定位模板文件） ----------
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
# workspace 根目录（fbc-standards 的上层目录）
WORKSPACE_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

# ---------- 检查模板文件是否存在 ----------
check_templates() {
    if [ ! -d "$SCRIPT_DIR/.aiproject" ]; then
        echo -e "${RED}❌ 未找到 .aiproject 规范目录，请确认脚本位置正确${NC}"
        exit 1
    fi
    if [ ! -d "$SCRIPT_DIR/templates" ]; then
        echo -e "${RED}❌ 未找到 templates 模板目录，请确认脚本位置正确${NC}"
        exit 1
    fi
}

# ---------- 安装函数（TARGET_DIR 由调用方设置） ----------
install_cursor() {
    cp "$SCRIPT_DIR/templates/cursorrules" "$TARGET_DIR/.cursorrules"
    mkdir -p "$TARGET_DIR/.cursor/rules"
    cp "$SCRIPT_DIR/templates/cursor_rules.md" "$TARGET_DIR/.cursor/rules/fbc-starter.md"
    echo -e "${GREEN}    ✅ .cursorrules + .cursor/rules/${NC}"
}

install_copilot() {
    mkdir -p "$TARGET_DIR/.github"
    cp "$SCRIPT_DIR/templates/copilot_instructions.md" "$TARGET_DIR/.github/copilot-instructions.md"
    echo -e "${GREEN}    ✅ .github/copilot-instructions.md${NC}"
}

install_gemini() {
    mkdir -p "$TARGET_DIR/.gemini"
    cp "$SCRIPT_DIR/templates/gemini_settings.json" "$TARGET_DIR/.gemini/settings.json"
    cp "$SCRIPT_DIR/templates/gemini_styleguide.md" "$TARGET_DIR/.gemini/styleguide.md"
    echo -e "${GREEN}    ✅ .gemini/settings.json + styleguide.md${NC}"
}

install_claude() {
    cp "$SCRIPT_DIR/templates/claude.md" "$TARGET_DIR/CLAUDE.md"
    echo -e "${GREEN}    ✅ CLAUDE.md${NC}"
}

install_windsurf() {
    cp "$SCRIPT_DIR/templates/windsurfrules" "$TARGET_DIR/.windsurfrules"
    echo -e "${GREEN}    ✅ .windsurfrules${NC}"
}

install_cline() {
    cp "$SCRIPT_DIR/templates/clinerules" "$TARGET_DIR/.clinerules"
    echo -e "${GREEN}    ✅ .clinerules${NC}"
}

install_antigravity() {
    mkdir -p "$TARGET_DIR/.agents/rules"
    cp -r "$SCRIPT_DIR/templates/.agents/rules/" "$TARGET_DIR/.agents/rules/"
    echo -e "${GREEN}    ✅ .agents/rules/${NC}"
}

# 根据编号执行安装
install_by_number() {
    local num=$1
    case "$num" in
        1) install_cursor ;;
        2) install_copilot ;;
        3) install_gemini ;;
        4) install_claude ;;
        5) install_windsurf ;;
        6) install_cline ;;
        7) install_antigravity ;;
        *) echo -e "${YELLOW}    ⚠️  未知工具编号: $num${NC}"; return 1 ;;
    esac
}

# =============================================================================
# --sync 批量同步模式
# =============================================================================
do_sync() {
    local tools_arg="$1"
    shift

    check_templates

    echo ""
    echo -e "${CYAN}╔══════════════════════════════════════════════════════════╗${NC}"
    echo -e "${CYAN}║   🔄 fbc-standards 批量同步                            ║${NC}"
    echo -e "${CYAN}╚══════════════════════════════════════════════════════════╝${NC}"
    echo ""

    # 解析工具编号（逗号分隔 → 数组）
    local tools=()
    if [ "$tools_arg" = "all" ]; then
        tools=(1 2 3 4 5 6 7)
    else
        IFS=',' read -ra tools <<< "$tools_arg"
    fi

    echo -e "  同步工具: ${GREEN}${tools[*]}${NC}"

    # 确定目标目录
    local targets=()
    if [ $# -gt 0 ]; then
        # 用户指定了目标目录
        for t in "$@"; do
            local abs_path
            if [[ "$t" = /* ]]; then
                abs_path="$t"
            else
                abs_path="$WORKSPACE_DIR/$t"
            fi
            if [ -d "$abs_path" ]; then
                targets+=("$abs_path")
            else
                echo -e "${YELLOW}  ⚠️  目录不存在，跳过: $t${NC}"
            fi
        done
    else
        # 自动发现 workspace 下的 ms-* 目录
        for dir in "$WORKSPACE_DIR"/ms-*/; do
            if [ -d "$dir" ]; then
                targets+=("${dir%/}")
            fi
        done
    fi

    if [ ${#targets[@]} -eq 0 ]; then
        echo -e "${RED}❌ 未找到任何目标微服务目录${NC}"
        exit 1
    fi

    echo -e "  目标项目: ${GREEN}${#targets[@]}${NC} 个"
    echo ""

    # 逐个项目同步
    local synced=0
    for target in "${targets[@]}"; do
        local name
        name=$(basename "$target")
        TARGET_DIR="$target"
        echo -e "${BLUE}  📦 $name${NC}"

        # 同步 .aiproject（静默覆盖）
        cp -r "$SCRIPT_DIR/.aiproject" "$TARGET_DIR/"

        # 同步指定工具
        for tool_num in "${tools[@]}"; do
            install_by_number "$tool_num"
        done

        synced=$((synced + 1))
    done

    echo ""
    echo -e "${CYAN}╔══════════════════════════════════════════════════════════╗${NC}"
    echo -e "${CYAN}║   ✅ 同步完成！                                        ║${NC}"
    echo -e "${CYAN}╚══════════════════════════════════════════════════════════╝${NC}"
    echo ""
    echo -e "  已同步 ${GREEN}${synced}${NC} 个微服务"
    echo ""
}

# =============================================================================
# 交互式初始化模式（原有逻辑）
# =============================================================================
do_init() {
    TARGET_DIR="${1:-.}"

    # 转为绝对路径
    TARGET_DIR="$(cd "$TARGET_DIR" 2>/dev/null && pwd)" || {
        echo -e "${RED}❌ 目标目录不存在: $1${NC}"
        exit 1
    }

    check_templates

    echo ""
    echo -e "${CYAN}╔══════════════════════════════════════════════════════════╗${NC}"
    echo -e "${CYAN}║   🚀 fbc-starter 微服务开发规范初始化                   ║${NC}"
    echo -e "${CYAN}╚══════════════════════════════════════════════════════════╝${NC}"
    echo ""
    echo -e "  目标目录: ${GREEN}${TARGET_DIR}${NC}"
    echo ""

    # ---------- 检查是否已存在 .aiproject ----------
    if [ -d "$TARGET_DIR/.aiproject" ]; then
        echo -e "${YELLOW}⚠️  目标目录已存在 .aiproject 目录${NC}"
        read -p "  是否覆盖？(y/N): " overwrite
        if [[ ! "$overwrite" =~ ^[Yy]$ ]]; then
            echo -e "${YELLOW}  跳过 .aiproject 复制${NC}"
            SKIP_AIPROJECT=true
        fi
    fi

    # ---------- 复制 .aiproject 规范文件 ----------
    if [ "$SKIP_AIPROJECT" != "true" ]; then
        echo ""
        echo -e "${BLUE}📋 复制 .aiproject 规范文件...${NC}"
        cp -r "$SCRIPT_DIR/.aiproject" "$TARGET_DIR/"
        echo -e "${GREEN}  ✅ .aiproject/ (README.md + P0-P9)${NC}"
    fi

    # ---------- 选择 AI 工具 ----------
    echo ""
    echo -e "${BLUE}🤖 选择你使用的 AI 编码工具（多选，用空格分隔编号）:${NC}"
    echo ""
    echo -e "  ${CYAN}1)${NC} Cursor          → .cursorrules + .cursor/rules/"
    echo -e "  ${CYAN}2)${NC} GitHub Copilot  → .github/copilot-instructions.md"
    echo -e "  ${CYAN}3)${NC} Gemini          → .gemini/settings.json + styleguide.md"
    echo -e "  ${CYAN}4)${NC} Claude Code     → CLAUDE.md"
    echo -e "  ${CYAN}5)${NC} Windsurf        → .windsurfrules"
    echo -e "  ${CYAN}6)${NC} Cline           → .clinerules"
    echo -e "  ${CYAN}7)${NC} Antigravity     → .agents/rules/"
    echo -e "  ${CYAN}8)${NC} 全部安装"
    echo ""
    read -p "  请输入编号 (例如: 1 3 4 7): " -a choices

    # 如果选择了 8（全部），展开为 1-7
    for choice in "${choices[@]}"; do
        if [ "$choice" = "8" ]; then
            choices=(1 2 3 4 5 6 7)
            break
        fi
    done

    echo ""

    # ---------- 执行安装 ----------
    installed=0
    for choice in "${choices[@]}"; do
        if install_by_number "$choice"; then
            installed=$((installed + 1))
        fi
    done

    # ---------- 更新 .gitignore（可选）----------
    echo ""
    if [ -f "$TARGET_DIR/.gitignore" ]; then
        has_aiproject=$(grep -c ".aiproject" "$TARGET_DIR/.gitignore" 2>/dev/null || true)
        if [ "$has_aiproject" = "0" ]; then
            add_gitignore=""
            read -p "  是否将 AI 工具配置加入 .gitignore？(y/N): " add_gitignore 2>/dev/null || true
            if [[ "$add_gitignore" =~ ^[Yy]$ ]]; then
                echo "" >> "$TARGET_DIR/.gitignore"
                echo "# AI 工具配置（项目级规范应提交到仓库）" >> "$TARGET_DIR/.gitignore"
                echo "# 如需共享规范，请注释掉以下行" >> "$TARGET_DIR/.gitignore"
                echo "# .cursorrules" >> "$TARGET_DIR/.gitignore"
                echo "# .cursor/" >> "$TARGET_DIR/.gitignore"
                echo "# .windsurfrules" >> "$TARGET_DIR/.gitignore"
                echo "# .clinerules" >> "$TARGET_DIR/.gitignore"
                echo "# .agents/" >> "$TARGET_DIR/.gitignore"
                echo "# CLAUDE.md" >> "$TARGET_DIR/.gitignore"
                echo -e "${GREEN}  ✅ 已更新 .gitignore（AI 配置默认提交，如需忽略请取消注释）${NC}"
            fi
        fi
    fi

    # ---------- 完成 ----------
    echo ""
    echo -e "${CYAN}╔══════════════════════════════════════════════════════════╗${NC}"
    echo -e "${CYAN}║   ✅ 初始化完成！                                       ║${NC}"
    echo -e "${CYAN}╚══════════════════════════════════════════════════════════╝${NC}"
    echo ""
    echo -e "  已安装 ${GREEN}${installed}${NC} 个 AI 工具配置"
    echo ""
    echo -e "  📖 规范文件位于: ${GREEN}${TARGET_DIR}/.aiproject/${NC}"
    echo -e "  📖 规范总览:     ${GREEN}${TARGET_DIR}/.aiproject/STANDARDS.md${NC}"
    echo ""
    echo -e "  ${YELLOW}💡 提示: 建议将 .aiproject/ 和 AI 工具配置提交到 Git 仓库，${NC}"
    echo -e "  ${YELLOW}   这样团队中所有开发者和 AI 工具都能读取规范。${NC}"
    echo ""
}

# =============================================================================
# 入口：根据参数分发
# =============================================================================
case "${1:-}" in
    --sync)
        if [ -z "${2:-}" ]; then
            echo -e "${RED}❌ 用法: bash init.sh --sync <工具编号> [目标目录...]${NC}"
            echo -e "  工具编号: 1-7 (逗号分隔) 或 all"
            echo -e "  示例: bash init.sh --sync 4,7"
            echo -e "        bash init.sh --sync all ms-im ms-auth"
            exit 1
        fi
        shift  # 移除 --sync
        do_sync "$@"
        ;;
    --help|-h)
        echo "用法:"
        echo "  bash init.sh [目标目录]           交互式初始化单个项目"
        echo "  bash init.sh --sync <工具> [目录]  批量同步到微服务"
        echo ""
        echo "工具编号: 1=Cursor 2=Copilot 3=Gemini 4=Claude 5=Windsurf 6=Cline 7=Antigravity all=全部"
        echo ""
        echo "示例:"
        echo "  bash init.sh ../ms-im              交互式初始化 ms-im"
        echo "  bash init.sh --sync 4,7            同步 Claude + Antigravity 到所有 ms-*"
        echo "  bash init.sh --sync all             同步全部工具到所有 ms-*"
        echo "  bash init.sh --sync 7 ms-im ms-auth 只同步指定项目"
        ;;
    *)
        do_init "$@"
        ;;
esac
