use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "vanity-native")]
#[command(about = "Native EVM vanity address generator")]
pub struct Cli {
    #[arg(long, default_value = "")]
    pub prefix: String,

    #[arg(long, default_value = "00000000")]
    pub suffix: String,

    #[arg(long, default_value_t = usize::max(1, num_cpus::get().saturating_sub(1)))]
    pub workers: usize,

    #[arg(long = "status-interval", default_value_t = 5)]
    pub status_interval: u64,

    #[arg(long = "batch-size", default_value_t = 1024)]
    pub batch_size: usize,

    #[arg(long = "max-seconds", default_value_t = 0)]
    pub max_seconds: u64,

    #[arg(long = "state-dir", default_value = "state")]
    pub state_dir: PathBuf,

    #[arg(long = "result-dir", default_value = "results")]
    pub result_dir: PathBuf,

    #[arg(long = "logs-dir", default_value = "logs")]
    pub logs_dir: PathBuf,

    #[arg(long = "case-sensitive", default_value_t = false)]
    pub case_sensitive: bool,

    #[arg(long = "redact-private-key", default_value_t = false)]
    pub redact_private_key: bool,

    #[arg(long = "plain-output", default_value_t = false)]
    pub plain_output: bool,
}
