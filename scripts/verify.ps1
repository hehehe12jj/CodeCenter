# CodeAgent Dashboard 验证脚本 (Windows PowerShell)
# 用法: .\scripts\verify.ps1

Write-Host "===================================" -ForegroundColor Cyan
Write-Host " CodeAgent Dashboard 验证脚本"
Write-Host "==================================="
Write-Host ""

# 检查 Node.js
Write-Host "检查 Node.js... " -NoNewline
if (Get-Command node -ErrorAction SilentlyContinue) {
    $nodeVersion = node --version
    Write-Host "✓ $nodeVersion" -ForegroundColor Green
} else {
    Write-Host "✗ 未安装" -ForegroundColor Red
    Write-Host "  请安装 Node.js: https://nodejs.org/"
    exit 1
}

# 检查 npm
Write-Host "检查 npm... " -NoNewline
if (Get-Command npm -ErrorAction SilentlyContinue) {
    $npmVersion = npm --version
    Write-Host "✓ $npmVersion" -ForegroundColor Green
} else {
    Write-Host "✗ 未安装" -ForegroundColor Red
    exit 1
}

# 检查 Rust/Cargo
Write-Host "检查 Cargo... " -NoNewline
if (Get-Command cargo -ErrorAction SilentlyContinue) {
    $cargoVersion = cargo --version
    Write-Host "✓ $cargoVersion" -ForegroundColor Green
} else {
    Write-Host "✗ 未安装" -ForegroundColor Red
    Write-Host ""
    Write-Host "正在尝试安装 Rust..."
    # 下载并运行 rustup
    Invoke-WebRequest -Uri https://win.rustup.rs/x86_64 -OutFile rustup-init.exe
    .\rustup-init.exe -y
    Remove-Item rustup-init.exe
    $env:PATH += ";$env:USERPROFILE\.cargo\bin"
}

Write-Host ""
Write-Host "===================================" -ForegroundColor Cyan
Write-Host " 环境检查完成"
Write-Host "==================================="
Write-Host ""

# 安装 npm 依赖
Write-Host "步骤 1: 安装 npm 依赖..."
Write-Host "-----------------------------------"
npm install
Write-Host "✓ npm 依赖安装完成" -ForegroundColor Green
Write-Host ""

# 构建前端
Write-Host "步骤 2: 构建前端..."
Write-Host "-----------------------------------"
npm run build
Write-Host "✓ 前端构建完成" -ForegroundColor Green
Write-Host ""

# 编译 Rust
Write-Host "步骤 3: 编译 Rust..."
Write-Host "-----------------------------------"
cd src-tauri
cargo build
Write-Host "✓ Rust 编译完成" -ForegroundColor Green
cd ..

Write-Host ""
Write-Host "===================================" -ForegroundColor Green
Write-Host " 所有检查通过！"
Write-Host "==================================="
Write-Host ""
Write-Host "现在可以运行开发服务器:"
Write-Host "  npm run tauri:dev"
