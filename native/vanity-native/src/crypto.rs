use secp256k1::{constants, PublicKey, Scalar, Secp256k1, SecretKey};
use sha3::{Digest, Keccak256};
use rand::RngCore;
use rand_chacha::ChaCha20Rng;

const HEX: &[u8; 16] = b"0123456789abcdef";

pub struct CryptoContext {
    secp: Secp256k1<secp256k1::All>,
    generator_key: PublicKey,
}

impl CryptoContext {
    pub fn new() -> Self {
        let secp = Secp256k1::new();
        let generator_key = Self::generator_public_key(&secp);
        CryptoContext { secp, generator_key }
    }

    pub fn secp(&self) -> &Secp256k1<secp256k1::All> {
        &self.secp
    }

    pub fn generator_key(&self) -> &PublicKey {
        &self.generator_key
    }

    fn generator_public_key(secp: &Secp256k1<secp256k1::All>) -> PublicKey {
        let secret_key = SecretKey::from_byte_array(constants::ONE).expect("curve one is valid");
        PublicKey::from_secret_key(secp, &secret_key)
    }
}

pub fn random_sequence_start(
    secp: &Secp256k1<secp256k1::All>,
    rng: &mut ChaCha20Rng,
) -> (SecretKey, PublicKey) {
    let mut private_key = [0u8; 32];
    loop {
        rng.fill_bytes(&mut private_key);
        if let Ok(secret_key) = SecretKey::from_byte_array(private_key) {
            let public_key = PublicKey::from_secret_key(secp, &secret_key);
            return (secret_key, public_key);
        }
    }
}

pub fn advance_sequence(
    secp: &Secp256k1<secp256k1::All>,
    generator_key: &PublicKey,
    sequence_secret_key: &mut SecretKey,
    public_key: &mut PublicKey,
    sequence_offset: &mut u64,
    rng: &mut ChaCha20Rng,
) {
    if *sequence_offset == u64::MAX {
        (*sequence_secret_key, *public_key) = random_sequence_start(secp, rng);
        *sequence_offset = 0;
        return;
    }

    match public_key.combine(generator_key) {
        Ok(next_public_key) => {
            *public_key = next_public_key;
            *sequence_offset += 1;
        }
        Err(_) => {
            (*sequence_secret_key, *public_key) = random_sequence_start(secp, rng);
            *sequence_offset = 0;
        }
    }
}

pub fn sequence_private_key(sequence_secret_key: &SecretKey, sequence_offset: u64) -> [u8; 32] {
    if sequence_offset == 0 {
        return sequence_secret_key.secret_bytes();
    }

    let mut offset_bytes = [0u8; 32];
    offset_bytes[24..].copy_from_slice(&sequence_offset.to_be_bytes());
    let offset = Scalar::from_be_bytes(offset_bytes).expect("u64 offset is a valid scalar");
    sequence_secret_key
        .add_tweak(&offset)
        .expect("active sequence private key is valid")
        .secret_bytes()
}

pub fn keccak(bytes: &[u8]) -> [u8; 32] {
    let mut hasher = Keccak256::new();
    hasher.update(bytes);
    hasher.finalize().into()
}

pub fn last20_to_hex(hash: &[u8; 32]) -> String {
    let mut output = String::with_capacity(40);
    for byte in &hash[12..32] {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 0x0f) as usize] as char);
    }
    output
}

pub fn bytes_to_hex(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 0x0f) as usize] as char);
    }
    output
}

pub fn checksum_body(address_body: &str) -> String {
    let hash = keccak(address_body.as_bytes());
    let mut output = String::with_capacity(address_body.len());

    for (index, byte) in address_body.bytes().enumerate() {
        let hash_byte = hash[index >> 1];
        let nibble = if index % 2 == 0 {
            hash_byte >> 4
        } else {
            hash_byte & 0x0f
        };

        if (b'a'..=b'f').contains(&byte) && nibble >= 8 {
            output.push((byte - 32) as char);
        } else {
            output.push(byte as char);
        }
    }

    output
}

pub fn address_nibble(hash: &[u8; 32], address_nibble_index: usize) -> u8 {
    let byte = hash[12 + (address_nibble_index >> 1)];
    if address_nibble_index.is_multiple_of(2) {
        byte >> 4
    } else {
        byte & 0x0f
    }
}

pub fn matches_address_hash(hash: &[u8; 32], prefix_nibbles: &[u8], suffix_nibbles: &[u8]) -> bool {
    for (index, expected) in prefix_nibbles.iter().enumerate() {
        if address_nibble(hash, index) != *expected {
            return false;
        }
    }

    let suffix_start = 40usize.saturating_sub(suffix_nibbles.len());
    for (index, expected) in suffix_nibbles.iter().enumerate() {
        if address_nibble(hash, suffix_start + index) != *expected {
            return false;
        }
    }

    true
}

pub fn matches_address_body(address_body: &str, prefix: &str, suffix: &str) -> bool {
    (prefix.is_empty() || address_body.starts_with(prefix))
        && (suffix.is_empty() || address_body.ends_with(suffix))
}
