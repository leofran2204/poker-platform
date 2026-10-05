<# Local isolated gameplay campaign; no public gateway or demo load. #>
[CmdletBinding()]
param(
    [ValidateSet("all", "motor", "api", "gateway", "frontend", "pineapple", "pineapple-cash")]
    [string]$Phase = "all",
    [ValidateRange(1, 60)]
    [int]$Minutes = 60,
    [switch]$Approved
)
$ErrorActionPreference = "Stop"
if (-not $Approved) { throw "Explicit authorization required: -Approved." }
$ProjectRoot = Split-Path -Parent $PSScriptRoot
$LinuxRoot = (& wsl.exe -d Ubuntu -- wslpath -a $ProjectRoot).Trim()
if ($LASTEXITCODE -ne 0) { throw "WSL Ubuntu is required by AGENTS.md." }
& wsl.exe -d Ubuntu -- env "FULL_VALIDATION_APPROVED=1" "FULL_VALIDATION_MINUTES=$Minutes" bash "$LinuxRoot/scripts/full-validation.sh" $Phase
if ($LASTEXITCODE -ne 0) { throw "Campaign failed or incomplete (exit $LASTEXITCODE). See artifacts/full-validation/." }
