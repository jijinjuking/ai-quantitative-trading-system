# Docker开发环境管理脚本 (PowerShell版本)

param(
    [Parameter(Position=0)]
    [string]$Command = "help",
    
    [Parameter(Position=1)]
    [string]$Service = ""
)

# 设置项目目录
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectDir = Split-Path -Parent $ScriptDir
Set-Location $ProjectDir

# 颜色输出函数
function Write-Info($Message) {
    Write-Host "[INFO] $Message" -ForegroundColor Blue
}

function Write-Success($Message) {
    Write-Host "[SUCCESS] $Message" -ForegroundColor Green
}

function Write-Warning($Message) {
    Write-Host "[WARNING] $Message" -ForegroundColor Yellow
}

function Write-Error($Message) {
    Write-Host "[ERROR] $Message" -ForegroundColor Red
}

# 显示帮助信息
function Show-Help {
    Write-Host "Docker开发环境管理脚本" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "用法: .\docker-dev.ps1 [命令]" -ForegroundColor White
    Write-Host ""
    Write-Host "命令:" -ForegroundColor White
    Write-Host "  build     构建开发环境镜像" -ForegroundColor Gray
    Write-Host "  up        启动所有服务" -ForegroundColor Gray
    Write-Host "  down      停止所有服务" -ForegroundColor Gray
    Write-Host "  restart   重启所有服务" -ForegroundColor Gray
    Write-Host "  shell     进入Rust开发容器" -ForegroundColor Gray
    Write-Host "  logs      查看服务日志" -ForegroundColor Gray
    Write-Host "  status    查看服务状态" -ForegroundColor Gray
    Write-Host "  clean     清理所有数据和镜像" -ForegroundColor Gray
    Write-Host "  compile   在容器中编译项目" -ForegroundColor Gray
    Write-Host "  test      在容器中运行测试" -ForegroundColor Gray
    Write-Host "  help      显示此帮助信息" -ForegroundColor Gray
    Write-Host ""
    Write-Host "示例:" -ForegroundColor White
    Write-Host "  .\docker-dev.ps1 build; .\docker-dev.ps1 up" -ForegroundColor Gray
    Write-Host "  .\docker-dev.ps1 shell" -ForegroundColor Gray
    Write-Host "  .\docker-dev.ps1 compile" -ForegroundColor Gray
}

# 检查Docker是否安装
function Test-Docker {
    try {
        $null = docker --version
        $null = docker-compose --version
        return $true
    }
    catch {
        Write-Error "Docker或Docker Compose未安装，请先安装Docker Desktop"
        return $false
    }
}

# 构建开发环境
function Build-Environment {
    Write-Info "构建Rust开发环境镜像..."
    
    try {
        docker-compose -f docker-compose.build.yml build rust-dev
        Write-Success "开发环境镜像构建完成"
    }
    catch {
        Write-Error "镜像构建失败: $_"
        exit 1
    }
}

# 启动服务
function Start-Services {
    Write-Info "启动开发环境服务..."
    
    # 创建必要的目录
    $directories = @("data", "data\redis", "data\clickhouse", "data\kafka", "data\zookeeper", "config", "config\clickhouse")
    foreach ($dir in $directories) {
        if (!(Test-Path $dir)) {
            New-Item -ItemType Directory -Path $dir -Force | Out-Null
        }
    }
    
    try {
        # 启动基础设施服务
        docker-compose -f docker-compose.build.yml up -d redis clickhouse zookeeper kafka
        
        Write-Info "等待服务启动..."
        Start-Sleep -Seconds 10
        
        # 启动管理工具
        docker-compose -f docker-compose.build.yml up -d kafka-ui redis-commander
        
        Write-Success "所有服务已启动"
        Write-Info "服务访问地址:"
        Write-Host "  - Kafka UI: http://localhost:8080" -ForegroundColor Gray
        Write-Host "  - Redis Commander: http://localhost:8081" -ForegroundColor Gray
        Write-Host "  - ClickHouse HTTP: http://localhost:8123" -ForegroundColor Gray
    }
    catch {
        Write-Error "服务启动失败: $_"
        exit 1
    }
}

