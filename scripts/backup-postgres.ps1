param(
  [string]$OutDir = "backups"
)
New-Item -ItemType Directory -Force -Path $OutDir | Out-Null
$stamp = Get-Date -Format "yyyyMMdd-HHmmss"
$out = Join-Path $OutDir "memecoin_os-$stamp.sql"
docker compose exec -T postgres pg_dump -U memecoin memecoin_os | Set-Content -Encoding utf8 $out
Write-Output "wrote $out"
