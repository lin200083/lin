[CmdletBinding()]
param(
    [string]$Version = "1.1.0"
)

$ErrorActionPreference = "Stop"

$ProjectRoot = (Resolve-Path $PSScriptRoot).Path
$DistDir = Join-Path $ProjectRoot "dist"
$StagingDir = Join-Path $DistDir "staging"
$BeginnerName = "vanity-wallet-generator-windows-x64-v$Version"
$SourceName = "vanity-wallet-generator-source-v$Version"
$BeginnerRoot = Join-Path $StagingDir $BeginnerName
$SourceRoot = Join-Path $StagingDir $SourceName
$NativeExe = Join-Path $ProjectRoot "bin\vanity-native.exe"
$StandaloneExe = Join-Path $DistDir "vanity-native-windows-x64-v$Version.exe"
$ReleaseNotes = Join-Path $DistDir "release-notes-v$Version.md"

function Assert-ProjectPath {
    param([string]$Path)

    $fullPath = [System.IO.Path]::GetFullPath($Path)
    if (-not $fullPath.StartsWith($ProjectRoot, [System.StringComparison]::OrdinalIgnoreCase)) {
        throw "Refusing to operate outside project: $fullPath"
    }
    return $fullPath
}

function Reset-Directory {
    param([string]$Path)

    $fullPath = Assert-ProjectPath $Path
    if (Test-Path -LiteralPath $fullPath) {
        Remove-Item -LiteralPath $fullPath -Recurse -Force
    }
    New-Item -ItemType Directory -Force -Path $fullPath | Out-Null
}

function Copy-ProjectItem {
    param(
        [string]$RelativePath,
        [string]$DestinationRoot
    )

    $source = Join-Path $ProjectRoot $RelativePath
    if (-not (Test-Path -LiteralPath $source)) {
        throw "Required release item not found: $RelativePath"
    }

    $destination = Join-Path $DestinationRoot $RelativePath
    $destinationParent = Split-Path -Parent $destination
    New-Item -ItemType Directory -Force -Path $destinationParent | Out-Null
    Copy-Item -LiteralPath $source -Destination $destination -Recurse -Force
}

if (-not (Test-Path -LiteralPath $NativeExe)) {
    & (Join-Path $ProjectRoot "Build-Native.ps1")
    if ($LASTEXITCODE -ne 0) {
        throw "Build-Native.ps1 failed with exit code $LASTEXITCODE."
    }
}

Reset-Directory $DistDir
New-Item -ItemType Directory -Force -Path $BeginnerRoot, $SourceRoot | Out-Null

$beginnerItems = @(
    "README.md",
    "LICENSE",
    "start-native.ps1",
    "Get-Status.ps1",
    "Measure-NativeSpeed.ps1",
    "Clean-Generated.ps1",
    "bin\vanity-native.exe"
)

foreach ($item in $beginnerItems) {
    Copy-ProjectItem $item $BeginnerRoot
}

$sourceItems = @(
    ".gitignore",
    "README.md",
    "LICENSE",
    "Build-Native.ps1",
    "Clean-Generated.ps1",
    "Get-Status.ps1",
    "Measure-NativeSpeed.ps1",
    "Release-Pack.ps1",
    "start-native.ps1",
    "Upload-Release.ps1",
    "native\vanity-native\Cargo.lock",
    "native\vanity-native\Cargo.toml",
    "native\vanity-native\src"
)

foreach ($item in $sourceItems) {
    Copy-ProjectItem $item $SourceRoot
}

$beginnerZip = Join-Path $DistDir "$BeginnerName.zip"
$sourceZip = Join-Path $DistDir "$SourceName.zip"

Compress-Archive -LiteralPath $BeginnerRoot -DestinationPath $beginnerZip -Force
Compress-Archive -LiteralPath $SourceRoot -DestinationPath $sourceZip -Force
Copy-Item -LiteralPath $NativeExe -Destination $StandaloneExe -Force
Set-Content -LiteralPath $ReleaseNotes -Encoding UTF8 -Value @"
## v$Version

This release focuses on performance and packaging.

### Changes

- Reworked the native search loop to use a random starting private key plus sequential public-key point addition.
- Improved observed throughput on the test machine from about 300k addresses/sec to about 2M addresses/sec with 12 workers.
- Added packaging support for separate source and beginner Windows downloads.
- Added generated-file cleanup helpers and ignore rules.
- Updated README performance estimates and release file names.

### Downloads

- `vanity-wallet-generator-windows-x64-v$Version.zip`: recommended for beginners. Unzip and run `start-native.ps1`.
- `vanity-wallet-generator-source-v$Version.zip`: source package for developers.
- `vanity-native-windows-x64-v$Version.exe`: standalone native executable.

### Safety

This tool generates private keys locally. Keep result files private and back up any private key before funding an address.
"@

Remove-Item -LiteralPath $StagingDir -Recurse -Force

Write-Host "Created:"
Write-Host $beginnerZip
Write-Host $sourceZip
Write-Host $StandaloneExe
Write-Host $ReleaseNotes
