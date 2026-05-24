[English](README_EN.md) | [中文](README.md)

# Vanity Wallet Generator

An **EVM vanity address generator** that runs on Windows.

**For beginners: download the beginner pack (`vanity-wallet-generator-windows-x64-vX.Y.Z.zip`) from [GitHub Releases](https://github.com/lin200083/vanity-wallet-generator/releases/latest), unzip, and double-click `双击我运行.bat`** — an interactive menu guides you through every step. No commands needed.

If you prefer the command line, use `start-native.ps1` directly with parameters.

It continuously generates random private keys, derives `0x...` addresses, checks whether they match your desired prefix or suffix, and saves matches to the `results` folder.

## Supported Chains

Works with:

- Ethereum
- BSC / BNB Chain
- Polygon
- Arbitrum
- Optimism
- Base
- Any chain using EVM `0x...` address format

Does NOT work with:

- Bitcoin
- Solana
- Tron native `T...` addresses
- Non-`0x...` wallet addresses

## Security Warning

Please read this carefully.

- A private key is full control of the assets. Anyone who sees it can transfer everything.
- Never share your private key online, in screenshots, or in cloud notes.
- Do NOT use online vanity address generators — your private key may be leaked.
- Back up `PrivateKey` immediately after a match.
- Before transferring significant funds, test with a small amount first.
- If you run with `-RedactPrivateKey`, the result file will not contain a usable private key.

## How It Works

This tool does not "create" a specific address — it brute-forces locally:

1. Each worker generates a random 32-byte starting private key.
2. Derives the starting public key via secp256k1.
3. Subsequent attempts advance the public key by adding the generator point — avoiding a full key derivation every time.
4. Computes the EVM address via Keccak-256.
5. Checks if the address matches the prefix or suffix rules.
6. If not, continues searching.
7. On a match, recovers the current private key, saves address and key, then stops.

Each fixed hex character multiplies the difficulty by 16.

```text
Suffix 0000       ~65,536 attempts on average
Suffix 000000     ~16,777,216 attempts on average
Suffix 00000000   ~4,294,967,296 attempts on average
Suffix 000000000  ~68,719,476,736 attempts on average
```

These are averages, not guarantees. You might get lucky — or unlucky.

## File Structure

Key files in the project root:

```text
双击我运行.bat           Beginner entry point — double-click to launch interactive menu
easy-start.ps1          Interactive guided script — step-by-step parameter setup
start-native.ps1        CLI launcher for power users
Build-Native.ps1        Compile the Rust native executable
Measure-NativeSpeed.ps1 Benchmark speed
Get-Status.ps1          Check current or last run status
Clean-Generated.ps1     Clean build caches and benchmark artifacts
Release-Pack.ps1        Generate source and beginner Windows zip packages
Upload-Release.ps1      Upload release artifacts to GitHub Releases
bin\vanity-native.exe   Compiled Windows executable
native\vanity-native\   Rust source code
results\                Match results
state\                  Status files
logs\                   Log files
```

## Cleaning Generated Files

Clean Rust build cache, benchmark state, and empty benchmark result directories:

```powershell
.\Clean-Generated.ps1
```

By default it does NOT delete wallet result files (they may contain private keys). Only use this if you're sure:

```powershell
.\Clean-Generated.ps1 -IncludeWalletResults
```

## First Run

### Interactive Mode (Recommended for Beginners)

Double-click `双击我运行.bat` in File Explorer, or open PowerShell and run:

```powershell
.\easy-start.ps1
```

Follow the on-screen prompts to select search mode, enter your pattern, and confirm. No need to remember any parameters.

If you get an execution policy error, temporarily bypass it:

```powershell
Set-ExecutionPolicy -Scope Process -ExecutionPolicy Bypass
```

### CLI Mode (For Power Users)

Open PowerShell and navigate to the project directory.

If the project is on your Desktop:

```powershell
cd "$env:USERPROFILE\Desktop\vanity-wallet-generator"
```

Run a quick test:

```powershell
.\start-native.ps1 -Suffix "0000" -Workers 4 -PreventSleep
```

If `bin\vanity-native.exe` does not exist, the script will auto-compile:

```powershell
.\Build-Native.ps1
```

You can also compile manually:

```powershell
.\Build-Native.ps1
```

## Full Run Examples

Search for 8 zeros suffix:

```powershell
.\start-native.ps1 -Suffix "00000000" -Workers 8 -PreventSleep
```

Search for 9 eights suffix:

```powershell
.\start-native.ps1 -Suffix "888888888" -Workers 8 -PreventSleep
```

Prefix only:

```powershell
.\start-native.ps1 -Prefix "000000" -Workers 8 -PreventSleep
```

Both prefix and suffix:

```powershell
.\start-native.ps1 -Prefix "0000" -Suffix "000000" -Workers 8 -PreventSleep
```

If your computer feels sluggish, reduce workers:

```powershell
.\start-native.ps1 -Suffix "00000000" -Workers 4 -PreventSleep
```

If you have many CPU cores, try more workers:

```powershell
.\start-native.ps1 -Suffix "00000000" -Workers 12 -PreventSleep
```

A good rule of thumb: leave 1-2 cores free for the system.

## Parameter Reference

### `-Prefix`

Address prefix, without `0x`.

```powershell
.\start-native.ps1 -Prefix "0000" -Workers 8
```

You can include `0x` — the script handles it automatically:

```powershell
.\start-native.ps1 -Prefix "0x0000" -Workers 8
```

### `-Suffix`

Address suffix, without `0x`.

Default value:

```text
00000000
```

So running without arguments defaults to suffix `00000000`:

```powershell
.\start-native.ps1
```

### `-Workers`

Number of parallel threads.

Common values:

```text
4    Low resource usage
8    Recommended starting point
12   For high-core-count CPUs
```

### `-PreventSleep

Prevents Windows from sleeping while running. Recommended for long runs:

```powershell
.\start-native.ps1 -Suffix "00000000" -Workers 8 -PreventSleep
```

It does not prevent manual shutdown and may not cover all power policies. For long runs, also check your Windows power settings.

### `-RedactPrivateKey

Hides the private key in result files — for testing only:

```powershell
.\start-native.ps1 -Suffix "0000" -Workers 4 -RedactPrivateKey
```

Do NOT use this for real searches. The result file will show:

```text
PrivateKey: [redacted by --redact-private-key]
```

Making the wallet unusable.

### `-PlainOutput

Restores line-by-line output mode.

By default, status updates refresh on the same line to avoid spam. Use this only if you want to keep terminal logs:

```powershell
.\start-native.ps1 -Suffix "00000000" -Workers 8 -PreventSleep -PlainOutput
```

### `-NoBuild`

Skip auto-compilation.

If `bin\vanity-native.exe` already exists:

```powershell
.\start-native.ps1 -Suffix "0000" -Workers 4 -NoBuild
```

Do not use this if the exe does not exist.

### Advanced Parameters

Usually not needed:

```text
-StatusIntervalSeconds   Status refresh interval, default 5s
-BatchSize               Attempts per batch, default 1024
-MaxSeconds              Max runtime in seconds, 0 = unlimited
-CaseSensitive           EIP-55 checksum exact match, not recommended
```

When `-CaseSensitive` is enabled and the pattern contains `a-f` letters, the actual difficulty increases because the address must also satisfy the EIP-55 checksum casing constraint.

## Runtime Output

When started, the tool first displays task info:

```text
Native EVM vanity search
Run ID: 20260423-120000000
Target: prefix '-' suffix '00000000'
Workers: 8
Average attempts estimate: 4,294,967,296
Status updates will refresh on one line. Use -PlainOutput for scrolling output.
```

Then updates status on the same line:

```text
[12:00:05] attempts=9,427,968 rate=1,709,062/s runtime=00:00:05 workers=8/8
```

Fields:

```text
attempts   Total attempts so far
rate       Current address generation/check speed per second
runtime    Elapsed time
workers    Current active workers
```

Default refresh is every 5 seconds. For more frequent updates:

```powershell
.\start-native.ps1 -Suffix "00000000" -Workers 8 -PreventSleep -StatusIntervalSeconds 1
```

## Benchmarking

Measure speed without waiting for a match:

```powershell
.\Measure-NativeSpeed.ps1 -Workers 8 -Seconds 20
```

It uses a practically unreachable target and runs for a fixed duration to show your `rate`.

Reference speeds from the test machine (your mileage will vary):

```text
8 workers     1,700,000 to 2,000,000 addr/s
12 workers    1,800,000 to 2,100,000 addr/s
```

Actual speed depends on your CPU, background load, and Windows power plan.

## Time Estimates

Rough estimates at `2,000,000 addr/s`:

```text
Suffix 0000       ~0.03 sec
Suffix 000000     ~8 sec
Suffix 00000000   ~36 min
Suffix 000000000  ~9.5 hours
```

These are averages, not guarantees.

## Checking Status from Another Window

Open a second PowerShell:

```powershell
cd "$env:USERPROFILE\Desktop\vanity-wallet-generator"
.\Get-Status.ps1
```

Example output:

```text
Run ID:        20260423-120000000
Engine:        native-rust
Target:        prefix '' suffix '00000000'
Attempts:      123456789
Rate:          220000 / sec
Runtime:       00:09:21
Workers:       8 / 8
Restarts:      0
Matched:       False
Last updated:  04/23/2026 12:30:00
```

## How to Stop

Press `Ctrl+C` in the running window.

To resume later, run the same command again:

```powershell
.\start-native.ps1 -Suffix "00000000" -Workers 8 -PreventSleep
```

The search uses random sampling — no progress to save. Every start is a fresh random search.

## When a Match Is Found

Results are saved to:

```text
results\
```

Each match generates:

```text
matched-wallet-native-<run-id>.txt
```

And updates:

```text
matched-wallet-latest.txt
```

Result file example:

```text
EVM Vanity Wallet Match

Engine: native-rust
RunId: 20260423-120000000
FoundAt: 2026-04-23T12:00:00+08:00
Address: 0x...
PrivateKey: 0x...
Prefix: -
Suffix: 00000000
CaseSensitive: false
EstimatedAverageAttempts: 4294967296
TotalAttemptsObserved: ...
WorkerId: ...
WorkerAttemptsThisRun: ...
```

Key fields:

```text
Address     Wallet address — can receive funds
PrivateKey  Private key — import into a wallet to control assets
```

**Always back up the `PrivateKey`.**

## FAQ

### Downloading Pre-built Releases

If you don't want to compile yourself, download from GitHub Releases:

```text
https://github.com/lin200083/vanity-wallet-generator/releases/latest
```

**For beginners**, download this zip:

```text
vanity-wallet-generator-windows-x64-vX.Y.Z.zip
```

Unzip and **double-click `双击我运行.bat`** — no PowerShell required.

Or from the CLI:

```powershell
.\start-native.ps1
```

If you only want the standalone exe:

```text
vanity-native-windows-x64-vX.Y.Z.exe
```

Place it in the `bin` directory and rename to:

```text
vanity-native.exe
```

Then `start-native.ps1` can call it directly.

### Script Won't Run

Run this to temporarily allow script execution:

```powershell
Set-ExecutionPolicy -Scope Process -ExecutionPolicy Bypass
```

This only affects the current PowerShell window.

### Cargo or Rust Not Found

If the build script says `cargo` is not found, Rust is not in your PATH.

On this machine it's already compiled:

```text
bin\vanity-native.exe
```

If you move to another machine, either install Rust or bring the prebuilt `bin\vanity-native.exe` with you.

### Match Found but No Private Key in Result

Check if you used:

```powershell
-RedactPrivateKey
```

This flag is only for testing. Do not use it for real searches.

### Computer Feels Slow

Reduce the number of workers:

```powershell
.\start-native.ps1 -Suffix "00000000" -Workers 4 -PreventSleep
```

### Taking Too Long

This is normal. `00000000` suffix averages ~4.3 billion attempts. `888888888` (9 digits) averages ~68.7 billion.

Benchmark first:

```powershell
.\Measure-NativeSpeed.ps1 -Workers 8 -Seconds 20
```

Then estimate based on your actual `rate`.

## Recommended Workflow

### Interactive (Beginner)

1. Double-click `双击我运行.bat` (or run `.\easy-start.ps1`)
2. Follow the menu: select search mode, enter your pattern
3. Confirm and let it run
4. Open the result file on match, back up `PrivateKey`

### CLI (Power User)

1. Navigate to the directory:

```powershell
cd "$env:USERPROFILE\Desktop\vanity-wallet-generator"
```

2. Quick test:

```powershell
.\start-native.ps1 -Suffix "0000" -Workers 4 -PreventSleep
```

3. Benchmark:

```powershell
.\Measure-NativeSpeed.ps1 -Workers 8 -Seconds 20
```

4. Full run:

```powershell
.\start-native.ps1 -Suffix "00000000" -Workers 8 -PreventSleep
```

5. Open result on match:

```text
results\matched-wallet-latest.txt
```

6. Back up `PrivateKey`.

7. Test with a small transfer to confirm the wallet works.

8. Only then consider moving significant funds.
