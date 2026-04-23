[CmdletBinding()]
param(
    [switch]$IncludeWalletResults
)

$ErrorActionPreference = "Stop"

$ProjectRoot = (Resolve-Path $PSScriptRoot).Path

function Remove-ProjectPath {
    param([string]$Path)

    if (-not (Test-Path -LiteralPath $Path)) {
        return
    }

    $resolved = (Resolve-Path -LiteralPath $Path).Path
    if (-not $resolved.StartsWith($ProjectRoot, [System.StringComparison]::OrdinalIgnoreCase)) {
        throw "Refusing to remove outside project: $resolved"
    }

    Remove-Item -LiteralPath $resolved -Recurse -Force
    Write-Host "Removed $resolved"
}

Remove-ProjectPath (Join-Path $ProjectRoot "native\vanity-native\target")
Get-ChildItem -LiteralPath (Join-Path $ProjectRoot "native\vanity-native") -Directory -Filter "target-*" -ErrorAction SilentlyContinue |
    ForEach-Object { Remove-ProjectPath $_.FullName }

Get-ChildItem -LiteralPath (Join-Path $ProjectRoot "state") -Directory -ErrorAction SilentlyContinue |
    Where-Object { $_.Name -like "bench-*" -or $_.Name -like "bench2-*" -or $_.Name -eq "native-speed-test" } |
    ForEach-Object { Remove-ProjectPath $_.FullName }

Get-ChildItem -LiteralPath (Join-Path $ProjectRoot "results") -Directory -ErrorAction SilentlyContinue |
    Where-Object { $_.Name -like "bench-*" -or $_.Name -like "bench2-*" -or $_.Name -eq "native-speed-test" -or $_.Name -eq "speed-test" } |
    ForEach-Object { Remove-ProjectPath $_.FullName }

if ($IncludeWalletResults) {
    Write-Warning "Removing wallet result files because -IncludeWalletResults was specified."
    Remove-ProjectPath (Join-Path $ProjectRoot "results")
}

Write-Host "Generated files cleaned."
