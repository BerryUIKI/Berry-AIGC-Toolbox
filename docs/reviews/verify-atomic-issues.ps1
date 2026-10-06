$ErrorActionPreference = 'Stop'
$plan = Get-Content (Join-Path $PSScriptRoot 'github-atomic-plan-2026-10-05.json') -Raw | ConvertFrom-Json
$published = @(Get-Content (Join-Path $PSScriptRoot 'github-atomic-publication-2026-10-05.jsonl') | ForEach-Object { $_ | ConvertFrom-Json })
$remote = Get-Content (Join-Path $PSScriptRoot 'github-atomic-verification-2026-10-05.json') -Raw | ConvertFrom-Json
$failures = @()
foreach ($spec in $plan) {
    $record = @($published | Where-Object id -EQ $spec.id)[-1]
    if (-not $record) { $failures += "Not published: $($spec.id)"; continue }
    $issue = $remote | Where-Object number -EQ $record.number
    if (-not $issue) { $failures += "Remote missing: $($spec.id)"; continue }
    if ($issue.title -cne $spec.title) { $failures += "Title mismatch: $($spec.id)" }
    if ($issue.body.Replace("`r`n","`n") -cne $spec.body.Replace("`r`n","`n")) { $failures += "Body mismatch: $($spec.id)" }
    if ($issue.state -ne 'OPEN') { $failures += "Not open: $($spec.id)" }
    $expectedLabels = @($spec.labels | Sort-Object) -join ','
    $actualLabels = @($issue.labels.name | Sort-Object) -join ','
    if ($expectedLabels -cne $actualLabels) { $failures += "Labels mismatch: $($spec.id)" }
}
$summary = @{ expected=$plan.Count; published=$published.Count; verified=($plan.Count-$failures.Count); failures=@($failures) }
$summary | ConvertTo-Json -Depth 5 | Set-Content (Join-Path $PSScriptRoot 'github-atomic-verification-summary-2026-10-05.json') -Encoding utf8
$summary | ConvertTo-Json -Compress
if ($failures.Count -gt 0) { exit 1 }
