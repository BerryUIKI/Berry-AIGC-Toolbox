param([ValidateSet('create','update')][string]$Mode='create')
$ErrorActionPreference = 'Stop'
$plan = Get-Content (Join-Path $PSScriptRoot 'github-atomic-plan-2026-10-05.json') -Raw | ConvertFrom-Json
$items = if ($Mode -eq 'create') { @($plan | Where-Object { -not $_.number }) } else { @($plan | Where-Object { $_.number }) }
foreach ($item in $items) {
    & (Join-Path $PSScriptRoot 'publish-atomic-issue.ps1') -Id $item.id
    if ($LASTEXITCODE -ne 0) { throw "Publication failed: $($item.id)" }
}
