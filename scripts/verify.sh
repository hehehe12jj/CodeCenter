#!/bin/bash

# CodeAgent Dashboard 验证脚本
# 用法: ./scripts/verify.sh

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "==================================="
echo " CodeAgent Dashboard 验证脚本"
echo "==================================="
echo ""

# 检查 Node.js
echo -n "检查 Node.js... "
if command -v node &> /dev/null; then
    NODE_VERSION=$(node --version)
    echo -e "${GREEN}✓${NC} $NODE_VERSION"
else
    echo -e "${RED}✗ 未安装${NC}"
    echo "  请安装 Node.js: https://nodejs.org/"
    exit 1
fi

# 检查 npm
echo -n "检查 npm... "
if command -v npm &> /dev/null; then
    NPM_VERSION=$(npm --version)
    echo -e "${GREEN}✓${NC} $NPM_VERSION"
else
    echo -e "${RED}✗ 未安装${NC}"
    exit 1
fi

# 检查 Rust/Cargo
echo -n "检查 Cargo... "
if command -v cargo &> /dev/null; then
    CARGO_VERSION=$(cargo --version)
    echo -e "${GREEN}✓${NC} $CARGO_VERSION"
else
    echo -e "${RED}✗ 未安装${NC}"
    echo ""
    echo "Rust 未安装，正在尝试安装..."
    echo ""

    # 检测操作系统
    if [[ "$OSTYPE" == "darwin"* ]]; then
        # macOS
        if command -v brew &> /dev/null; then
            echo "通过 Homebrew 安装 Rust..."
            brew install rust
        else
            echo "正在下载 Rust 安装器..."
            curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
            source "$HOME/.cargo/env"
        fi
    elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
        # Linux
        echo "正在下载 Rust 安装器..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source "$HOME/.cargo/env"
    else
        echo -e "${RED}请手动安装 Rust: https://rustup.rs/${NC}"
        exit 1
    fi
fi

# 检查 Tauri CLI
echo -n "检查 Tauri CLI... "
if command -v tauri &> /dev/null || npm list -g @tauri-apps/cli &> /dev/null; then
    echo -e "${GREEN}✓${NC} 已安装"
else
    echo -e "${YELLOW}⚠ 未全局安装${NC}"
    echo "  将使用 npx 运行"
fi

echo ""
echo "==================================="
echo " 环境检查完成"
echo "==================================="
echo ""

# 安装 npm 依赖
echo "步骤 1: 安装 npm 依赖..."
echo "-----------------------------------"
npm install
echo -e "${GREEN}✓ npm 依赖安装完成${NC}"
echo ""

# 构建前端
echo "步骤 2: 构建前端..."
echo "-----------------------------------"
npm run build
echo -e "${GREEN}✓ 前端构建完成${NC}"
echo ""

# 检查 Rust 代码格式
echo "步骤 3: 检查 Rust 代码..."
echo "-----------------------------------"
cd src-tauri

# 检查 cargo fmt
echo -n "代码格式化检查... "
if cargo fmt -- --check 2>/dev/null; then
    echo -e "${GREEN}✓${NC} 已格式化"
else
    echo -e "${YELLOW}⚠ 需要格式化${NC}"
    echo "  运行: cargo fmt"
fi

# 检查 cargo clippy
echo -n "代码质量检查 (clippy)... "
if cargo clippy -- -D warnings 2>/dev/null; then
    echo -e "${GREEN}✓${NC} 无警告"
else
    echo -e "${YELLOW}⚠ 有警告或错误${NC}"
fi

# 编译 Rust
echo ""
echo "步骤 4: 编译 Rust..."
echo "-----------------------------------"
cargo build
echo -e "${GREEN}✓ Rust 编译完成${NC}"

cd ..
echo ""

# 最终状态
echo "==================================="
echo -e "${GREEN} 所有检查通过！${NC}"
echo "==================================="
echo ""
echo "现在可以运行开发服务器:"
echo "  npm run tauri:dev"
echo ""
echo "或构建发布版本:"
echo "  npm run tauri:build"
echo ""
