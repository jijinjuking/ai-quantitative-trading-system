# 验证Rust微服务交易系统状态
# PowerShell脚本

Write-Host "🔍 验证企业级量化交易平台状态..." -ForegroundColor Green

# 检查端口占用情况
Write-Host ""
Write-Host "📊 检查服务端口状态:" -ForegroundColor Yellow

$services = @(
    @{Name="前端应用"; Port=3000; Color="Magenta"},
    @{Name="API网关"; Port=8080; Color="Cyan"},
    @{Name="用户管理"; Port=8081; Color="Blue"},
    @{Name="交易引擎"; Port=8082; Color="Red"},
    @{Name="市场数据"; Port=8083; Color="Green"},
    @{Name="策略引擎"; Port=8084; Color="Yellow"},
    @{Name="风险管理"; Port=8085; Color="Magenta"},
    @{Name="通知服务"; Port=8086; Color="Cyan"},
    @{Name="分析服务"; Port=8087; Color="Blue"},
    @{Name="Kafka"; Port=9092; Color="DarkYellow"},
    @{Name="Redis"; Port=6379; Color="DarkRed"},
    @{Name="ClickHouse"; Port=8123; Color="DarkGreen"}
)

foreach ($service in $services) {
    try {
        $connection = Get-NetTCPConnection -LocalPort $service.Port -State Listen -ErrorAction SilentlyContinue
        if ($connection) {
            Write-Host "  ✅ $($service.Name) (端口 $($service.Port)): 运行中" -ForegroundColor $service.Color
        } else {
            Write-Host "  ❌ $($service.Name) (端口 $($service.Port)): 未运行" -ForegroundColor Red
        }
    } catch {
        Write-Host "  ❌ $($service.Name) (端口 $($service.Port)): 未运行" -ForegroundColor Red
    }
}

# 检查Docker容器状态
Write-Host ""
Write-Host "🐳 检查Docker容器状态:" -ForegroundColor Yellow
try {
    $containers = docker-compose -f docker-compose.dev.yml ps --format "table {{.Name}}\t{{.State}}"
    if ($containers) {
        $containers | ForEach-Object {
            if ($_ -match "running") {
                Write-Host "  ✅ $_" -ForegroundColor Green
            } elseif ($_ -match "exited") {
                Write-Host "  ❌ $_" -ForegroundColor Red
            } else {
                Write-Host "  ⚠️ $_" -ForegroundColor Yellow
            }
        }
    } else {
        Write-Host "  ❌ 未找到Docker容器" -ForegroundColor Red
    }
} catch {
    Write-Host "  ❌ Docker未运行或配置错误" -ForegroundColor Red
}

# 检查API网关健康状态
Write-Host ""
Write-Host "🌐 检查API网关健康状态:" -ForegroundColor Yellow
try {
    $response = Invoke-RestMethod -Uri "http://localhost:8080/health" -Method Get -TimeoutSec 5
    Write-Host "  ✅ API网关健康检查通过" -ForegroundColor Green
    Write-Host "  📊 响应: $($response | ConvertTo-Json -Compress)" -ForegroundColor Cyan
} catch {
    Write-Host "  ❌ API网关健康检查失败: $($_.Exception.Message)" -ForegroundColor Red
}

# 检查市场数据服务
Write-Host ""
Write-Host "📈 检查市场数据服务:" -ForegroundColor Yellow
try {
    $response = Invoke-RestMethod -Uri "http://localhost:8080/api/v1/market-data/health" -Method Get -TimeoutSec 5
    Write-Host "  ✅ 市场数据服务健康检查通过" -ForegroundColor Green
} catch {
    Write-Host "  ❌ 市场数据服务健康检查失败: $($_.Exception.Message)" -ForegroundColor Red
}

