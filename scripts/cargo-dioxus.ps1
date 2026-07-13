<#
.SYNOPSIS
    Wrapper para cargo no projeto 09-Frontend-Dioxus com configuração automática
    do linker MinGW (WinLibs) — resolve o erro "unable to find library -lgcc_eh".

.DESCRIPTION
    O toolchain stable-x86_64-pc-windows-gnu precisa das bibliotecas GCC do
    WinLibs (instalado via winget) para compilar proc-macro crates do Dioxus.
    Este script configura LIBRARY_PATH e C_INCLUDE_PATH automaticamente antes
    de executar qualquer subcomando cargo.

    REGRA DE OURO: SEMPRE use este script (ou os comandos documentados abaixo)
    ao rodar cargo no 09-Frontend-Dioxus. NUNCA rode "cargo check" diretamente
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

    Se a versão do GCC mudar (ex: 16.1.0 → 16.2.0), verificar o diretório real:
    Get-ChildItem "C:\Users\leofr\AppData\Local\Microsoft\WinGet\Packages\BrechtSanders.WinLibs.POSIX.UCRT_Microsoft.Winget.Source_8wekyb3d8bbwe\mingw64\lib\gcc\x86_64-w64-mingw32\"
#>

# Aceita qualquer subcomando cargo e seus argumentos via $args.
# Exemplos:
#   .\scripts\cargo-dioxus.ps1 check
#   .\scripts\cargo-dioxus.ps1 clippy -- -D warnings
#   .\scripts\cargo-dioxus.ps1 test
#   .\scripts\cargo-dioxus.ps1 build --release

# --- WinLibs base path (instalado via winget) ---
$winLibsBase = "C:\Users\leofr\AppData\Local\Microsoft\WinGet\Packages\BrechtSanders.WinLibs.POSIX.UCRT_Microsoft.Winget.Source_8wekyb3d8bbwe\mingw64"

# --- GCC version directory (auto-detect or use known 16.1.0) ---
$gccVersionDir = Join-Path $winLibsBase "lib\gcc\x86_64-w64-mingw32"

if (Test-Path $gccVersionDir) {
    $latestGcc = Get-ChildItem $gccVersionDir -Directory | Sort-Object Name -Descending | Select-Object -First 1
    if ($latestGcc) {
        $gccLibPath = $latestGcc.FullName
    } else {
        $gccLibPath = Join-Path $gccVersionDir "16.1.0"
    }
} else {
    $gccLibPath = Join-Path $gccVersionDir "16.1.0"
}

$env:LIBRARY_PATH = $gccLibPath
$env:C_INCLUDE_PATH = Join-Path $winLibsBase "include"

Write-Host "==> LIBRARY_PATH  = $env:LIBRARY_PATH" -ForegroundColor DarkGray
Write-Host "==> C_INCLUDE_PATH = $env:C_INCLUDE_PATH" -ForegroundColor DarkGray

# --- Navegar para 09-Frontend-Dioxus ---
$scriptRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
$projectRoot = Split-Path -Parent $scriptRoot
$dioxusDir = Join-Path $projectRoot "09-Frontend-Dioxus"

Set-Location $dioxusDir
Write-Host "==> cwd = $dioxusDir" -ForegroundColor DarkGray
Write-Host ""

# --- Executar cargo com toolchain explícito ---
# NOTA: Usamos Invoke-Expression em vez de @args porque o PowerShell
#       consome o separador '--' durante o splatting, fazendo com que
#       flags como '-D warnings' sejam passadas para o cargo em vez de
#       para o subcomando (ex: clippy).
$cargoCmd = "cargo +stable-x86_64-pc-windows-gnu $args"
Invoke-Expression $cargoCmd
exit $LASTEXITCODE
