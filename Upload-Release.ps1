[CmdletBinding()]
param(
    [string]$Version = "1.1.0",
    [string]$Repository = "lin200083/vanity-wallet-generator"
)

$ErrorActionPreference = "Stop"

$ProjectRoot = (Resolve-Path $PSScriptRoot).Path
$DistDir = Join-Path $ProjectRoot "dist"
$Tag = "v$Version"
$NotesPath = Join-Path $DistDir "release-notes-$Tag.md"
$Assets = @(
    Join-Path $DistDir "vanity-wallet-generator-windows-x64-$Tag.zip"
    Join-Path $DistDir "vanity-wallet-generator-source-$Tag.zip"
    Join-Path $DistDir "vanity-native-windows-x64-$Tag.exe"
)

$gh = (Get-Command gh -ErrorAction SilentlyContinue).Source
if (-not $gh) {
    $gh = "C:\Program Files\GitHub CLI\gh.exe"
}

if (-not (Test-Path -LiteralPath $gh)) {
    throw "GitHub CLI was not found. Install it with: winget install --id GitHub.cli --exact"
}

& $gh auth status
if ($LASTEXITCODE -ne 0) {
    throw "GitHub CLI is not logged in. Run: gh auth login"
}

foreach ($asset in $Assets) {
    if (-not (Test-Path -LiteralPath $asset)) {
        throw "Release asset not found: $asset"
    }
}

if (-not (Test-Path -LiteralPath $NotesPath)) {
    throw "Release notes not found: $NotesPath"
}

& $gh release view $Tag --repo $Repository *> $null
if ($LASTEXITCODE -eq 0) {
    Write-Host "Release $Tag exists. Uploading assets with --clobber..."
    & $gh release upload $Tag @Assets --repo $Repository --clobber
} else {
    Write-Host "Creating release $Tag..."
    & $gh release create $Tag @Assets --repo $Repository --title $Tag --notes-file $NotesPath
}

if ($LASTEXITCODE -ne 0) {
    throw "GitHub release upload failed with exit code $LASTEXITCODE."
}

Write-Host "Release ready:"
Write-Host "https://github.com/$Repository/releases/tag/$Tag"
