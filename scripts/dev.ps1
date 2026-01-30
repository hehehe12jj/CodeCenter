# 快速启动开发服务器 (Windows)
# 用法: .\scripts\dev.ps1

Write-Host "启动 CodeAgent Dashboard 开发服务器..."
Write-Host ""

# 检查依赖是否已安装
if (-not (Test-Path "node_modules")) {
    Write-Host "npm 依赖未安装，正在安装..."
    npm install
}

# 启动开发服务器
Write-Host "启动 Tauri 开发服务器..."
Write-Host ""
npm run tauri:dev
