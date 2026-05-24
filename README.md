<p align="center">
  <img src="https://img.shields.io/badge/Windows-x64-blue?logo=windows" alt="Windows">
  <img src="https://img.shields.io/badge/Rust-1.70+-orange?logo=rust" alt="Rust">
  <img src="https://img.shields.io/github/v/release/lin200083/vanity-wallet-generator?logo=github" alt="Release">
  <img src="https://img.shields.io/badge/license-MIT-green" alt="License">
</p>

<h1 align="center">⚡ Vanity Wallet Generator</h1>
<p align="center">Windows 上的 EVM 靓号钱包地址生成器 — 暴力搜索你想要的 <code>0x...</code> 前缀或后缀</p>

<br>

> **⚠️ 安全提醒**  
> 私钥 = 资产控制权。命中后**立刻备份 `PrivateKey`**，不要泄露给任何人。  
> 转入大额资产前，务必先小额测试。

<br>

## 🚀 快速开始

选择你的入口：

| 如果你是... | 怎么做 |
|-------------|--------|
| 🧑 **新手** | 双击 `双击我运行.bat`，菜单引导，无需记命令 |
| 🧑‍💻 **高手** | 用 `start-native.ps1` 传参运行 |
| 📦 **直接下载** | 去 [Releases](https://github.com/lin200083/vanity-wallet-generator/releases/latest) 下载 zip，解压即用 |

**脚本不能运行？** 先放行执行策略：

```powershell
Set-ExecutionPolicy -Scope Process -ExecutionPolicy Bypass
```

**命令行示例：**

```powershell
# 搜索 8 个 0 后缀（默认）
.\start-native.ps1 -Suffix "00000000" -Workers 8 -PreventSleep

# 搜索前缀
.\start-native.ps1 -Prefix "dead" -Workers 8

# 同时要求前缀和后缀
.\start-native.ps1 -Prefix "0000" -Suffix "beef" -Workers 8
```

> 第一次运行会自动编译 Rust 引擎，稍等片刻即可。

<br>

## 📋 常用操作

| 操作 | 命令 |
|------|------|
| 🔬 测速 | `.\Measure-NativeSpeed.ps1 -Workers 8 -Seconds 20` |
| 📊 查看运行状态 | `.\Get-Status.ps1` |
| 🧹 清理构建缓存 | `.\Clean-Generated.ps1` |
| 🗑️ 清理缓存 + 钱包结果 | `.\Clean-Generated.ps1 -IncludeWalletResults` |
| 📖 查看全部参数 | `.\start-native.ps1 --help` |

<br>

## ⚙️ 工作原理

本地暴力穷举：随机私钥 → 推导公钥 → 计算地址 → 匹配规则 → 命中保存，否则继续。

优化：公钥点加递增，避免每次都重新做完整推导。

**难度参考**（平均值，非保证值）：

```
后缀 0000       ~6.5 万次尝试
后缀 000000     ~1677 万次尝试
后缀 00000000   ~42.9 亿次尝试  → 约 36 分钟 @ 200万/秒
后缀 000000000  ~687 亿次尝试   → 约 9.5 小时
```

## 🔗 适用链

✅ Ethereum、BSC、Polygon、Arbitrum、Optimism、Base 等所有 EVM 链  
❌ Bitcoin、Solana、Tron 等非 EVM 链

<br>

## ❓ 常见问题

| 问题 | 解决办法 |
|------|----------|
| 💻 电脑变卡？ | 减少 worker：`-Workers 4` |
| ⏳ 跑很久没出？ | 先测速，再按 rate 估算时间，这很正常 |
| 🔑 命中了没有私钥？ | 检查是否误加了 `-RedactPrivateKey` |
| 🦀 找不到 Rust/Cargo？ | 已预编译 `bin\vanity-native.exe`，直接运行 |
| 📂 怎么看结果？ | 打开 `results\matched-wallet-latest.txt`，**备份 PrivateKey** |
