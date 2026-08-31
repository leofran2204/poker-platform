[CmdletBinding()]
param(
    [string]$AllowedDepositorId = ""
)

$ErrorActionPreference = "Stop"
$projectRoot = Split-Path -Parent $PSScriptRoot
$envPath = Join-Path $projectRoot "Infraestrutura-Docker/.env"
$examplePath = Join-Path $projectRoot "Infraestrutura-Docker/.env.example"

function Read-SecretText([string]$Prompt) {
    $secure = Read-Host $Prompt -AsSecureString
    $pointer = [Runtime.InteropServices.Marshal]::SecureStringToBSTR($secure)
    try {
        return [Runtime.InteropServices.Marshal]::PtrToStringBSTR($pointer)
    }
    finally {
        [Runtime.InteropServices.Marshal]::ZeroFreeBSTR($pointer)
    }
}

function Set-EnvValue([string]$Content, [string]$Name, [string]$Value) {
    $line = "$Name=$Value"
    $pattern = "(?m)^" + [Regex]::Escape($Name) + "=.*$"
    $regex = [Regex]::new($pattern)
    if ($regex.IsMatch($Content)) {
        return $regex.Replace($Content, $line, 1)
    }
    if (-not $Content.EndsWith("`n")) {
        $Content += "`n"
    }
    return $Content + $line + "`n"
}

Write-Host "Configuração local segura da DePix (sandbox)" -ForegroundColor Cyan
Write-Host "A chave e o webhook secret não serão exibidos nem enviados ao chat."

$apiKey = Read-SecretText "Cole a chave sk_test_ da DePix"
if ($apiKey -notmatch '^sk_test_[A-Za-z0-9_-]+$' -or $apiKey -match '\s') {
    throw "A chave deve começar com sk_test_ e não pode conter espaços."
}

$webhookSecret = Read-SecretText "Cole o Webhook Secret da DePix"
if ($webhookSecret.Length -lt 24 -or $webhookSecret -match '\s') {
    throw "O Webhook Secret deve ter pelo menos 24 caracteres e não pode conter espaços."
}

$headers = @{ Authorization = "Bearer $apiKey" }
try {
    $merchant = Invoke-RestMethod -Method Get -Uri "https://api.depixapp.com/api/me" -Headers $headers -TimeoutSec 20
}
catch {
    throw "A DePix recusou a chave de teste. Confirme a chave e tente novamente."
}
if ($merchant.is_live -ne $false) {
    throw "A chave informada não foi reconhecida como sandbox. Nenhuma configuração foi gravada."
}

if (-not (Test-Path -LiteralPath $envPath)) {
    if (-not (Test-Path -LiteralPath $examplePath)) {
        throw "Arquivo de exemplo não encontrado: $examplePath"
    }
    Copy-Item -LiteralPath $examplePath -Destination $envPath
}

$content = [System.IO.File]::ReadAllText($envPath)
$content = Set-EnvValue $content "PIX_PROVIDER" "depix"
$content = Set-EnvValue $content "PIX_MODE" "sandbox"
$content = Set-EnvValue $content "DEPIX_API_BASE_URL" "https://api.depixapp.com"
$content = Set-EnvValue $content "DEPIX_API_KEY" $apiKey
$content = Set-EnvValue $content "DEPIX_WEBHOOK_SECRET" $webhookSecret
$content = Set-EnvValue $content "DEPIX_CALLBACK_URL" ""
$content = Set-EnvValue $content "DEPIX_REDIRECT_URL" ""
if (-not [string]::IsNullOrWhiteSpace($AllowedDepositorId)) {
    if ($AllowedDepositorId -notmatch '^[0-9a-fA-F-]{36}$') {
        throw "AllowedDepositorId deve ser um UUID interno válido."
    }
    $content = Set-EnvValue $content "PIX_ALLOWED_DEPOSITOR_IDS" $AllowedDepositorId.Trim()
}

$tempPath = Join-Path (Split-Path -Parent $envPath) (".env.depix." + [Guid]::NewGuid().ToString("N") + ".tmp")
try {
    [System.IO.File]::WriteAllText($tempPath, $content, [System.Text.UTF8Encoding]::new($false))
    Move-Item -LiteralPath $tempPath -Destination $envPath -Force
}
finally {
    if (Test-Path -LiteralPath $tempPath) {
        Remove-Item -LiteralPath $tempPath -Force
    }
    $apiKey = $null
    $webhookSecret = $null
    $headers = $null
}

Write-Host "Chave validada para o merchant '$($merchant.name)' e salva no .env local ignorado pelo Git." -ForegroundColor Green
if ([string]::IsNullOrWhiteSpace($AllowedDepositorId)) {
    Write-Host "A integração permanece bloqueada até configurar PIX_ALLOWED_DEPOSITOR_IDS com o UUID do usuário de teste." -ForegroundColor Yellow
}
