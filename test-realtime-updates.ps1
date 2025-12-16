# Test Real-time Data Updates
Write-Host "Testing real-time data updates..." -ForegroundColor Yellow

Write-Host "`nMonitoring price changes (every 2 seconds)..." -ForegroundColor Cyan

for ($i = 1; $i -le 10; $i++) {
    try {
        $response = Invoke-WebRequest -Uri "http://localhost:8081/api/v1/tickers" | ConvertFrom-Json
        $btc = $response.data | Where-Object {$_.symbol -eq "BTCUSDT"}
        
        if ($btc) {
            $timestamp = Get-Date -Format "HH:mm:ss"
            Write-Host "[$timestamp] BTC: $($btc.price) ($($btc.change)%)" -ForegroundColor Green
        } else {
            Write-Host "[$timestamp] BTC data not found" -ForegroundColor Red
        }
    } catch {
        Write-Host "[$timestamp] API request failed: $($_.Exception.Message)" -ForegroundColor Red
    }
    
    if ($i -lt 10) {
        Start-Sleep -Seconds 2
    }
}

Write-Host "`nChecking K-line data..." -ForegroundColor Cyan
try {
    $klines = Invoke-WebRequest -Uri "http://localhost:8081/api/v1/klines" | ConvertFrom-Json
    if ($klines.success) {
        $count = $klines.data.Count
        $latest = $klines.data | Select-Object -First 1
        Write-Host "K-line data: $count records" -ForegroundColor Green
        Write-Host "Latest K-line: O:$($latest.open) H:$($latest.high) L:$($latest.low) C:$($latest.close)" -ForegroundColor Green
    } else {
        Write-Host "K-line data fetch failed: $($klines.error)" -ForegroundColor Red
    }
} catch {
    Write-Host "K-line API request failed: $($_.Exception.Message)" -ForegroundColor Red
}

Write-Host "`nTest completed" -ForegroundColor Yellow