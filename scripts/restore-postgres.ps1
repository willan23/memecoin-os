param(
  [Parameter(Mandatory = $true)]
  [string]$Dump
)
Get-Content -Raw $Dump | docker compose exec -T postgres psql -U memecoin -d memecoin_os
Write-Output "restored $Dump"
