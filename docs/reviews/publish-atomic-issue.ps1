param([Parameter(Mandatory=$true)][string]$Id)
$ErrorActionPreference = 'Stop'
$planPath = Join-Path $PSScriptRoot 'github-atomic-plan-2026-10-05.json'
$manifestPath = Join-Path $PSScriptRoot 'github-atomic-publication-2026-10-05.jsonl'
$spec = Get-Content -LiteralPath $planPath -Raw | ConvertFrom-Json | Where-Object id -EQ $Id
if (-not $spec) { throw "Unknown issue: $Id" }
$prior = if (Test-Path -LiteralPath $manifestPath) { @(Get-Content -LiteralPath $manifestPath | ForEach-Object { $_ | ConvertFrom-Json } | Where-Object id -EQ $Id) } else { @() }
if ($prior.Count -gt 0) { $prior[-1] | ConvertTo-Json -Compress; exit 0 }
$inputPath = Join-Path $PSScriptRoot 'atomic-api-input.json'
$payload = @{ title=$spec.title; body=$spec.body; labels=@($spec.labels) }
$payload | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $inputPath -Encoding utf8
$endpoint = if ($spec.number) { "repos/BerryUIKI/Omera/issues/$($spec.number)" } else { 'repos/BerryUIKI/Omera/issues' }
$method = if ($spec.number) { 'PATCH' } else { 'POST' }
$json = & gh api --method $method $endpoint --input $inputPath
if ($LASTEXITCODE -ne 0) { throw "GitHub API failed: $Id" }
$issue = $json | ConvertFrom-Json
if ($issue.title -ne $spec.title -or $issue.body.Replace("`r`n","`n") -ne $spec.body.Replace("`r`n","`n")) { throw "GitHub content mismatch: $Id" }
$event = @{ id=$Id; number=$issue.number; url=$issue.html_url; title=$issue.title; state=$issue.state; action=$method; timestamp=[DateTime]::UtcNow.ToString('o') }
$event | ConvertTo-Json -Compress | Add-Content -LiteralPath $manifestPath -Encoding utf8
$event | ConvertTo-Json -Compress
