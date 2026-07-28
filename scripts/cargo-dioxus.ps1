<#
.SYNOPSIS
    Wrapper para cargo no projeto Frontend-Dioxus com configuração automática
    do linker MinGW (WinLibs) — resolve o erro "unable to find library -lgcc_eh".

.DESCRIPTION
    O toolchain stable-x86_64-pc-windows-gnu precisa das bibliotecas GCC do
    WinLibs (instalado via winget) para compilar proc-macro crates do Dioxus.
    Este script configura LIBRARY_PATH e C_INCLUDE_PATH automaticamente antes
    de executar qualquer subcomando cargo.

    REGRA DE OURO: SEMPRE use este script (ou os comandos documentados abaixo)
    ao rodar cargo no Frontend-Dioxus. NUNCA rode "cargo check" diretamente
    sem configurar as variáveis de ambiente — vai falhar com erro de linker.

.EXAMPLE
    .\scripts\cargo-dioxus.ps1 check
    .\scripts\cargo-dioxus.ps1 --% clippy -- -D warnings
    .\scripts\cargo-dioxus.ps1 test
    .\scripts\cargo-dioxus.ps1 build --release

.NOTES
    IMPORTANTE: O PowerShell consome o separador '--' ao chamar scripts .ps1.
    Para passar '--' para o cargo (ex: clippy -- -D warnings), use --%:
        .\scripts\cargo-dioxus.ps1 --% clippy -- -D warnings
    Para comandos sem '--' (check, test, build --release), chame normalmente.

    O diretório do WinLibs é descoberto automaticamente a partir de LOCALAPPDATA.
#>

# Aceita qualquer subcomando cargo e seus argumentos via $args.
# Exemplos:
#   .\scripts\cargo-dioxus.ps1 check
#   .\scripts\cargo-dioxus.ps1 clippy -- -D warnings
#   .\scripts\cargo-dioxus.ps1 test
#   .\scripts\cargo-dioxus.ps1 build --release

# --- WinLibs base path (instalado via winget) ---
$packagesDir = Join-Path $env:LOCALAPPDATA "Microsoft\WinGet\Packages"
$winLibsPackage = Get-ChildItem -Path $packagesDir -Directory -Filter "BrechtSanders.WinLibs.POSIX.UCRT_*" -ErrorAction SilentlyContinue |
    Sort-Object Name -Descending |
    Select-Object -First 1

if (-not $winLibsPackage) {
    throw "WinLibs não encontrado em $packagesDir. Instale BrechtSanders.WinLibs.POSIX.UCRT pelo winget."
}

$winLibsBase = Join-Path $winLibsPackage.FullName "mingw64"

# --- GCC version directory (auto-detect) ---
$gccVersionDir = Join-Path $winLibsBase "lib\gcc\x86_64-w64-mingw32"

if (-not (Test-Path $gccVersionDir)) {
    throw "Diretório de bibliotecas GCC não encontrado: $gccVersionDir"
}
$latestGcc = Get-ChildItem $gccVersionDir -Directory | Sort-Object Name -Descending | Select-Object -First 1
if (-not $latestGcc) {
    throw "Nenhuma versão GCC foi encontrada em $gccVersionDir"
}
$gccLibPath = $latestGcc.FullName

$env:LIBRARY_PATH = "$gccLibPath;$env:LIBRARY_PATH"
$env:C_INCLUDE_PATH = Join-Path $winLibsBase "include"

Write-Host "==> LIBRARY_PATH  = $env:LIBRARY_PATH" -ForegroundColor DarkGray
Write-Host "==> C_INCLUDE_PATH = $env:C_INCLUDE_PATH" -ForegroundColor DarkGray

# --- Navegar para Frontend-Dioxus ---
$scriptRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
$projectRoot = Split-Path -Parent $scriptRoot
$dioxusDir = Join-Path $projectRoot "Frontend-Dioxus"

if (-not (Test-Path $dioxusDir)) {
    throw "Diretório do frontend não encontrado: $dioxusDir"
}

Set-Location $dioxusDir
Write-Host "==> cwd = $dioxusDir" -ForegroundColor DarkGray
Write-Host ""

# --- Executar cargo com toolchain explícito ---
# O operador de chamada preserva os argumentos e evita interpretar texto
# fornecido pelo usuário como código PowerShell.
& cargo +stable-x86_64-pc-windows-gnu @args
exit $LASTEXITCODE
