[CmdletBinding(SupportsShouldProcess = $true)]
param(
    [Parameter(Mandatory = $true)]
    [ValidatePattern('^[A-Za-z0-9.-]+$')]
    [string]$VpsHost,

    [ValidatePattern('^[A-Za-z0-9._-]+$')]
    [string]$VpsUser = 'root',

    [string]$IdentityFile = '',

    [ValidatePattern('^/[A-Za-z0-9._/-]+$')]
    [string]$RemoteEnvPath = '/opt/poker-platform/Infraestrutura-Docker/.env',

    [Parameter(Mandatory = $true)]
    [string[]]$AllowedDepositorIds,

    [ValidateRange(500, 600000)]
    [int]$MaxDepositCents = 100000,

    [ValidatePattern('^https://[A-Za-z0-9.-]+$')]
    [string]$PublicBaseUrl = 'https://zerotiltpoker.net',

    [Parameter(Mandatory = $true)]
    [switch]$ConfirmMerchantOnlyScopes,

    [switch]$Apply
)

$ErrorActionPreference = 'Stop'

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
    if ($Value.Contains("`r") -or $Value.Contains("`n")) {
        throw "Valor inválido para $Name"
    }
    $line = "$Name=$Value"
    $pattern = '(?m)^' + [Regex]::Escape($Name) + '=.*$'
    $regex = [Regex]::new($pattern)
    if ($regex.IsMatch($Content)) {
        return $regex.Replace($Content, [System.Text.RegularExpressions.MatchEvaluator]{ param($match) $line }, 1)
    }
    if (-not $Content.EndsWith("`n")) { $Content += "`n" }
    return $Content + $line + "`n"
}

if (-not $ConfirmMerchantOnlyScopes) {
    throw 'Confirme que a chave possui somente merchant_read e merchant_write usando -ConfirmMerchantOnlyScopes.'
}
if ($AllowedDepositorIds.Count -eq 0) {
    throw 'Informe ao menos um UUID em -AllowedDepositorIds para o rollout inicial.'
}
foreach ($id in $AllowedDepositorIds) {
    $parsed = [Guid]::Empty
    if (-not [Guid]::TryParse($id, [ref]$parsed)) {
        throw "UUID de depositante inválido: $id"
    }
}
if ($IdentityFile -and -not (Test-Path -LiteralPath $IdentityFile -PathType Leaf)) {
    throw "Chave SSH não encontrada: $IdentityFile"
}

Write-Host 'Ativação segura DePix live na VPS' -ForegroundColor Cyan
Write-Host 'Os segredos não serão exibidos nem enviados ao chat.'
$apiKey = Read-SecretText 'Cole a chave DePix sk_live_'
$webhookSecret = Read-SecretText 'Cole o Webhook Secret da DePix'
if ($apiKey -notmatch '^sk_live_[A-Za-z0-9_-]+$' -or $apiKey -match '\s') {
    throw 'A chave deve começar com sk_live_ e não pode conter espaços.'
}
if ($webhookSecret.Length -lt 24 -or $webhookSecret -match '\s') {
    throw 'O Webhook Secret deve ter pelo menos 24 caracteres e não pode conter espaços.'
}

$headers = @{ Authorization = "Bearer $apiKey" }
try {
    $merchant = Invoke-RestMethod -Method Get -Uri 'https://api.depixapp.com/api/me' -Headers $headers -TimeoutSec 20
    $verification = Invoke-RestMethod -Method Get -Uri 'https://api.depixapp.com/api/verification' -Headers $headers -TimeoutSec 20
}
catch {
    throw 'A DePix recusou a chave live ou a consulta de verificação. Nenhum arquivo foi alterado.'
}
if ($merchant.is_live -ne $true) {
    throw 'A chave não foi reconhecida como live. Nenhum arquivo foi alterado.'
}
if ($verification.verified -ne $true) {
    throw 'A conta DePix ainda não está verificada para produção. Nenhum arquivo foi alterado.'
}
if (-not $Apply) {
    Write-Host "Chave live válida para '$($merchant.name)'. Execute novamente com -Apply para instalar na VPS." -ForegroundColor Green
    exit 0
}

