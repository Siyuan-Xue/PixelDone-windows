[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)][string]$Owner,
    [Parameter(Mandatory = $true)][string]$Repository,
    [Parameter(Mandatory = $true)][string]$Tag,
    [Parameter(Mandatory = $true)][string]$ExpectedCommit,
    [Parameter(Mandatory = $true)][string]$Title,
    [Parameter(Mandatory = $true)][string]$NotesPath,
    [Parameter(Mandatory = $true)][string[]]$AssetPaths,
    [int]$TagPollAttempts = 20,
    [int]$TagPollSeconds = 30
)

$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"
Set-StrictMode -Version Latest
$ApiBase = "https://gitee.com/api/v5/repos/$Owner/$Repository"
$Headers = @{ Authorization = "token $($env:GITEE_ACCESS_TOKEN)"; Accept = "application/json" }

function Normalize-Text([string]$Value) { return ($Value -replace "`r`n", "`n").TrimEnd() }
function Invoke-GiteeJson([string]$Uri, [string]$Method = "Get", [hashtable]$Body = $null) {
    $parameters = @{ Uri = $Uri; Method = $Method; Headers = $Headers }
    if ($null -ne $Body) { $parameters.Body = $Body }
    return Invoke-RestMethod @parameters
}
function Find-GiteeTag {
    $tags = Invoke-GiteeJson "$ApiBase/tags?per_page=100&page=1"
    return @($tags) | Where-Object { $_.name -eq $Tag } | Select-Object -First 1
}
function Get-GiteeRelease {
    try { return Invoke-GiteeJson "$ApiBase/releases/tags/$Tag" }
    catch { if ($_.Exception.Response.StatusCode.value__ -eq 404) { return $null }; throw }
}
function Get-Attachments([long]$ReleaseId) { return @(Invoke-GiteeJson "$ApiBase/releases/$ReleaseId/attach_files") }
function Upload-Attachment([string]$Path, [long]$ReleaseId) {
    $response = Join-Path $env:RUNNER_TEMP ("gitee-upload-" + [guid]::NewGuid() + ".json")
    try {
        $status = curl --fail-with-body --show-error --location --http1.1 --connect-timeout 30 --max-time 2700 `
            --form-string "access_token=$($env:GITEE_ACCESS_TOKEN)" --form-string "owner=$Owner" `
            --form-string "repo=$Repository" --form-string "release_id=$ReleaseId" --form "file=@$Path" `
            --output $response --write-out "%{http_code}" "$ApiBase/releases/$ReleaseId/attach_files"
        if ($LASTEXITCODE -ne 0 -or $status -notin @("200", "201")) { throw "Gitee upload failed for $Path (HTTP $status)." }
    } finally { Remove-Item $response -Force -ErrorAction SilentlyContinue }
}
function Assert-Attachment([pscustomobject]$Attachment, [string]$ExpectedPath, [long]$ReleaseId) {
    $download = Join-Path $env:RUNNER_TEMP ("gitee-" + [guid]::NewGuid() + "-" + $Attachment.name)
    try {
        Invoke-WebRequest -Uri "$ApiBase/releases/$ReleaseId/attach_files/$($Attachment.id)/download" -Headers $Headers -OutFile $download -TimeoutSec 2700
        if ((Get-FileHash -Algorithm SHA256 $download).Hash -ne (Get-FileHash -Algorithm SHA256 $ExpectedPath).Hash) {
            throw "Gitee attachment $($Attachment.name) conflicts with the CI artifact."
        }
    } finally { Remove-Item $download -Force -ErrorAction SilentlyContinue }
}

foreach ($path in @($NotesPath) + $AssetPaths) {
    if (-not (Test-Path -LiteralPath $path -PathType Leaf)) { throw "Missing release input: $path" }
}
if ([string]::IsNullOrWhiteSpace($env:GITEE_ACCESS_TOKEN)) { throw "GITEE_ACCESS_TOKEN is required." }

$giteeTag = $null
for ($attempt = 1; $attempt -le $TagPollAttempts; $attempt++) {
    $giteeTag = Find-GiteeTag
    if ($null -ne $giteeTag) { break }
    if ($attempt -lt $TagPollAttempts) { Start-Sleep -Seconds $TagPollSeconds }
}
if ($null -eq $giteeTag) { throw "Gitee mirror did not expose tag $Tag." }
if ([string]$giteeTag.commit.sha -ne $ExpectedCommit) { throw "Gitee tag commit mismatch." }

$notes = Normalize-Text (Get-Content -Raw $NotesPath)
$release = Get-GiteeRelease
if ($null -eq $release) {
    $release = Invoke-GiteeJson "$ApiBase/releases" "Post" @{
        tag_name = $Tag; target_commitish = $ExpectedCommit; name = $Title; body = $notes; prerelease = "false"
    }
}
if ([string]$release.tag_name -ne $Tag -or [string]$release.name -ne $Title) { throw "Gitee Release metadata mismatch." }
if (([string]$release.prerelease).ToLowerInvariant() -eq "true") { throw "Gitee Release is a prerelease." }
if ((Normalize-Text ([string]$release.body)) -ne $notes) { throw "Gitee Release notes conflict." }

$releaseId = [long]$release.id
$attachments = Get-Attachments $releaseId
foreach ($path in $AssetPaths) {
    $name = Split-Path -Leaf $path
    $matches = @($attachments | Where-Object { $_.name -eq $name })
    if ($matches.Count -gt 1) { throw "Duplicate Gitee attachment $name." }
    if ($matches.Count -eq 0) {
        Upload-Attachment $path $releaseId
        $attachments = Get-Attachments $releaseId
        $matches = @($attachments | Where-Object { $_.name -eq $name })
    }
    if ($matches.Count -ne 1) { throw "Gitee did not expose exactly one $name attachment." }
    Assert-Attachment $matches[0] $path $releaseId
}
Write-Output "Verified Gitee Release $Tag with the mirrored commit and identical assets."
