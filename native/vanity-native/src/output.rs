use chrono::Local;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::Path;
use std::time::Duration;
use crate::config::Config;
use crate::error::{Result, VanityError};

pub struct StatusSnapshot {
    pub attempts: u64,
    pub rate: u64,
    pub runtime: String,
    pub alive_workers: usize,
    pub matched: bool,
}

pub struct MatchResult {
    pub address: String,
    pub private_key: String,
    pub worker_id: usize,
    pub worker_attempts: u64,
    pub found_at: String,
}

pub fn print_banner(config: &Config, run_id: &str) {
    println!("Native EVM vanity search");
    println!("Run ID: {run_id}");
    println!(
        "Target: prefix '{}' suffix '{}'",
        display_pattern(&config.prefix),
        display_pattern(&config.suffix)
    );
    println!("Workers: {}", config.workers);
    println!(
        "Average attempts estimate: {}",
        config.average_attempts_text()
    );
    if !config.plain_output {
        println!("Status updates will refresh on one line. Use -PlainOutput for scrolling output.");
    }
}

pub fn print_status(
    config: &Config,
    snapshot: &StatusSnapshot,
    last_live_len: &mut usize,
) -> Result<()> {
    let line = format!(
        "[{}] attempts={} rate={}/s runtime={} workers={}/{}",
        Local::now().format("%H:%M:%S"),
        format_number(snapshot.attempts),
        format_number(snapshot.rate),
        snapshot.runtime,
        snapshot.alive_workers,
        config.workers
    );

    if config.plain_output {
        println!("{line}");
    } else {
        let padding = last_live_len.saturating_sub(line.len());
        print!("\r{line}{}", " ".repeat(padding));
        io::stdout()
            .flush()
            .map_err(|e| VanityError::Msg(format!("flush stdout failed: {e}")))?;
        *last_live_len = line.len();
    }

    Ok(())
}

pub fn finish_live_line(config: &Config, last_live_len: &mut usize) -> Result<()> {
    if config.plain_output || *last_live_len == 0 {
        return Ok(());
    }

    print!("\r{}\r\n", " ".repeat(*last_live_len));
    io::stdout()
        .flush()
        .map_err(|e| VanityError::Msg(format!("flush stdout failed: {e}")))?;
    *last_live_len = 0;
    Ok(())
}

pub fn write_status(config: &Config, run_id: &str, snapshot: &StatusSnapshot) -> Result<()> {
    let json = format!(
        concat!(
            "{{\n",
            "  \"runId\": \"{}\",\n",
            "  \"updatedAt\": \"{}\",\n",
            "  \"matched\": {},\n",
            "  \"engine\": \"native-rust\",\n",
            "  \"pattern\": {{\n",
            "    \"prefix\": \"{}\",\n",
            "    \"suffix\": \"{}\",\n",
            "    \"caseSensitive\": {}\n",
            "  }},\n",
            "  \"totalAttempts\": {},\n",
            "  \"totalRatePerSecond\": {},\n",
            "  \"runtime\": \"{}\",\n",
            "  \"aliveWorkers\": {},\n",
            "  \"configuredWorkers\": {},\n",
            "  \"totalRestarts\": 0\n",
            "}}\n"
        ),
        run_id,
        Local::now().to_rfc3339(),
        snapshot.matched,
        config.prefix,
        config.suffix,
        config.case_sensitive,
        snapshot.attempts,
        snapshot.rate,
        snapshot.runtime,
        snapshot.alive_workers,
        config.workers,
    );

    atomic_write(&config.state_dir.join("status.json"), json.as_bytes())
}

