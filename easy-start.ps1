[CmdletBinding()]
param()

$ErrorActionPreference = "Stop"
$ProjectRoot = $PSScriptRoot

function Write-Color {
    param([string]$Text, [string]$Color = "White")
    Write-Host $Text -ForegroundColor $Color
}

function Read-Choice {
    param([string]$Prompt, [string[]]$Options, [int]$DefaultIndex = 0)

    Write-Host ""
    Write-Host $Prompt -ForegroundColor Cyan
    for ($i = 0; $i -lt $Options.Length; $i++) {
        $mark = if ($i -eq $DefaultIndex) { " [默认]" } else { "" }
        Write-Host "  [$($i+1)] $($Options[$i])$mark"
    }
    Write-Host "  [0] 退出" -ForegroundColor DarkGray

    $input = Read-Host "请选择 (0-$($Options.Length))"
    if ($input -eq "0") { exit 0 }
    if ($input -eq "") { return $DefaultIndex }
    $num = [int]::TryParse($input, [ref]$null)
    if ($num -and [int]$input -ge 1 -and [int]$input -le $Options.Length) {
        return [int]$input - 1
    }
    Write-Color "输入无效，使用默认值。" Yellow
    return $DefaultIndex
}

function Read-HexInput {
    param([string]$Prompt, [string]$Default, [string]$Name)

    while ($true) {
        $input = Read-Host "${Prompt} (默认: $Default)"
        if ($input -eq "") { return $Default }

        $normalized = $input.Trim()
        if ($normalized.StartsWith("0x", [System.StringComparison]::OrdinalIgnoreCase)) {
            $normalized = $normalized.Substring(2)
        }
        if ($normalized -notmatch '^[0-9a-fA-F]*$') {
            Write-Color "$Name 只能包含十六进制字符 (0-9, a-f)，也可以加 0x 前缀" Yellow
            continue
        }
        return $input.Trim()
    }
}

function Read-Number {
    param([string]$Prompt, [int]$Default, [int]$Min = 1, [int]$Max = 999)

    while ($true) {
        $input = Read-Host "${Prompt} (默认: $Default)"
        if ($input -eq "") { return $Default }
        $num = 0
        if ([int]::TryParse($input, [ref]$num) -and $num -ge $Min -and $num -le $Max) {
            return $num
        }
        Write-Color "请输入 $Min 到 $Max 之间的数字" Yellow
    }
}

function Read-YesNo {
    param([string]$Prompt, [bool]$Default = $true)

    $defaultStr = if ($Default) { "Y" } else { "N" }
    $input = Read-Host "${Prompt} (Y/N, 默认: $defaultStr)"
    if ($input -eq "") { return $Default }
    return $input -match '^[Yy]'
}

function Show-TimeEstimate {
    param([int]$DigitCount, [bool]$CaseSensitive, [int]$LetterCount)

    $est = [math]::Pow(16, $DigitCount)
    if ($CaseSensitive -and $LetterCount -gt 0) {
        $est = $est * [math]::Pow(2, $LetterCount)
    }

    $rate = 2000000
    $seconds = $est / $rate

    if ($seconds -lt 1) {
        $timeStr = "不到 1 秒"
    } elseif ($seconds -lt 60) {
        $timeStr = "约 $([math]::Round($seconds)) 秒"
    } elseif ($seconds -lt 3600) {
        $timeStr = "约 $([math]::Round($seconds/60)) 分钟"
    } elseif ($seconds -lt 86400) {
        $timeStr = "约 $([math]::Round($seconds/3600, 1)) 小时"
    } else {
        $timeStr = "约 $([math]::Round($seconds/86400, 1)) 天 —— 建议降低难度"
    }

    return @"
  难度指数: 16^${DigitCount} $(
        if ($CaseSensitive -and $LetterCount -gt 0) { "× 2^${LetterCount}" } else { "" }
    )
  平均尝试次数: $([math]::Round($est).ToString('N0'))
  预估时间(8线程): $timeStr
"@
}

Clear-Host
Write-Color "╔══════════════════════════════════════════╗" Cyan
Write-Color "║       Vanity Wallet 交互式生成器        ║" Cyan
Write-Color "║       EVM 靓号地址生成工具              ║" Cyan
Write-Color "╚══════════════════════════════════════════╝" Cyan
Write-Color ""
Write-Color "本工具会一步步引导你设置参数，无需记命令。" Green
Write-Color ""

# === 模式选择 ===
$mode = Read-Choice -Prompt "搜索模式：" -Options @(
    "只搜后缀 (比如地址以 888888 结尾)",
    "只搜前缀 (比如地址以 000000 开头)",
    "同时搜前缀和后缀"
)

$prefix = ""
$suffix = "00000000"

