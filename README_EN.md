[English](README_EN.md) | [中文](README.md)

# Vanity Wallet Generator

An **EVM vanity address generator** for Windows. Brute-forces `0x...` addresses matching your custom prefix or suffix.

> **⚠️ Security: Your private key IS your asset control. Back up `PrivateKey` immediately after a match. Never share it. Test with a small amount before transferring significant funds.**

## Quick Start

### Beginner (Interactive)

Double-click `双击我运行.bat` and follow the menu. No commands needed.

Execution policy error? Run this once:

```powershell
Set-ExecutionPolicy -Scope Process -ExecutionPolicy Bypass
```

### Power User (CLI)

```powershell
# Search 8-zero suffix (default)
.\start-native.ps1 -Suffix "00000000" -Workers 8 -PreventSleep

# Search by prefix
.\start-native.ps1 -Prefix "dead" -Workers 8

# Both prefix and suffix
.\start-native.ps1 -Prefix "0000" -Suffix "beef" -Workers 8
```

> The Rust engine auto-compiles on first run — just wait a moment.

### Download Pre-built

Get the beginner pack from [GitHub Releases](https://github.com/lin200083/vanity-wallet-generator/releases/latest) (`vanity-wallet-generator-windows-x64-vX.Y.Z.zip`), unzip, and double-click `双击我运行.bat`.

---

## Common Operations

| Action | Command |
|--------|---------|
| Benchmark speed | `.\Measure-NativeSpeed.ps1 -Workers 8 -Seconds 20` |
| Check run status | `.\Get-Status.ps1` |
| Clean build cache | `.\Clean-Generated.ps1` |
| Clean cache + results | `.\Clean-Generated.ps1 -IncludeWalletResults` |
| Full parameter list | `.\start-native.ps1 --help` |

---

## How It Works

Local brute-force: random private key → secp256k1 public key → Keccak-256 address → match prefix/suffix → save if hit, repeat if not.

Optimization: advances the public key by adding the generator point, avoiding full key derivation on every attempt.

Difficulty (averages, not guarantees):

```text
Suffix 0000       ~65,536 attempts
Suffix 000000     ~16.8 million attempts
Suffix 00000000   ~4.3 billion attempts
Suffix 000000000  ~68.7 billion attempts
```

At 2M addr/s: `00000000` ≈ 36 min, `000000000` ≈ 9.5 hours.

## Supported Chains

Ethereum, BSC, Polygon, Arbitrum, Optimism, Base — any EVM chain using `0x...` format.

**Not supported**: Bitcoin, Solana, Tron, or any non-EVM chain.

---

## FAQ

**Computer slow?** Reduce workers: `-Workers 4`

**Taking too long?** Normal. Benchmark first, then estimate from your `rate`.

**Match found but no private key?** Check for `-RedactPrivateKey` (testing flag only).

**Cargo/Rust not found?** A prebuilt `bin\vanity-native.exe` is included. Run directly.

**Where are results?** `results\matched-wallet-latest.txt` — contains `Address` (receiving) and `PrivateKey` (control). **Back it up. Keep it secret.**