$target = "$VpsUser@$VpsHost"
$sshOptions = @('-o', 'BatchMode=yes', '-o', 'StrictHostKeyChecking=accept-new')
if ($IdentityFile) { $sshOptions += @('-i', (Resolve-Path -LiteralPath $IdentityFile).Path) }
$tempDir = Join-Path ([System.IO.Path]::GetTempPath()) ('poker-depix-' + [Guid]::NewGuid().ToString('N'))
$localEnv = Join-Path $tempDir '.env'
$remoteNext = "$RemoteEnvPath.depix.next"
$timestamp = [DateTime]::UtcNow.ToString('yyyyMMddTHHmmssZ')
$remoteBackup = "$RemoteEnvPath.bak.$timestamp"

try {
    New-Item -ItemType Directory -Path $tempDir | Out-Null
    & ssh @sshOptions $target "test -f $RemoteEnvPath"
    if ($LASTEXITCODE -ne 0) { throw 'O .env remoto não foi encontrado.' }
    & scp @sshOptions "${target}:$RemoteEnvPath" $localEnv
    if ($LASTEXITCODE -ne 0) { throw 'Falha ao baixar o .env remoto para atualização protegida.' }

    $content = [System.IO.File]::ReadAllText($localEnv)
    $content = Set-EnvValue $content 'ENVIRONMENT' 'production'
    $content = Set-EnvValue $content 'PIX_PROVIDER' 'depix'
    $content = Set-EnvValue $content 'PIX_MODE' 'production'
    $content = Set-EnvValue $content 'PIX_LIVE_ENABLED' 'true'
    $content = Set-EnvValue $content 'PIX_LIVE_ALLOWED_DEPOSITOR_IDS' (($AllowedDepositorIds | ForEach-Object { $_.Trim() }) -join ',')
    $content = Set-EnvValue $content 'DEPIX_LIVE_MAX_DEPOSIT_CENTS' $MaxDepositCents.ToString()
    $content = Set-EnvValue $content 'DEPIX_API_BASE_URL' 'https://api.depixapp.com'
    $content = Set-EnvValue $content 'DEPIX_API_KEY' $apiKey
    $content = Set-EnvValue $content 'DEPIX_WEBHOOK_SECRET' $webhookSecret
    $content = Set-EnvValue $content 'DEPIX_CALLBACK_URL' "$PublicBaseUrl/api/webhooks/pix"
    $content = Set-EnvValue $content 'DEPIX_REDIRECT_URL' "$PublicBaseUrl/wallet"
    $content = Set-EnvValue $content 'PLAY_MONEY_PIX_KEY' ''
    [System.IO.File]::WriteAllText($localEnv, $content, [System.Text.UTF8Encoding]::new($false))

    & ssh @sshOptions $target "cp -p -- $RemoteEnvPath $remoteBackup"
    if ($LASTEXITCODE -ne 0) { throw 'Falha ao criar backup do .env remoto.' }
    & scp @sshOptions $localEnv "${target}:$remoteNext"
    if ($LASTEXITCODE -ne 0) { throw 'Falha ao enviar a configuração DePix para a VPS.' }
    & ssh @sshOptions $target "chmod 600 $remoteNext && mv -f -- $remoteNext $RemoteEnvPath"
    if ($LASTEXITCODE -ne 0) { throw 'Falha ao ativar o .env novo; o backup remoto foi preservado.' }

    Write-Host "Configuração live instalada. Backup remoto: $remoteBackup" -ForegroundColor Green
    Write-Host 'Recrie a API somente depois que o código com o gate DePix live estiver implantado.' -ForegroundColor Yellow
}
finally {
    $apiKey = $null
    $webhookSecret = $null
    $headers = $null
    if (Test-Path -LiteralPath $tempDir) {
        Remove-Item -LiteralPath $tempDir -Recurse -Force
    }
}
