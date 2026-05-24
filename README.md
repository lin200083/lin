[English](README_EN.md) | [中文](README.md)

# Vanity Wallet Generator

Windows 上的 **EVM 靓号钱包地址生成器**，暴力搜索符合自定义前缀/后缀的 `0x...` 地址。

> **⚠️ 安全警告：私钥就是资产控制权。命中后立刻备份 `PrivateKey`，不要发给任何人。正式使用前建议小额测试。**

## 快速开始

### 新手（交互模式）

双击 `双击我运行.bat`，按菜单提示操作即可。全程不需要记任何命令。

脚本不能运行？先放行执行策略：

```powershell
Set-ExecutionPolicy -Scope Process -ExecutionPolicy Bypass
```

### 高手（命令行模式）

```powershell
# 搜索 8 个 0 后缀（默认）
.\start-native.ps1 -Suffix "00000000" -Workers 8 -PreventSleep

# 搜索前缀
.\start-native.ps1 -Prefix "dead" -Workers 8

# 同时要求前缀和后缀
.\start-native.ps1 -Prefix "0000" -Suffix "beef" -Workers 8
```

> 第一次运行会自动编译 Rust 引擎，稍等片刻即可。

### 从 Releases 下载

去 [GitHub Releases](https://github.com/lin200083/vanity-wallet-generator/releases/latest) 下载 `vanity-wallet-generator-windows-x64-vX.Y.Z.zip`，解压后双击 `双击我运行.bat`。

---

## 常用操作

| 操作 | 命令 |
|------|------|
| 测速 | `.\Measure-NativeSpeed.ps1 -Workers 8 -Seconds 20` |
| 查看运行时状态 | `.\Get-Status.ps1` |
| 清理构建缓存 | `.\Clean-Generated.ps1` |
| 清理缓存 + 钱包结果 | `.\Clean-Generated.ps1 -IncludeWalletResults` |
| 查看全部参数 | `.\start-native.ps1 --help` |

---

## 工作原理

本地暴力搜索：随机生成私钥 → secp256k1 推导公钥 → Keccak-256 计算地址 → 匹配前缀/后缀 → 命中则保存，否则继续。

优化：通过公钥点加连续递增，避免每次重新做完整私钥推导。

难度（平均值，非保证值）：

```text
后缀 0000       ~6.5 万次尝试
后缀 000000     ~1677 万次尝试
后缀 00000000   ~42.9 亿次尝试
后缀 000000000  ~687 亿次尝试
```

按 200 万地址/秒估算：`00000000` 约 36 分钟，`000000000` 约 9.5 小时。

## 适用链

Ethereum、BSC、Polygon、Arbitrum、Optimism、Base 等所有 EVM `0x...` 格式的链。

**不适用**：Bitcoin、Solana、Tron 等非 EVM 链。

---

## 常见问题

**电脑变卡？** 减少 worker：`-Workers 4`

**跑很久没出？** 正常。先测速确认 `rate`，再根据难度估算时间。

**命中了但没有私钥？** 检查是否误加了 `-RedactPrivateKey`

**找不到 Rust/Cargo？** 这台机器已预编译 `bin\vanity-native.exe`，直接运行即可。

**命中后怎么看结果？** 打开 `results\matched-wallet-latest.txt`，里面有 `Address`（收款地址）和 `PrivateKey`（私钥）。**备份私钥，不要泄露。**
