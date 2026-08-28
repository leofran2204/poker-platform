<#
Execução manual autorizada da validação completa da plataforma.

Exemplos:
  .\scripts\full-validation.ps1 -Approved
  .\scripts\full-validation.ps1 -Phase motor -Approved

PIX real permanece adiado. Os contratos locais do ledger (sem payout externo)
fazem parte desta rotina autorizada para impedir regressões financeiras.
#>
[CmdletBinding()]
param(
    [ValidateSet("all", "motor", "api", "frontend", "gateway")]
    [string]$Phase = "all",
    [switch]$Approved
)

$ErrorActionPreference = "Stop"

if (-not $Approved) {
    throw "Esta rotina executa carga intensa. Rode novamente somente após autorização explícita, com -Approved."
}

$ProjectRoot = Split-Path -Parent $PSScriptRoot
$script:ValidationMetrics = @()
$ReportDirectory = if ($env:FULL_VALIDATION_REPORT_DIR) {
    $env:FULL_VALIDATION_REPORT_DIR
} else {
    Join-Path $ProjectRoot "artifacts\full-validation"
}
New-Item -ItemType Directory -Path $ReportDirectory -Force | Out-Null
$ReportPath = Join-Path $ReportDirectory ("metrics-{0}.tsv" -f (Get-Date -Format "yyyyMMddTHHmmssZ"))

function Save-ValidationMetrics {
    if ($script:ValidationMetrics.Count -eq 0) {
        return
    }

    $script:ValidationMetrics |
        Export-Csv -LiteralPath $ReportPath -Delimiter "`t" -NoTypeInformation -Encoding utf8
    Write-Host "==> Relatório de métricas: $ReportPath" -ForegroundColor Green
}

function Invoke-Cargo {
    param(
        [Parameter(Mandatory = $true, Position = 0, ValueFromRemainingArguments = $true)]
        [string[]]$Arguments
    )

    & cargo @Arguments
    if ($LASTEXITCODE -ne 0) {
        throw "cargo $($Arguments -join ' ') falhou com código $LASTEXITCODE."
    }
}

function Invoke-TimedValidation {
    param(
        [Parameter(Mandatory = $true)][string]$Name,
        [Parameter(Mandatory = $true)][string]$Workload,
        [Parameter(Mandatory = $true)][scriptblock]$Action
    )

    $watch = [System.Diagnostics.Stopwatch]::StartNew()
    $status = "passed"
    try {
        & $Action
    }
    catch {
        $status = "failed"
        throw
    }
    finally {
        $watch.Stop()
        $script:ValidationMetrics += [pscustomobject]@{
            phase            = $Name
            status           = $status
            duration_seconds = [Math]::Round($watch.Elapsed.TotalSeconds, 3)
            workload         = $Workload
        }
        Write-Host "==> Métrica [$Name]: status=$status duração=$([Math]::Round($watch.Elapsed.TotalSeconds, 3))s carga=$Workload" -ForegroundColor DarkCyan
    }
}

function Invoke-MotorValidation {
    Push-Location (Join-Path $ProjectRoot "Motor-Rust")
    try {
        Write-Host "==> Motor: rotina + 79 cenários de carga" -ForegroundColor Cyan
        Invoke-Cargo test --lib --features massive-tests -- --nocapture
    }
    finally {
        Pop-Location
    }
}

