param([int]$MaxCount = 5)
$ErrorActionPreference = 'Stop'
$reviewRoot = 'D:\dev\Omera\docs\reviews'
$reviewPlan = Get-Content -LiteralPath (Join-Path $reviewRoot 'github-issue-plan-2026-10-05.json') -Raw | ConvertFrom-Json
$reviewManifest = Join-Path $reviewRoot 'github-published-issues-2026-10-05.jsonl'
$reviewBodyDir = Join-Path $reviewRoot 'github-issue-bodies'
[System.IO.Directory]::CreateDirectory($reviewBodyDir) | Out-Null
$reviewEvents = @()
if (Test-Path -LiteralPath $reviewManifest) {
    $reviewEvents = @(Get-Content -LiteralPath $reviewManifest | Where-Object { $_ } | ForEach-Object { $_ | ConvertFrom-Json })
}
$reviewUtf8 = New-Object System.Text.UTF8Encoding($false)
$reviewDone = @($reviewEvents | Where-Object { $_.event -eq 'published' } | ForEach-Object { $_.id })
$reviewSelected = @($reviewPlan | Where-Object { $_.id -notin $reviewDone } | Select-Object -First $MaxCount)
foreach ($reviewSpec in $reviewSelected) {
    $reviewBodyFile = Join-Path $reviewBodyDir ($reviewSpec.id + '.md')
    [System.IO.File]::WriteAllText($reviewBodyFile, $reviewSpec.body, $reviewUtf8)
    $reviewLabels = $reviewSpec.labels -join ','
    if ($reviewSpec.reopen) {
        $reviewNumber = [int]$reviewSpec.reopen
        if (-not ($reviewEvents | Where-Object { $_.id -eq $reviewSpec.id -and $_.event -eq 'commented' })) {
            $reviewCommentUrl = & gh issue comment $reviewNumber --repo BerryUIKI/Omera --body-file $reviewBodyFile
            if ($LASTEXITCODE -ne 0) { throw "Comment failed for $($reviewSpec.id); inspect remote state before retry." }
            $reviewCommentEvent = @{ event = 'commented'; id = $reviewSpec.id; number = $reviewNumber; url = [string]::Join([Environment]::NewLine, $reviewCommentUrl).Trim() } | ConvertTo-Json -Compress
            [System.IO.File]::AppendAllText($reviewManifest, $reviewCommentEvent + [Environment]::NewLine, $reviewUtf8)
        }
        & gh issue edit $reviewNumber --repo BerryUIKI/Omera --add-label $reviewLabels | Out-Null
        if ($LASTEXITCODE -ne 0) { throw "Label update failed for $($reviewSpec.id)." }
        & gh issue reopen $reviewNumber --repo BerryUIKI/Omera | Out-Null
        if ($LASTEXITCODE -ne 0) { throw "Reopening failed for $($reviewSpec.id)." }
        $reviewUrl = "https://github.com/BerryUIKI/Omera/issues/$reviewNumber"
        $reviewAction = 'reopened'
    } else {
        $reviewCreateOutput = & gh issue create --repo BerryUIKI/Omera --title $reviewSpec.title --body-file $reviewBodyFile --label $reviewLabels
        if ($LASTEXITCODE -ne 0) { throw "Issue creation failed for $($reviewSpec.id); inspect remote state before retry." }
        $reviewUrl = [string]::Join([Environment]::NewLine, $reviewCreateOutput).Trim()
        if ($reviewUrl -notmatch '^https://github\.com/BerryUIKI/Omera/issues/(\d+)$') { throw "Unexpected creation response: $reviewUrl" }
        $reviewNumber = [int]$Matches[1]
        $reviewAction = 'created'
    }
    $reviewPublishedEvent = @{
        event = 'published'; id = $reviewSpec.id; number = $reviewNumber; url = $reviewUrl;
        action = $reviewAction; title = $reviewSpec.title; priority = $reviewSpec.priority
    } | ConvertTo-Json -Compress
    [System.IO.File]::AppendAllText($reviewManifest, $reviewPublishedEvent + [Environment]::NewLine, $reviewUtf8)
    Write-Output ("Published {0} ({1}): {2}" -f $reviewSpec.id, $reviewAction, $reviewUrl)
}

