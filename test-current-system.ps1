# 测试当前运行的系统状态
Write-Host "🔍 测试当前系统状态..." -ForegroundColor Green

# 测试前端
Write-Host ""
Write-Host "🎨 测试前端服务 (端口3000):" -ForegroundColor Yellow
try {
    $response = Invoke-RestMethod -Uri "http://localhost:3000" -Method Get -TimeoutSec 5
    Write-Host "  ✅ 前端服务正常运行" -ForegroundColor Green
} catch {
    Write-Host "  ❌ 前端服务异常: $($_.Exception.Message)" -ForegroundColor Red
}

# 测试网关服务
Write-Host ""
Write-Host "🌐 测试网关服务 (端口8080):" -ForegroundColor Yellow
try {
    $connection = Get-NetTCPConnection -LocalPort 8080 -State Listen -ErrorAction SilentlyContinue
    if ($connection) {
        Write-Host "  ✅ 网关服务端口监听正常" -ForegroundColor Green
    } else {
        Write-Host "  ❌ 网关服务端口未监听" -ForegroundColor Red
    }
} catch {
    Write-Host "  ❌ 网关服务检查失败" -ForegroundColor Red
}

# 测试市场数据服务
Write-Host ""
Write-Host "📈 测试市场数据服务 (端口8081):" -ForegroundColor Yellow
try {
    # 健康检查
    $health = Invoke-RestMethod -Uri "http://localhost:8081/health" -Method Get -TimeoutSec 5
    Write-Host "  ✅ 健康检查通过: $($health.status)" -ForegroundColor Green
    
    # 测试价格数据
    $tickers = Invoke-RestMethod -Uri "http://localhost:8081/api/v1/tickers" -Method Get -TimeoutSec 5
    Write-Host "  ✅ 价格数据API正常，返回 $($tickers.data.Count) 条数据" -ForegroundColor Green
    
    # 测试K线数据
    $klines = Invoke-RestMethod -Uri "http://localhost:8081/api/v1/klines" -Method Get -TimeoutSec 5
    Write-Host "  ✅ K线数据API正常，返回 $($klines.data.Count) 条数据" -ForegroundColor Green
    
} catch {
    Write-Host "  ❌ 市场数据服务异常: $($_.Exception.Message)" -ForegroundColor Red
}

# 测试WebSocket连接
Write-Host ""
Write-Host "🔌 测试WebSocket连接:" -ForegroundColor Yellow
try {
    # 简单的WebSocket连接测试
    $ws = New-Object System.Net.WebSockets.ClientWebSocket
    $uri = [System.Uri]::new("ws://localhost:8080/ws/market-data/stream")
    $cancellationToken = [System.Threading.CancellationToken]::None
    
    $connectTask = $ws.ConnectAsync($uri, $cancellationToken)
    $connectTask.Wait(5000)
    
    if ($ws.State -eq "Open") {
        Write-Host "  ✅ WebSocket连接成功" -ForegroundColor Green
        $ws.CloseAsync([System.Net.WebSockets.WebSocketCloseStatus]::NormalClosure, "Test complete", $cancellationToken).Wait()
    } else {
        Write-Host "  ❌ WebSocket连接失败，状态: $($ws.State)" -ForegroundColor Red
    }
} catch {
    Write-Host "  ❌ WebSocket测试失败: $($_.Exception.Message)" -ForegroundColor Red
}

# 显示示例数据
Write-Host ""
Write-Host "📊 示例数据展示:" -ForegroundColor Yellow
try {
    $tickers = Invoke-RestMethod -Uri "http://localhost:8081/api/v1/tickers" -Method Get -TimeoutSec 5
    $firstTicker = $tickers.data[0]
    Write-Host "  交易对: $($firstTicker.symbol)" -ForegroundColor Cyan
    Write-Host "  价格: $($firstTicker.price)" -ForegroundColor Cyan
    Write-Host "  涨跌: $($firstTicker.change)%" -ForegroundColor Cyan
    Write-Host "  时间: $($firstTicker.timestamp)" -ForegroundColor Cyan
} catch {
    Write-Host "  ❌ 无法获取示例数据" -ForegroundColor Red
}

Write-Host ""
Write-Host "🎉 系统测试完成！" -ForegroundColor Green
Write-Host ""
Write-Host "💡 访问地址:" -ForegroundColor Yellow
Write-Host "  🎨 前端界面: http://localhost:3000" -ForegroundColor White
Write-Host "  📈 市场数据: http://localhost:8081/api/v1/tickers" -ForegroundColor White
Write-Host "  🏥 健康检查: http://localhost:8081/health" -ForegroundColor White