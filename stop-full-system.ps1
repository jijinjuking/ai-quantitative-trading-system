# 停止完整的Rust微服务交易系统
# PowerShell脚本

Write-Host "🛑 停止企业级量化交易平台..." -ForegroundColor Red

# 停止所有Rust进程
Write-Host "🦀 停止Rust微服务进程..." -ForegroundColor Yellow
$rustProcesses = Get-Process | Where-Object { $_.ProcessName -like "*cargo*" -or $_.ProcessName -like "*gateway*" -or $_.ProcessName -like "*market-data*" -or $_.ProcessName -like "*trading-engine*" -or $_.ProcessName -like "*strategy-engine*" }
if ($rustProcesses) {
    $rustProcesses | Stop-Process -Force
    Write-Host "✅ Rust微服务进程已停止" -ForegroundColor Green
} else {
    Write-Host "ℹ️ 未找到运行中的Rust微服务进程" -ForegroundColor Cyan
}

# 停止Node.js进程（前端）
Write-Host "🎨 停止前端应用进程..." -ForegroundColor Yellow
$nodeProcesses = Get-Process | Where-Object { $_.ProcessName -like "*node*" -and $_.CommandLine -like "*vite*" }
if ($nodeProcesses) {
    $nodeProcesses | Stop-Process -Force
    Write-Host "✅ 前端应用进程已停止" -ForegroundColor Green
} else {
    Write-Host "ℹ️ 未找到运行中的前端应用进程" -ForegroundColor Cyan
}

# 停止Docker容器
Write-Host "🐳 停止Docker基础设施服务..." -ForegroundColor Yellow
try {
    docker-compose -f docker-compose.dev.yml down
    Write-Host "✅ Docker基础设施服务已停止" -ForegroundColor Green
} catch {
    Write-Host "⚠️ 停止Docker服务时出现错误" -ForegroundColor Yellow
}

# 清理端口占用
Write-Host "🧹 清理端口占用..." -ForegroundColor Yellow
$ports = @(3000, 8080, 8081, 8082, 8083, 8084, 8085, 8086, 8087, 9092, 6379, 8123)
foreach ($port in $ports) {
    $process = Get-NetTCPConnection -LocalPort $port -ErrorAction SilentlyContinue
    if ($process) {
        $processId = $process.OwningProcess
        try {
            Stop-Process -Id $processId -Force -ErrorAction SilentlyContinue
            Write-Host "✅ 端口 $port 已释放" -ForegroundColor Green
        } catch {
            Write-Host "⚠️ 无法释放端口 $port" -ForegroundColor Yellow
        }
    }
}

Write-Host ""
Write-Host "🎉 系统已完全停止！" -ForegroundColor Green
Write-Host ""
Write-Host "📋 已停止的服务:" -ForegroundColor Yellow
Write-Host "  🌐 API网关服务" -ForegroundColor White
Write-Host "  📈 市场数据服务" -ForegroundColor White
Write-Host "  ⚡ 交易引擎服务" -ForegroundColor White
Write-Host "  🧠 策略引擎服务" -ForegroundColor White
Write-Host "  🎨 前端应用" -ForegroundColor White
Write-Host "  🐳 Docker基础设施" -ForegroundColor White
Write-Host ""
Write-Host "💡 提示:" -ForegroundColor Yellow
Write-Host "  - 所有服务进程已终止" -ForegroundColor White
Write-Host "  - 端口占用已清理" -ForegroundColor White
Write-Host "  - Docker容器已停止" -ForegroundColor White
Write-Host "  - 可以重新运行 start-full-system.ps1 来启动系统" -ForegroundColor White