# 检查WebSocket连接
Write-Host ""
Write-Host "🔌 检查WebSocket代理:" -ForegroundColor Yellow
try {
    # 简单的WebSocket连接测试
    $ws = New-Object System.Net.WebSockets.ClientWebSocket
    $uri = [System.Uri]::new("ws://localhost:8080/ws/market-data/stream")
    $cancellationToken = [System.Threading.CancellationToken]::None
    
    $connectTask = $ws.ConnectAsync($uri, $cancellationToken)
    $connectTask.Wait(5000)
    
    if ($ws.State -eq "Open") {
        Write-Host "  ✅ WebSocket代理连接成功" -ForegroundColor Green
        $ws.CloseAsync([System.Net.WebSockets.WebSocketCloseStatus]::NormalClosure, "Test complete", $cancellationToken).Wait()
    } else {
        Write-Host "  ❌ WebSocket代理连接失败" -ForegroundColor Red
    }
} catch {
    Write-Host "  ❌ WebSocket代理测试失败: $($_.Exception.Message)" -ForegroundColor Red
}

# 检查进程状态
Write-Host ""
Write-Host "⚙️ 检查关键进程:" -ForegroundColor Yellow

# 检查Rust进程
$rustProcesses = Get-Process | Where-Object { $_.ProcessName -like "*cargo*" -or $_.ProcessName -like "*gateway*" -or $_.ProcessName -like "*market-data*" -or $_.ProcessName -like "*trading-engine*" }
if ($rustProcesses) {
    Write-Host "  ✅ Rust微服务进程: $($rustProcesses.Count) 个运行中" -ForegroundColor Green
} else {
    Write-Host "  ❌ 未找到Rust微服务进程" -ForegroundColor Red
}

# 检查Node.js进程
$nodeProcesses = Get-Process | Where-Object { $_.ProcessName -like "*node*" }
if ($nodeProcesses) {
    Write-Host "  ✅ Node.js进程: $($nodeProcesses.Count) 个运行中" -ForegroundColor Green
} else {
    Write-Host "  ❌ 未找到Node.js进程" -ForegroundColor Red
}

# 系统资源使用情况
Write-Host ""
Write-Host "💻 系统资源使用情况:" -ForegroundColor Yellow
$cpu = Get-Counter '\Processor(_Total)\% Processor Time' | Select-Object -ExpandProperty CounterSamples | Select-Object -ExpandProperty CookedValue
$memory = Get-Counter '\Memory\Available MBytes' | Select-Object -ExpandProperty CounterSamples | Select-Object -ExpandProperty CookedValue
$totalMemory = (Get-CimInstance Win32_PhysicalMemory | Measure-Object -Property capacity -Sum).sum /1mb
$usedMemory = $totalMemory - $memory

Write-Host "  📊 CPU使用率: $([math]::Round($cpu, 2))%" -ForegroundColor Cyan
Write-Host "  🧠 内存使用: $([math]::Round($usedMemory, 0))MB / $([math]::Round($totalMemory, 0))MB ($([math]::Round(($usedMemory/$totalMemory)*100, 1))%)" -ForegroundColor Cyan

# 总结
Write-Host ""
Write-Host "📋 系统状态总结:" -ForegroundColor Yellow

$runningServices = 0
$totalServices = $services.Count

foreach ($service in $services) {
    $connection = Get-NetTCPConnection -LocalPort $service.Port -State Listen -ErrorAction SilentlyContinue
    if ($connection) {
        $runningServices++
    }
}

$healthPercentage = [math]::Round(($runningServices / $totalServices) * 100, 1)

if ($healthPercentage -ge 80) {
    Write-Host "  🎉 系统状态: 良好 ($healthPercentage%)" -ForegroundColor Green
} elseif ($healthPercentage -ge 60) {
    Write-Host "  ⚠️ 系统状态: 部分运行 ($healthPercentage%)" -ForegroundColor Yellow
} else {
    Write-Host "  ❌ 系统状态: 需要检查 ($healthPercentage%)" -ForegroundColor Red
}

Write-Host "  📊 运行中的服务: $runningServices / $totalServices" -ForegroundColor Cyan

Write-Host ""
Write-Host "💡 建议:" -ForegroundColor Yellow
if ($healthPercentage -lt 100) {
    Write-Host "  - 检查未运行的服务并重新启动" -ForegroundColor White
    Write-Host "  - 运行 start-full-system.ps1 启动所有服务" -ForegroundColor White
}
Write-Host "  - 访问 http://localhost:3000 查看前端界面" -ForegroundColor White
Write-Host "  - 在前端右上角查看实时系统状态" -ForegroundColor White