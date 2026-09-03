# wipe-emalupe-users.ps1 — Dry-run + execute para @emalupe.com (human-safe)
param([switch]$Execute)
$ErrorActionPreference='Stop'
Set-Location "C:\Users\leofr\Projetos\Poker_Project\Infraestrutura-Docker"
$candidates = docker --context desktop-linux exec poker_postgres psql -U user -d poker_db -t -c "SELECT COUNT(*) FROM users WHERE email ILIKE '%@%emalupe.com';"
Write-Host "Candidatos @emalupe.com: $candidates"
docker --context desktop-linux exec poker_postgres psql -U user -d poker_db -c "SELECT id::text, username, email, role, status FROM users WHERE email ILIKE '%@%emalupe.com' ORDER BY created_at DESC;"
if (-not $Execute) { Write-Host "DRY-RUN — use -Execute para apagar"; exit 0 }
$confirm = Read-Host "Digite CONFIRMO para apagar"
if ($confirm -ne "CONFIRMO") { Write-Host "Cancelado"; exit 1 }
$stamp = Get-Date -Format "yyyyMMdd_HHmmss"
$backup = "..\backups\poker_db_pre_wipe_emalupe_$stamp.sql"
docker --context desktop-linux exec poker_postgres pg_dump -U user -d poker_db > $backup
Write-Host "Backup em $backup"
Get-Content "..\scripts\wipe-emalupe-users.sql" | docker --context desktop-linux exec -i poker_postgres psql -U user -d poker_db
Write-Host "Wipe concluido. Validacao:"
docker --context desktop-linux exec poker_postgres psql -U user -d poker_db -c "SELECT COUNT(*) AS emalupe_restante FROM users WHERE email ILIKE '%@%emalupe.com'; SELECT COUNT(*) AS total FROM users;"
