#!/bin/bash

# 快速启动开发服务器
# 用法: ./scripts/dev.sh

echo "启动 CodeAgent Dashboard 开发服务器..."
echo ""

# 检查依赖是否已安装
if [ ! -d "node_modules" ]; then
    echo "npm 依赖未安装，正在安装..."
    npm install
fi

# 启动开发服务器
echo "启动 Tauri 开发服务器..."
echo ""
npm run tauri:dev
