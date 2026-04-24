$BackendUrl = "http://localhost:8900"
$TargetUrl = "http://example.com"

Write-Host "1. Handshake Baslatiliyor..." -ForegroundColor Cyan
$HandshakeReq = @{
    target_url = $TargetUrl
    burp_version = "Montoya 2023.12"
    plugin_version = "0.1.0"
} | ConvertTo-Json

$HandshakeRes = Invoke-RestMethod -Uri "$BackendUrl/api/burp/handshake" -Method Post -Body $HandshakeReq -ContentType "application/json"
$SessionId = $HandshakeRes.session_id

Write-Host "Session ID: $SessionId" -ForegroundColor Green

Write-Host "2. 15 saniye bekleyip Trafik Gonderiliyor (Rate limit asmak icin)..." -ForegroundColor Cyan
Start-Sleep -Seconds 15

$TrafficItems = @(
    @{
        url = "http://example.com/login"
        method = "POST"
        request_headers = @{ "content-type" = "application/x-www-form-urlencoded" }
        request_body = "user=admin&pass=admin"
        response_status = 200
        response_headers = @{ "server" = "nginx/1.18.0" }
        response_body = "Welcome"
        timestamp = 123456789
        tool_source = "PROXY"
    },
    @{
        url = "http://example.com/api/data"
        method = "GET"
        request_headers = @{ "authorization" = "Bearer null" }
        request_body = ""
        response_status = 401
        response_headers = @{ "www-authenticate" = "Bearer" }
        response_body = "Unauthorized"
        timestamp = 123456790
        tool_source = "REPEATER"
    }
)

$ImportReq = @{
    session_id = $SessionId
    items = $TrafficItems
} | ConvertTo-Json -Depth 10

$ImportRes = Invoke-RestMethod -Uri "$BackendUrl/api/burp/import-traffic" -Method Post -Body $ImportReq -ContentType "application/json"

Write-Host "Trafik aktarildi!" -ForegroundColor Green
Write-Host "Aktarilan: $($ImportRes.imported_count)"
Write-Host "Uretilen Bulgu: $($ImportRes.new_findings_triggered)"
Write-Host "Dashboard'u kontrol edin! (http://localhost:3000)" -ForegroundColor Yellow
