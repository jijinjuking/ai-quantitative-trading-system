# Test WebSocket Connection
Write-Host "Testing Binance WebSocket Connection..." -ForegroundColor Yellow

# Test DNS resolution
Write-Host "`n1. Testing DNS resolution..." -ForegroundColor Cyan
try {
    $dns = Resolve-DnsName -Name "stream.binance.com" -Type A
    Write-Host "DNS resolution successful: $($dns.IPAddress)" -ForegroundColor Green
} catch {
    Write-Host "DNS resolution failed: $($_.Exception.Message)" -ForegroundColor Red
}

# Test TCP connection
Write-Host "`n2. Testing TCP connection..." -ForegroundColor Cyan
try {
    $tcpClient = New-Object System.Net.Sockets.TcpClient
    $tcpClient.Connect("stream.binance.com", 9443)
    if ($tcpClient.Connected) {
        Write-Host "TCP connection successful" -ForegroundColor Green
        $tcpClient.Close()
    }
} catch {
    Write-Host "TCP connection failed: $($_.Exception.Message)" -ForegroundColor Red
}

# Test HTTPS connection
Write-Host "`n3. Testing HTTPS connection..." -ForegroundColor Cyan
try {
    $response = Invoke-WebRequest -Uri "https://stream.binance.com" -TimeoutSec 10
    Write-Host "HTTPS connection successful: $($response.StatusCode)" -ForegroundColor Green
} catch {
    Write-Host "HTTPS connection failed: $($_.Exception.Message)" -ForegroundColor Red
}

# Check proxy settings
Write-Host "`n4. Checking proxy settings..." -ForegroundColor Cyan
$proxy = [System.Net.WebRequest]::GetSystemWebProxy()
$proxyUri = $proxy.GetProxy("https://stream.binance.com")
if ($proxyUri.ToString() -ne "https://stream.binance.com/") {
    Write-Host "Proxy detected: $($proxyUri)" -ForegroundColor Yellow
} else {
    Write-Host "No proxy settings" -ForegroundColor Green
}

# Test market data service
Write-Host "`n5. Testing market data service..." -ForegroundColor Cyan
try {
    $health = Invoke-WebRequest -Uri "http://localhost:8081/health" | ConvertFrom-Json
    Write-Host "Service health status: $($health.status)" -ForegroundColor Green
} catch {
    Write-Host "Service connection failed: $($_.Exception.Message)" -ForegroundColor Red
}

Write-Host "`nTest completed" -ForegroundColor Yellow