mod cli;
mod config;
mod crypto;
mod error;
mod output;
mod worker;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

use clap::Parser;
use chrono::Local;

use cli::Cli;
use config::Config;
use error::Result;
use output::{MatchResult, StatusSnapshot};
use worker::ThreadState;

fn main() {
    if let Err(error) = run() {
        eprintln!("Error: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    let config = Arc::new(Config::from_cli(cli)?);
    std::fs::create_dir_all(&config.state_dir)?;
    std::fs::create_dir_all(&config.result_dir)?;
    std::fs::create_dir_all(&config.logs_dir)?;

    let run_id = Local::now().format("%Y%m%d-%H%M%S%3f").to_string();
    output::log_event(
        &config,
        &format!(
            "run {run_id} started prefix={} suffix={} workers={}",
            output::display_pattern(&config.prefix),
            output::display_pattern(&config.suffix),
            config.workers
        ),
    )?;
    output::print_banner(&config, &run_id);

    let stop = Arc::new(AtomicBool::new(false));
    let interrupted = Arc::new(AtomicBool::new(false));
    let found = Arc::new(AtomicBool::new(false));
    let match_result = Arc::new(Mutex::new(None::<MatchResult>));
    let states = Arc::new(
        (0..config.workers)
            .map(|_| ThreadState {
                attempts: std::sync::atomic::AtomicU64::new(0),
                alive: AtomicBool::new(false),
            })
            .collect::<Vec<_>>(),
    );

    {
        let stop = Arc::clone(&stop);
        let interrupted = Arc::clone(&interrupted);
        ctrlc::set_handler(move || {
            interrupted.store(true, Ordering::SeqCst);
            stop.store(true, Ordering::SeqCst);
        })
        .map_err(|e| format!("failed to install Ctrl+C handler: {e}"))?;
    }

    let mut handles = Vec::with_capacity(config.workers);
    for worker_index in 0..config.workers {
        let worker_config = Arc::clone(&config);
        let worker_stop = Arc::clone(&stop);
        let worker_found = Arc::clone(&found);
        let worker_result = Arc::clone(&match_result);
        let worker_states = Arc::clone(&states);

        handles.push(thread::spawn(move || {
            worker::worker_loop(
                worker_index + 1,
                worker_config,
                worker_stop,
                worker_found,
                worker_result,
                worker_states,
            );
        }));
    }

    let started = Instant::now();
    let mut next_status = Instant::now() + config.status_interval;
    let mut last_attempts = 0u64;
    let mut last_status_at = Instant::now();
    let mut last_live_len = 0usize;
    let mut last_rate = 0u64;
    let stop_reason: String;

    loop {
        if found.load(Ordering::Relaxed) {
            stop_reason = String::from("match found");
            break;
        }

        if stop.load(Ordering::Relaxed) {
            stop_reason = if interrupted.load(Ordering::Relaxed) {
                String::from("interrupted by Ctrl+C")
            } else {
                String::from("stopped")
            };
            break;
        }

        if config.max_seconds > 0 && started.elapsed().as_secs() >= config.max_seconds {
            stop_reason = format!("max run time reached after {}s", config.max_seconds);
            stop.store(true, Ordering::SeqCst);
            break;
        }

        if Instant::now() >= next_status {
            let attempts = worker::total_attempts(&states);
            let elapsed = last_status_at.elapsed().as_secs_f64().max(0.001);
            last_rate = ((attempts.saturating_sub(last_attempts)) as f64 / elapsed).round() as u64;
            last_attempts = attempts;
            last_status_at = Instant::now();

            let snapshot = StatusSnapshot {
                attempts,
                rate: last_rate,
                runtime: output::format_duration(started.elapsed()),
                alive_workers: worker::alive_workers(&states),
                matched: false,
            };

            output::write_status(&config, &run_id, &snapshot)?;
            output::print_status(&config, &snapshot, &mut last_live_len)?;
            next_status += config.status_interval;
        }

        thread::sleep(std::time::Duration::from_millis(50));
    }

    stop.store(true, Ordering::SeqCst);
    for handle in handles {
        let _ = handle.join();
    }

    output::finish_live_line(&config, &mut last_live_len)?;

    let final_attempts = worker::total_attempts(&states);
    let final_snapshot = StatusSnapshot {
        attempts: final_attempts,
        rate: last_rate,
        runtime: output::format_duration(started.elapsed()),
        alive_workers: worker::alive_workers(&states),
        matched: found.load(Ordering::Relaxed),
    };
    output::write_status(&config, &run_id, &final_snapshot)?;

    let result = match_result
        .lock()
        .map_err(|_| "match result lock poisoned".to_string())?
        .take();
    if let Some(result) = result {
        let result_path = output::write_result(&config, &run_id, &result, final_attempts)?;
        output::log_event(
            &config,
            &format!(
                "match found address={} result={}",
                result.address,
                result_path.display()
            ),
        )?;
        println!("MATCH FOUND");
        println!("Address: {}", result.address);
        println!("Result:  {}", result_path.display());
    } else {
        output::log_event(
            &config,
            &format!("run {run_id} stopping: {stop_reason} attempts={final_attempts}"),
        )?;
        println!("Stopped: {stop_reason}");
        println!("Attempts: {}", output::format_number(final_attempts));
    }

    if interrupted.load(Ordering::Relaxed) {
        std::process::exit(130);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crypto::*;

    #[test]
    fn sequence_private_key_matches_incremented_public_key() {
        let ctx = CryptoContext::new();
        let mut rng = rand_chacha::ChaCha20Rng::from_seed([7u8; 32]);
        let (sequence_secret_key, mut public_key) =
            random_sequence_start(ctx.secp(), &mut rng);

        for offset in 0..128u64 {
            let private_key = sequence_private_key(&sequence_secret_key, offset);
            let secret_key =
                secp256k1::SecretKey::from_byte_array(private_key).unwrap();
            let derived_public_key =
                secp256k1::PublicKey::from_secret_key(ctx.secp(), &secret_key);

            assert_eq!(
                derived_public_key.serialize_uncompressed(),
                public_key.serialize_uncompressed()
            );

            public_key = public_key.combine(ctx.generator_key()).unwrap();
        }
    }

    #[test]
    fn hex_to_nibbles_parses_all_digits() {
        use config::hex_to_nibbles;
        assert_eq!(
            hex_to_nibbles("0123456789abcdef").unwrap(),
            (0..16).collect::<Vec<u8>>()
        );
        assert_eq!(
            hex_to_nibbles("ABCDEF").unwrap(),
            vec![10, 11, 12, 13, 14, 15]
        );
    }

    #[test]
    fn keccak_output_length() {
        let hash = keccak(b"hello");
        assert_eq!(hash.len(), 32);
    }

    #[test]
    fn keccak_is_deterministic() {
        assert_eq!(keccak(b"test"), keccak(b"test"));
    }

    #[test]
    fn keccak_differs_on_input() {
        assert_ne!(keccak(b"a"), keccak(b"b"));
    }

    #[test]
    fn address_nibble_positions() {
        let mut hash = [0u8; 32];
        hash[12] = 0xAB;
        hash[13] = 0xCD;

        assert_eq!(address_nibble(&hash, 0), 0xA);
        assert_eq!(address_nibble(&hash, 1), 0xB);
        assert_eq!(address_nibble(&hash, 2), 0xC);
        assert_eq!(address_nibble(&hash, 3), 0xD);
    }

    #[test]
    fn matches_address_hash_prefix() {
        let mut hash = [0u8; 32];
        hash[12] = 0xAB;
        hash[13] = 0xC0;

        assert!(matches_address_hash(&hash, &[0xA, 0xB, 0xC], &[]));

        hash[12] = 0xBB;
        assert!(!matches_address_hash(&hash, &[0xA, 0xB, 0xC], &[]));
    }

    #[test]
    fn matches_address_hash_suffix() {
        let mut hash = [0u8; 32];
        hash[31] = 0xDE;

        assert!(matches_address_hash(&hash, &[], &[0xD, 0xE]));

        hash[31] = 0xDF;
        assert!(!matches_address_hash(&hash, &[], &[0xD, 0xE]));
    }

    #[test]
    fn checksum_body_changes_case() {
        let body = "abc123def456";
        let checksummed = checksum_body(body);
        assert_eq!(checksummed.len(), body.len());
        let lower = checksummed.to_ascii_lowercase();
        assert_eq!(lower, body);
    }

    #[test]
    fn format_duration_output() {
        use std::time::Duration;
        assert_eq!(output::format_duration(Duration::from_secs(0)), "00:00:00");
        assert_eq!(output::format_duration(Duration::from_secs(61)), "00:01:01");
        assert_eq!(output::format_duration(Duration::from_secs(3661)), "01:01:01");
    }

    #[test]
    fn format_number_output() {
        assert_eq!(output::format_number(0), "0");
        assert_eq!(output::format_number(1000), "1,000");
        assert_eq!(output::format_number(1234567), "1,234,567");
    }

    #[test]
    fn display_pattern_empty_is_dash() {
        assert_eq!(output::display_pattern(""), "-");
        assert_eq!(output::display_pattern("abc"), "abc");
    }
}
