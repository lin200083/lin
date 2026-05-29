use std::path::PathBuf;
use std::time::Duration;
use crate::cli::Cli;
use crate::error::{Result, VanityError};

pub struct Config {
    pub prefix: String,
    pub suffix: String,
    pub prefix_nibbles: Vec<u8>,
    pub suffix_nibbles: Vec<u8>,
    pub workers: usize,
    pub status_interval: Duration,
    pub batch_size: usize,
    pub case_sensitive: bool,
    pub redact_private_key: bool,
    pub plain_output: bool,
    pub max_seconds: u64,
    pub state_dir: PathBuf,
    pub result_dir: PathBuf,
    pub logs_dir: PathBuf,
}

impl Config {
    pub fn from_cli(cli: Cli) -> Result<Self> {
        let prefix = normalize_hex_pattern(&cli.prefix, "prefix", cli.case_sensitive)?;
        let suffix = normalize_hex_pattern(&cli.suffix, "suffix", cli.case_sensitive)?;

        if prefix.is_empty() && suffix.is_empty() {
            return Err(VanityError::NoPattern);
        }

        if prefix.len() + suffix.len() > 40 {
            return Err(VanityError::PatternTooLong);
        }

        if cli.workers < 1 {
            return Err(VanityError::InvalidWorkers);
        }

        if cli.status_interval < 1 {
            return Err(VanityError::InvalidInterval);
        }

        if cli.batch_size < 1 {
            return Err(VanityError::InvalidBatchSize);
        }

        Ok(Config {
            prefix_nibbles: hex_to_nibbles(&prefix)?,
            suffix_nibbles: hex_to_nibbles(&suffix)?,
            prefix,
            suffix,
            workers: cli.workers,
            status_interval: Duration::from_secs(cli.status_interval),
            batch_size: cli.batch_size,
            case_sensitive: cli.case_sensitive,
            redact_private_key: cli.redact_private_key,
            plain_output: cli.plain_output,
            max_seconds: cli.max_seconds,
            state_dir: cli.state_dir,
            result_dir: cli.result_dir,
            logs_dir: cli.logs_dir,
        })
    }

    pub fn average_attempts_text(&self) -> String {
        let digits = self.prefix.len() + self.suffix.len();
        let letters = self.checksum_sensitive_letter_count();

        self.average_attempts_plain()
            .map(format_number)
            .unwrap_or_else(|| {
                if self.case_sensitive && letters > 0 {
                    format!("16^{digits} x 2^{letters}")
                } else {
                    format!("16^{digits}")
                }
            })
    }

    fn average_attempts_plain(&self) -> Option<u64> {
        let digits = self.prefix.len() + self.suffix.len();
        let letters = self.checksum_sensitive_letter_count();
        let mut value = 1u64;

        for _ in 0..digits {
            value = value.checked_mul(16)?;
        }

        if self.case_sensitive {
            for _ in 0..letters {
                value = value.checked_mul(2)?;
            }
        }

        Some(value)
    }

    fn checksum_sensitive_letter_count(&self) -> usize {
        if !self.case_sensitive {
            return 0;
        }

        self.prefix
            .chars()
            .chain(self.suffix.chars())
            .filter(|ch| matches!(ch, 'a'..='f' | 'A'..='F'))
            .count()
    }
}

fn normalize_hex_pattern(value: &str, name: &str, preserve_case: bool) -> Result<String> {
    let mut normalized = value.trim().to_string();
    if normalized.to_ascii_lowercase().starts_with("0x") {
        normalized = normalized[2..].to_string();
    }

    if !normalized.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(VanityError::InvalidHex(format!(
            "{} must contain only hexadecimal characters, optionally prefixed by 0x",
            name
        )));
    }

    if preserve_case {
        Ok(normalized)
    } else {
        Ok(normalized.to_ascii_lowercase())
    }
}

pub fn hex_to_nibbles(value: &str) -> Result<Vec<u8>> {
    value
        .bytes()
        .map(|byte| match byte {
            b'0'..=b'9' => Ok(byte - b'0'),
            b'a'..=b'f' => Ok(byte - b'a' + 10),
            b'A'..=b'F' => Ok(byte - b'A' + 10),
            _ => Err(VanityError::InvalidHex("invalid hex character".to_string())),
        })
        .collect()
}

fn format_number(value: u64) -> String {
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