# 停止服务
function Stop-Services {
    Write-Info "停止所有服务..."
    
    try {
        docker-compose -f docker-compose.build.yml down
        Write-Success "所有服务已停止"
    }
    catch {
        Write-Error "停止服务失败: $_"
    }
}

# 重启服务
function Restart-Services {
    Write-Info "重启所有服务..."
    Stop-Services
    Start-Services
}

# 进入开发容器
function Enter-Shell {
    Write-Info "进入Rust开发容器..."
    
    # 检查容器是否运行
    $containerStatus = docker-compose -f docker-compose.build.yml ps rust-dev
    if ($containerStatus -notmatch "Up") {
        Write-Info "启动Rust开发容器..."
        docker-compose -f docker-compose.build.yml up -d rust-dev
        Start-Sleep -Seconds 5
    }
    
    docker-compose -f docker-compose.build.yml exec rust-dev bash
}

# 查看日志
function Show-Logs {
    if ($Service -eq "") {
        Write-Info "显示所有服务日志..."
        docker-compose -f docker-compose.build.yml logs -f
    } else {
        Write-Info "显示 $Service 服务日志..."
        docker-compose -f docker-compose.build.yml logs -f $Service
    }
}

# 查看服务状态
function Show-Status {
    Write-Info "服务状态:"
    docker-compose -f docker-compose.build.yml ps
    
    Write-Host ""
    Write-Info "容器资源使用情况:"
    docker stats --no-stream --format "table {{.Container}}\t{{.CPUPerc}}\t{{.MemUsage}}\t{{.NetIO}}"
}

# 清理环境
function Clean-Environment {
    Write-Warning "这将删除所有容器、镜像和数据，确定要继续吗? (y/N)"
    $response = Read-Host
    
    if ($response -eq "y" -or $response -eq "Y") {
        Write-Info "停止并删除所有容器..."
        docker-compose -f docker-compose.build.yml down -v --rmi all
        
        Write-Info "清理未使用的Docker资源..."
        docker system prune -f
        
        Write-Info "删除数据目录..."
        if (Test-Path "data") {
            Remove-Item -Path "data" -Recurse -Force
        }
        
        Write-Success "环境清理完成"
    } else {
        Write-Info "取消清理操作"
    }
}

# 编译项目
function Compile-Project {
    Write-Info "在容器中编译项目..."
    
    # 检查容器是否运行
    $containerStatus = docker-compose -f docker-compose.build.yml ps rust-dev
    if ($containerStatus -notmatch "Up") {
        Write-Info "启动Rust开发容器..."
        docker-compose -f docker-compose.build.yml up -d rust-dev
        Start-Sleep -Seconds 5
    }
    
    try {
        docker-compose -f docker-compose.build.yml exec rust-dev cargo build --release
        Write-Success "项目编译成功"
    }
    catch {
        Write-Error "项目编译失败: $_"
        exit 1
    }
}

# 运行测试
function Run-Tests {
    Write-Info "在容器中运行测试..."
    
    # 检查容器是否运行
    $containerStatus = docker-compose -f docker-compose.build.yml ps rust-dev
    if ($containerStatus -notmatch "Up") {
        Write-Info "启动Rust开发容器..."
        docker-compose -f docker-compose.build.yml up -d rust-dev
        Start-Sleep -Seconds 5
    }
    
    try {
        docker-compose -f docker-compose.build.yml exec rust-dev cargo test
        Write-Success "所有测试通过"
    }
    catch {
        Write-Error "测试失败: $_"
        exit 1
    }
}

# 主逻辑
if (!(Test-Docker)) {
    exit 1
}

switch ($Command.ToLower()) {
    "build" { Build-Environment }
    "up" { Start-Services }
    "down" { Stop-Services }
    "restart" { Restart-Services }
    "shell" { Enter-Shell }
    "logs" { Show-Logs }
    "status" { Show-Status }
    "clean" { Clean-Environment }
    "compile" { Compile-Project }
    "test" { Run-Tests }
    "help" { Show-Help }
    default {
        Write-Error "未知命令: $Command"
        Write-Host ""
        Show-Help
        exit 1
    }
}