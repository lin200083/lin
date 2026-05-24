<p align="center">
  <img src="https://img.shields.io/badge/Windows-x64-blue?logo=windows" alt="Windows">
  <img src="https://img.shields.io/badge/Rust-1.70+-orange?logo=rust" alt="Rust">
  <img src="https://img.shields.io/github/v/release/lin200083/vanity-wallet-generator?logo=github" alt="Release">
  <img src="https://img.shields.io/badge/license-MIT-green" alt="License">
</p>

<h1 align="center">⚡ Vanity Wallet Generator</h1>
<p align="center">EVM vanity address generator for Windows — brute-force your desired <code>0x...</code> prefix or suffix</p>

<br>

> **⚠️ Security**  
> Your private key IS your asset control. **Back up `PrivateKey` immediately** after a match. Never share it.  
> Test with a small amount before transferring significant funds.

<br>

## 🚀 Quick Start

Pick your path:

| If you are... | Do this |
|---------------|---------|
| 🧑 **Beginner** | Double-click `双击我运行.bat` — menu-guided, no commands needed |
| 🧑‍💻 **Power user** | Use `start-native.ps1` with CLI parameters |
| 📦 **Download** | Grab the zip from [Releases](https://github.com/lin200083/vanity-wallet-generator/releases/latest), unzip and run |

**Execution policy error?** Bypass it:

```powershell
Set-ExecutionPolicy -Scope Process -ExecutionPolicy Bypass
```

**CLI examples:**

```powershell
# Search 8-zero suffix (default)
.\start-native.ps1 -Suffix "00000000" -Workers 8 -PreventSleep

# Search by prefix
.\start-native.ps1 -Prefix "dead" -Workers 8

# Both prefix and suffix
.\start-native.ps1 -Prefix "0000" -Suffix "beef" -Workers 8
```

> The Rust engine auto-compiles on first run — just wait a moment.

<br>

## 📋 Common Operations

| Action | Command |
|--------|---------|
| 🔬 Benchmark | `.\Measure-NativeSpeed.ps1 -Workers 8 -Seconds 20` |
| 📊 Check status | `.\Get-Status.ps1` |
| 🧹 Clean build cache | `.\Clean-Generated.ps1` |
| 🗑️ Clean cache + wallets | `.\Clean-Generated.ps1 -IncludeWalletResults` |
| 📖 Full parameter list | `.\start-native.ps1 --help` |

<br>

## ⚙️ How It Works

Local brute-force: random private key → derive public key → compute address → match rule → save or repeat.

Optimization: advances the public key via point addition, avoiding full derivation each time.

**Difficulty** (averages, not guarantees):

```
Suffix 0000       ~65,536 attempts
Suffix 000000     ~16.8 million attempts
Suffix 00000000   ~4.3 billion attempts  → ≈ 36 min @ 2M/s
Suffix 000000000  ~68.7 billion attempts → ≈ 9.5 hours
```

## 🔗 Supported Chains

✅ Ethereum, BSC, Polygon, Arbitrum, Optimism, Base — any EVM chain  
❌ Bitcoin, Solana, Tron — non-EVM chains not supported

<br>

## ❓ FAQ

| Problem | Solution |
|---------|----------|
| 💻 Computer slow? | Reduce workers: `-Workers 4` |
| ⏳ Taking too long? | Normal. Benchmark first, estimate from your `rate` |
| 🔑 Match found but no key? | Check for `-RedactPrivateKey` (testing only) |
| 🦀 Rust/Cargo not found? | Prebuilt `bin\vanity-native.exe` included |
| 📂 Where are results? | `results\matched-wallet-latest.txt` — **back up PrivateKey** |