if ($mode -eq 0) {
    $suffix = Read-HexInput -Prompt "想搜什么后缀" -Default "00000000" -Name "后缀"
} elseif ($mode -eq 1) {
    $suffix = ""
    $prefix = Read-HexInput -Prompt "想搜什么前缀" -Default "000000" -Name "前缀"
} else {
    $prefix = Read-HexInput -Prompt "前缀" -Default "" -Name "前缀"
    $suffix = Read-HexInput -Prompt "后缀" -Default "00000000" -Name "后缀"
}

# === 大小写敏感 ===
$totalDigits = (($prefix -replace '^0x', '').Length) + (($suffix -replace '^0x', '').Length)
$hasLetter = ($prefix -match '[a-fA-F]') -or ($suffix -match '[a-fA-F]')

$caseSensitive = $false
if ($hasLetter) {
    Write-Color ""
    Write-Color "提示：你的搜索内容包含字母 (a-f)，启用大小写敏感可以按 EIP-55 精确匹配，" Yellow
    Write-Color "但难度会翻倍（每个字母位 ×2）。通常不需要开启。" Yellow
    $caseSensitive = Read-YesNo -Prompt "启用大小写敏感匹配？" -Default $false
}

# === 线程数 ===
$defaultCores = [Math]::Max(1, [Environment]::ProcessorCount - 1)
$workers = Read-Number -Prompt "使用几个 CPU 线程" -Default $defaultCores -Min 1 -Max 64

# === 防止睡眠 ===
$preventSleep = Read-YesNo -Prompt "运行时防止电脑睡眠？" -Default $true

# === 快速测试 ===
Write-Color ""
Write-Color "快速测试模式：只跑 20 秒，不保存结果，用来测速度。" Yellow
$quickTest = Read-YesNo -Prompt "先测个速再决定？" -Default $true

$letterCount = 0
if ($caseSensitive) {
    $letterCount = @(($prefix + $suffix).ToCharArray() | Where-Object { $_ -match '[a-fA-F]' }).Count
}

Write-Color ""
Write-Color "╔══════════ 配置摘要 ══════════╗" Cyan
Write-Color "  搜索模式:  $(
    if ($mode -eq 0) { "后缀" }
    elseif ($mode -eq 1) { "前缀" }
    else { "前缀 + 后缀" }
)"
if ($prefix -ne "") { Write-Color "  前缀:      $prefix" }
if ($suffix -ne "") { Write-Color "  后缀:      $suffix" }
Write-Color "  线程数:    $workers"
Write-Color "  防止睡眠:  $(if ($preventSleep) { '是' } else { '否' })"
Write-Color "  大小写:    $(if ($caseSensitive) { '敏感' } else { '不敏感' })"
if ($quickTest) {
    Write-Color "  模式:      先测速 20 秒" Yellow
} else {
    Write-Color "  模式:      直接开始搜索" Green
}
Write-Color ""

if ($totalDigits -gt 0 -and $totalDigits -le 20) {
    Write-Color (Show-TimeEstimate -DigitCount $totalDigits -CaseSensitive $caseSensitive -LetterCount $letterCount) Yellow
}
Write-Color "╚══════════════════════════════╝" Cyan

Write-Color ""
if (-not (Read-YesNo -Prompt "确认无误，开始？" -Default $true)) {
    Write-Color "已取消。" Red
    exit 0
}

$scriptArgs = @()

if ($prefix -ne "") { $scriptArgs += "-Prefix"; $scriptArgs += $prefix }
if ($suffix -ne "") { $scriptArgs += "-Suffix"; $scriptArgs += $suffix }
$scriptArgs += "-Workers"; $scriptArgs += $workers
$scriptArgs += "-StatusIntervalSeconds"; $scriptArgs += 2

if ($caseSensitive) { $scriptArgs += "-CaseSensitive" }
if ($preventSleep) { $scriptArgs += "-PreventSleep" }

if ($quickTest) {
    Write-Color ""
    Write-Color "→ 先跑 20 秒测速，感受一下速度。" Green
    Write-Color "  测速完成后会询问你是否正式开跑。" Green
    Write-Color ""
    & (Join-Path $ProjectRoot "Measure-NativeSpeed.ps1") -Workers $workers -Seconds 20

    Write-Color ""
    Write-Color "测速完成！以上是实时速度。" Green
    if (Read-YesNo -Prompt "现在正式开跑？" -Default $true) {
        & (Join-Path $ProjectRoot "start-native.ps1") @scriptArgs
    } else {
        Write-Color "已跳过正式搜索。需要时可以重新运行本脚本。" Green
    }
} else {
    & (Join-Path $ProjectRoot "start-native.ps1") @scriptArgs
}
