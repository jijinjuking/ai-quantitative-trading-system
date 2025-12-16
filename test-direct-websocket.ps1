# 临时禁用代理测试WebSocket连接
Write-Host "Temporarily disabling proxy for WebSocket test..." -ForegroundColor Yellow

# 保存当前代理设置
$originalProxy = [System.Net.WebRequest]::DefaultWebProxy

# 禁用代理
[System.Net.WebRequest]::DefaultWebProxy = $null

Write-Host "Proxy disabled, testing direct connection..." -ForegroundColor Cyan

# 重启市场数据服务进行测试
Write-Host "Please restart the market-data service now and check the logs..." -ForegroundColor Green

# 等待用户输入
Read-Host "Press Enter to restore proxy settings"

# 恢复代理设置
[System.Net.WebRequest]::DefaultWebProxy = $originalProxy
Write-Host "Proxy settings restored" -ForegroundColor Yellow