pub fn write_result(
    config: &Config,
    run_id: &str,
    result: &MatchResult,
    total_attempts: u64,
) -> Result<std::path::PathBuf> {
    let result_path = config
        .result_dir
        .join(format!("matched-wallet-native-{run_id}.txt"));
    let private_key = if config.redact_private_key {
        String::from("[redacted by --redact-private-key]")
    } else {
        result.private_key.clone()
    };

    let body = format!(
        concat!(
            "EVM Vanity Wallet Match\n\n",
            "Engine: native-rust\n",
            "RunId: {}\n",
            "FoundAt: {}\n",
            "Address: {}\n",
            "PrivateKey: {}\n",
            "Prefix: {}\n",
            "Suffix: {}\n",
            "CaseSensitive: {}\n",
            "EstimatedAverageAttempts: {}\n",
            "TotalAttemptsObserved: {}\n",
            "WorkerId: {}\n",
            "WorkerAttemptsThisRun: {}\n\n",
            "Security notes:\n",
            "- Keep the private key offline and never paste it into websites.\n",
            "- Fund the address only after you have backed up the private key.\n",
            "- Anyone who sees this private key can spend funds from this address.\n"
        ),
        run_id,
        result.found_at,
        result.address,
        private_key,
        display_pattern(&config.prefix),
        display_pattern(&config.suffix),
        config.case_sensitive,
        config.average_attempts_text(),
        total_attempts,
        result.worker_id,
        result.worker_attempts,
    );

    atomic_write(&result_path, body.as_bytes())?;
    atomic_write(
        &config.result_dir.join("matched-wallet-latest.txt"),
        body.as_bytes(),
    )?;
    Ok(result_path)
}

pub fn log_event(config: &Config, message: &str) -> Result<()> {
    fs::create_dir_all(&config.logs_dir)
        .map_err(|e| VanityError::Msg(format!("create logs dir failed: {e}")))?;
    let log_path = config
        .logs_dir
        .join(format!("{}.log", Local::now().format("%Y-%m-%d")));
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .map_err(|e| VanityError::Msg(format!("open log {} failed: {e}", log_path.display())))?;
    writeln!(file, "[{}] {}", Local::now().to_rfc3339(), message)
        .map_err(|e| VanityError::Msg(format!("write log failed: {e}")))
}

pub fn format_duration(duration: Duration) -> String {
    let seconds = duration.as_secs();
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let seconds = seconds % 60;
    format!("{hours:02}:{minutes:02}:{seconds:02}")
}

pub fn format_number(value: u64) -> String {
    let text = value.to_string();
    let mut output = String::with_capacity(text.len() + text.len() / 3);
    for (index, character) in text.chars().rev().enumerate() {
        if index > 0 && index % 3 == 0 {
            output.push(',');
        }
        output.push(character);
    }
    output.chars().rev().collect()
}

pub fn display_pattern(value: &str) -> &str {
    if value.is_empty() {
        "-"
    } else {
        value
    }
}

fn atomic_write(path: &Path, bytes: &[u8]) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| VanityError::Msg(format!("create dir {} failed: {e}", parent.display())))?;
    }

    let temp_path = path.with_file_name(format!(
        "{}.{}.{}.tmp",
        path.file_name().unwrap_or_default().to_string_lossy(),
        std::process::id(),
        Local::now().timestamp_nanos_opt().unwrap_or_default()
    ));

    {
        let mut file = fs::File::create(&temp_path)
            .map_err(|e| VanityError::Msg(format!("create {} failed: {e}", temp_path.display())))?;
        file.write_all(bytes)
            .map_err(|e| VanityError::Msg(format!("write {} failed: {e}", temp_path.display())))?;
        file.sync_all()
            .map_err(|e| VanityError::Msg(format!("sync {} failed: {e}", temp_path.display())))?;
    }

    replace_file(&temp_path, path)
}

fn replace_file(temp_path: &Path, path: &Path) -> Result<()> {
    #[cfg(windows)]
    {
        replace_file_windows(temp_path, path)
    }

    #[cfg(not(windows))]
    {
        fs::rename(temp_path, path).map_err(|e| {
            VanityError::Msg(format!(
                "rename {} to {} failed: {e}",
                temp_path.display(),
                path.display()
            ))
        })
    }
}

#[cfg(windows)]
fn replace_file_windows(temp_path: &Path, path: &Path) -> Result<()> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::{
        MoveFileExW, MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH,
    };

    fn to_wide(path: &Path) -> Vec<u16> {
        path.as_os_str()
            .encode_wide()
            .chain(std::iter::once(0))
            .collect()
    }

    let from = to_wide(temp_path);
    let to = to_wide(path);

    let ok = unsafe {
        MoveFileExW(
            from.as_ptr(),
            to.as_ptr(),
            MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
        )
    };

    if ok == 0 {
        return Err(VanityError::Msg(format!(
            "replace {} with {} failed",
            path.display(),
            temp_path.display()
        )));
    }

    Ok(())
}
