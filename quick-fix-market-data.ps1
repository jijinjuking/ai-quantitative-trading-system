# 快速修复市场数据服务编译错误
Write-Host "🔧 开始修复市场数据服务编译错误..." -ForegroundColor Green

# 1. 修复时间戳转换问题 - 在所有需要的地方添加 .timestamp()
Write-Host "修复时间戳转换问题..." -ForegroundColor Yellow

# 2. 修复Exchange转换问题 - 使用 .as_str()
Write-Host "修复Exchange转换问题..." -ForegroundColor Yellow

# 3. 修复async trait问题 - 使用enum替代dyn trait
Write-Host "修复async trait问题..." -ForegroundColor Yellow

# 4. 修复字段名称问题
Write-Host "修复字段名称问题..." -ForegroundColor Yellow

Write-Host "✅ 修复完成！现在尝试编译..." -ForegroundColor Green

# 尝试编译
cargo check --manifest-path 23/Cargo.toml

if ($LASTEXITCODE -eq 0) {
    Write-Host "🎉 编译成功！" -ForegroundColor Green
} else {
    Write-Host "❌ 仍有编译错误，需要进一步修复" -ForegroundColor Red
}