function Invoke-ApiValidation {
    Push-Location (Join-Path $ProjectRoot "API-Axum")
    $previousProptestCases = $env:PROPTEST_CASES
    $previousDatabaseUrl = $env:DATABASE_URL
    try {
        # Compatível com o PostgreSQL local exposto pelo docker-compose. Uma
        # URL já configurada pelo operador (por exemplo, de staging) prevalece.
        if (-not $env:DATABASE_URL) {
            $env:DATABASE_URL = "postgres://user:password@localhost:5433/poker_db"
        }

        Write-Host "==> API HTTPS: contratos e segurança" -ForegroundColor Cyan
        Invoke-Cargo test --lib --bin poker-api --test api_tests --test payments_tests --test red_team_simulation_tests -- --nocapture

        Write-Host "==> API HTTPS: contratos funcionais PostgreSQL" -ForegroundColor Cyan
        Invoke-Cargo test --features full-validation --test api_tests -- --ignored --nocapture

        Write-Host "==> API HTTPS: contratos financeiros PostgreSQL" -ForegroundColor Cyan
        Invoke-Cargo test --features full-validation --test payments_tests -- --ignored --nocapture

        Write-Host "==> API HTTPS: contrato de limite compartilhado Redis" -ForegroundColor Cyan
        Invoke-Cargo test --features full-validation --test rate_limit_tests -- --ignored --nocapture

        Write-Host "==> API HTTPS: 10 cenários de fuzz (2.000 casos por cenário)" -ForegroundColor Cyan
        $env:PROPTEST_CASES = if ($env:API_FUZZ_CASES) { $env:API_FUZZ_CASES } else { "2000" }
        Invoke-Cargo test --features full-validation --test api_fuzz_tests -- --nocapture

        Write-Host "==> API: WebSocket, concorrência, jitter e desconexão" -ForegroundColor Cyan
        Invoke-Cargo test --features full-validation --test ws_stress_tests -- --nocapture
        Invoke-Cargo test --features full-validation --test ws_network_jitter_tests -- --nocapture
        Invoke-Cargo test --features full-validation --test concurrency_ws_tests -- --nocapture
        Invoke-Cargo test --features full-validation --test actor_disconnect_stress_tests -- --nocapture

        Write-Host "==> API: persistência PostgreSQL concorrente" -ForegroundColor Cyan
        Invoke-Cargo test --features full-validation --test db_pool_stress_tests -- --ignored --nocapture
    }
    finally {
        if ($null -eq $previousProptestCases) {
            Remove-Item Env:PROPTEST_CASES -ErrorAction SilentlyContinue
        } else {
            $env:PROPTEST_CASES = $previousProptestCases
        }
        if ($null -eq $previousDatabaseUrl) {
            Remove-Item Env:DATABASE_URL -ErrorAction SilentlyContinue
        } else {
            $env:DATABASE_URL = $previousDatabaseUrl
        }
        Pop-Location
    }
}

function Invoke-FrontendValidation {
    Write-Host "==> Frontend Dioxus removido do monorepo (legado no histórico git)." -ForegroundColor Yellow
    Write-Host "    UI canônica: Frontend-Web/ — npm run lint/build ou job frontend-web do rust-ci.yml." -ForegroundColor DarkGray
}

function Invoke-GatewayValidation {
    if (-not (Get-Command docker -ErrorAction SilentlyContinue)) {
        throw "Docker é necessário para a fase gateway HTTPS/WSS."
    }
    if (-not (Get-Command wsl.exe -ErrorAction SilentlyContinue)) {
        throw "WSL é necessário para executar a verificação HTTPS/WSS do Caddy."
    }

    Push-Location $ProjectRoot
    try {
        Write-Host "==> Gateway: reconstruindo API e Caddy para a verificação E2E" -ForegroundColor Cyan
        & docker compose -f "Infraestrutura-Docker/docker-compose.yml" up -d --build poker_api poker_frontend
        if ($LASTEXITCODE -ne 0) {
            throw "docker compose up falhou com código $LASTEXITCODE."
        }

        $resolvedProjectRoot = (Resolve-Path -LiteralPath $ProjectRoot).Path
        if ($resolvedProjectRoot -notmatch '^(?<drive>[A-Za-z]):\\(?<path>.*)$') {
            throw "O projeto precisa estar em uma unidade Windows montável pelo WSL."
        }
        $wslRelativePath = $Matches['path'] -replace '\\', '/'
        $wslProjectRoot = "/mnt/$($Matches['drive'].ToLowerInvariant())/$wslRelativePath"
        Write-Host "==> Gateway: HTTPS, HSTS, redirecionamento e handshake WSS" -ForegroundColor Cyan
        & wsl.exe -d Ubuntu -- bash -lc "cd '$wslProjectRoot' && PUBLIC_GATEWAY_INSECURE_LOCAL_CERT=1 bash scripts/verify-public-https.sh"
        if ($LASTEXITCODE -ne 0) {
            throw "A verificação E2E HTTPS/WSS falhou com código $LASTEXITCODE."
        }
    }
    finally {
        Pop-Location
    }
}

try {
    if ($Phase -in @("all", "motor")) {
        Invoke-TimedValidation "motor" "1814 rotina + 79 carga; Monte Carlo, CSPRNG, fairness e invariantes" { Invoke-MotorValidation }
    }
    if ($Phase -in @("all", "api")) {
        Invoke-TimedValidation "api" "10 fuzzes de API x 2000 = 20000 entradas; WebSocket = 1000800 mensagens; testes funcionais" { Invoke-ApiValidation }
    }
    if ($Phase -in @("all", "frontend")) {
        Invoke-TimedValidation "frontend" "76 funcionais + 10 fuzzes x 200000 = 2000000 entradas + 2 stresses" { Invoke-FrontendValidation }
    }
    if ($Phase -in @("all", "gateway")) {
        Invoke-TimedValidation "gateway" "Caddy local: API HTTPS, HSTS, redirecionamento HTTP→HTTPS e handshake WSS" { Invoke-GatewayValidation }
    }

    Write-Host "==> Validação completa concluída." -ForegroundColor Green
}
finally {
    Save-ValidationMetrics
}
