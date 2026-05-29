use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use rand::SeedableRng;
use rand::rngs::OsRng;
use rand::RngCore;
use rand_chacha::ChaCha20Rng;
use chrono::Local;

use crate::config::Config;
use crate::crypto::{self, CryptoContext};
use crate::output::MatchResult;

pub struct ThreadState {
    pub attempts: AtomicU64,
    pub alive: AtomicBool,
}

pub fn worker_loop(
    worker_id: usize,
    config: Arc<Config>,
    stop: Arc<AtomicBool>,
    found: Arc<AtomicBool>,
    match_result: Arc<Mutex<Option<MatchResult>>>,
    states: Arc<Vec<ThreadState>>,
) {
    let crypto_ctx = CryptoContext::new();
    let mut seed = <ChaCha20Rng as SeedableRng>::Seed::default();
    OsRng.fill_bytes(&mut seed);
    let mut rng = ChaCha20Rng::from_seed(seed);
    let (mut sequence_secret_key, mut public_key) =
        crypto::random_sequence_start(crypto_ctx.secp(), &mut rng);
    let mut sequence_offset = 0u64;
    let mut attempts = 0u64;
    states[worker_id - 1].alive.store(true, Ordering::SeqCst);

    while !stop.load(Ordering::Relaxed) && !found.load(Ordering::Relaxed) {
        for _ in 0..config.batch_size {
            // Optimize: check stop/found only per batch, not per attempt
            attempts = attempts.wrapping_add(1);

            let serialized = public_key.serialize_uncompressed();
            let hash = crypto::keccak(&serialized[1..]);

            if !crypto::matches_address_hash(
                &hash,
                &config.prefix_nibbles,
                &config.suffix_nibbles,
            ) {
                crypto::advance_sequence(
                    crypto_ctx.secp(),
                    crypto_ctx.generator_key(),
                    &mut sequence_secret_key,
                    &mut public_key,
                    &mut sequence_offset,
                    &mut rng,
                );
                continue;
            }

            let address_body = crypto::last20_to_hex(&hash);
            let comparable_body = if config.case_sensitive {
                crypto::checksum_body(&address_body)
            } else {
                address_body.clone()
            };

            if !crypto::matches_address_body(&comparable_body, &config.prefix, &config.suffix) {
                crypto::advance_sequence(
                    crypto_ctx.secp(),
                    crypto_ctx.generator_key(),
                    &mut sequence_secret_key,
                    &mut public_key,
                    &mut sequence_offset,
                    &mut rng,
                );
                continue;
            }

            let private_key = crypto::sequence_private_key(&sequence_secret_key, sequence_offset);
            states[worker_id - 1]
                .attempts
                .store(attempts, Ordering::Relaxed);
            if !found.swap(true, Ordering::SeqCst) {
                stop.store(true, Ordering::SeqCst);
                let address = format!("0x{}", crypto::checksum_body(&address_body));
                let result = MatchResult {
                    address,
                    private_key: format!("0x{}", crypto::bytes_to_hex(&private_key)),
                    worker_id,
                    worker_attempts: attempts,
                    found_at: Local::now().to_rfc3339(),
                };

                if let Ok(mut slot) = match_result.lock() {
                    *slot = Some(result);
                }
            }

            states[worker_id - 1].alive.store(false, Ordering::SeqCst);
            return;
        }

        states[worker_id - 1]
            .attempts
            .store(attempts, Ordering::Relaxed);
    }

    states[worker_id - 1].alive.store(false, Ordering::SeqCst);
    states[worker_id - 1]
        .attempts
        .store(attempts, Ordering::Relaxed);
}

pub fn total_attempts(states: &[ThreadState]) -> u64 {
    states
        .iter()
        .map(|state| state.attempts.load(Ordering::Relaxed))
        .sum()
}

pub fn alive_workers(states: &[ThreadState]) -> usize {
    states
        .iter()
        .filter(|state| state.alive.load(Ordering::Relaxed))
        .count()
}
