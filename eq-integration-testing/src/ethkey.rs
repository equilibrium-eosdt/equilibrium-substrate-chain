//! API for generation random Ethereum keys, addresses and signatures.
//!
//! Features:
//! - Random Ethereum key pair and address generation
//! - Signing and verification
//!
//! Usage example:
//! ```rust
//!     let key = ethkey::EthKey::generate_random();
//!     println!(key.address());
//!     let msg = "Message";
//!     let signature = key.sign(msg);
//!     println!(hex::encode(signature));
//!     assert!(key.verify(&signature, msg));
//! ```

use eq_claim::EcdsaSignature;
use secp256k1::{self, util, Message, PublicKey, SecretKey};
use sp_core::crypto::Pair as TraitPair;
use sp_core::ecdsa::{Pair, Signature};
use sp_io::hashing::keccak_256;

/// Represents Ethereum keys and provides API for address and signing.
pub struct EthKey {
    pair: Pair,
}

impl EthKey {
    /// Constructs from string seed.
    pub fn from_string(seed: &str) -> Self {
        EthKey {
            pair: create_pair(seed),
        }
    }

    /// Constructs from random seed.
    pub fn generate_random() -> Self {
        EthKey {
            pair: generate_random_pair(),
        }
    }

    /// Returns address corresponding the key.
    pub fn address(&self) -> String {
        derive_address(&self.pair)
    }

    /// Signs message.
    pub fn sign(&self, msg: &str) -> EcdsaSignature {
        sign_text(&self.pair, msg)
    }

    /// Verifies message which has been signed with signature.
    pub fn verify(&self, signature: &EcdsaSignature, msg: &str) -> bool {
        verify_signature(&self.pair, signature, msg)
    }
}

const ETHEREUM_RECOVERY_ID_BASE: u8 = 0x1b;

/// Creates pair of keys from seed.
fn create_pair(seed: &str) -> Pair {
    Pair::from_string(seed, None).expect("pair of keys")
}

/// Generates random pair of keys.
fn generate_random_pair() -> Pair {
    let s = generate_random_string();
    create_pair(&ensure_canonical_seed(&s))
}

/// Derives address from pair of keys.
/// The address is last 20 bytes (from 32) of hash of uncompressed public key.
fn derive_address(pair: &Pair) -> String {
    // Pair::to_raw_vec() returns bytes of secret
    let secret_key = SecretKey::parse_slice(&pair.to_raw_vec()[..]).expect("secret key");
    let public_key = PublicKey::from_secret_key(&secret_key);
    let uncompressed = public_key.serialize();
    assert_eq!(uncompressed.len(), util::FULL_PUBLIC_KEY_SIZE);
    assert_eq!(uncompressed[0], 0x04); // drop this prefix byte
    let addr = &keccak_256(&uncompressed[1..])[12..];
    format!("0x{}", hex::encode(addr))
}

/// Signs text and returns the signature.
/// Note: it's important to hash with keccak_256 function.
/// ecdsa::Pair::sign uses blake2_256 which inappropriate for Ethereum.
fn sign_text(pair: &Pair, msg: &str) -> EcdsaSignature {
    // Pair::to_raw_vec() returns bytes of secret
    let secret = SecretKey::parse_slice(&pair.to_raw_vec()[..]).expect("secret key");
    let msg = Message::parse(&keccak_256(msg.as_bytes()));
    let mut signature: Signature = secp256k1::sign(&msg, &secret).into();
    signature.as_mut()[util::SIGNATURE_SIZE] += ETHEREUM_RECOVERY_ID_BASE;
    EcdsaSignature(*signature.as_ref())
}

/// Verifies a signature on a message. Returns true if the signature is good.
/// Note: it's important to hash with keccak_256 function.
/// ecdsa::Pair::verify uses blake2_256 which inappropriate for Ethereum.
fn verify_signature(pair: &Pair, signature: &EcdsaSignature, msg: &str) -> bool {
    let arr: &[u8; 65] = signature.as_ref();
    let s =
        secp256k1::Signature::parse_slice(&arr[..util::SIGNATURE_SIZE]).expect("valid signature");
    let r = secp256k1::RecoveryId::parse(arr[util::SIGNATURE_SIZE] - ETHEREUM_RECOVERY_ID_BASE)
        .expect("valid recovery id");
    let msg = Message::parse(&keccak_256(msg.as_bytes()));
    match secp256k1::recover(&msg, &s, &r) {
        Ok(actual) => pair.public().as_ref() == &actual.serialize_compressed()[..],
        _ => false,
    }
}

/// Generates random string.
fn generate_random_string() -> String {
    use rand::{distributions::Alphanumeric, Rng};
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(10)
        .collect()
}

/// Prepends "//" to the string if absent.
fn ensure_canonical_seed(s: &str) -> String {
    if !s.starts_with("//") {
        return format!("//{}", s);
    }
    String::from(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canonical_seed() {
        assert_eq!(ensure_canonical_seed(""), "//");
        assert_eq!(ensure_canonical_seed("aaa"), "//aaa");
        assert_eq!(ensure_canonical_seed("//bbb"), "//bbb");
        assert_eq!(ensure_canonical_seed("/ccc"), "///ccc");
    }

    #[test]
    fn test_random_string() {
        let s = generate_random_string();
        assert_eq!(s.len(), 10);
        assert!(s.chars().all(char::is_alphanumeric));
    }

    #[test]
    fn test_random_key() {
        let key = EthKey::generate_random(); // may panic
        let address = key.address(); // hex string of 20-byte array with '0x' prefix
        assert_eq!(address.len(), 42);
        assert!(address.chars().all(char::is_alphanumeric));
        assert!(address.chars().skip(2).all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    #[should_panic]
    fn test_incorrect_seed() {
        let key = EthKey::from_string("symbol arm limb"); // should panic
        assert_eq!(key.address(), "0x");
    }

    #[test]
    fn test_address() {
        let secret = "0x17d08f5fe8c77af811caa0c9a187e668ce3b74a99acc3f6d976f075fa8e0be55";
        let key = EthKey::from_string(secret); // may panic
        assert_eq!(key.address(), "0x26d1ec50b4e62c1d1a40d16e7cacc6a6580757d5");

        let secret = "0x79afbf7147841fca72b45a1978dd7669470ba67abbe5c220062924380c9c364b";
        let key = EthKey::from_string(secret); // may panic
        assert_eq!(key.address(), "0xddab73bff78dae0a0a00e424703598454b96ff17");

        let key = EthKey::from_string("//bank"); // may panic
        assert_eq!(key.address(), "0x8c4e717cb7ba84f675016a6f7961de651aebf62b");

        let seed_phrase =
            "wonder visa weasel winner lemon daughter hen capable flee theory recycle enjoy";
        let key = EthKey::from_string(seed_phrase); // may panic
        assert_eq!(key.address(), "0xf78f73755c8d14e9cf09d1c723ed793be6dc13f7");
    }

    #[test]
    fn test_signature() {
        let secret = "0x17d08f5fe8c77af811caa0c9a187e668ce3b74a99acc3f6d976f075fa8e0be55";
        let key = EthKey::from_string(secret); // may panic
        let msg = "\u{19}Ethereum Signed Message:\n86Pay EQ to the account:8a1616edd811b144840b82eed04d31eed1bc6db7b71f8cdd27cefc2124c60d08";
        let signature = key.sign(msg);
        assert_eq!(hex::encode(signature),
                   "35b8870f1b08f4400b8a6ecdc24442eed678165c9413cc0b863a4b9891fe9ca52770d1d8c73a2b2962c2dd4eb0e918a0a50f01f53a8292c55d8b5c512db7cbdd1b");

        let secret = "0x79afbf7147841fca72b45a1978dd7669470ba67abbe5c220062924380c9c364b";
        let key = EthKey::from_string(secret); // may panic
        let msg = "Message for ECDSA signing";
        let signature = key.sign(msg);
        assert_eq!(hex::encode(signature),
                   "27736efb8299f3372cc0fc1410b5e930525aa6637a99dd042b455cec484b350e6678f4c077bbc5df8a34a55548a443bbb7f747d3808b9cc83d0c7877d7a2b29f1c");

        let key = EthKey::from_string("//bank"); // may panic
        let msg = "Message";
        let signature = key.sign(msg);
        assert_eq!(hex::encode(signature),
                   "cb9efcd29154b93544aecfa5d82c9ebf94f11ca36da8717940c6dff0148b06ab4dd064e7ee3f25a702d74cd64fb1a73c55ec4dbb962e1662599ead445e410e8b1c");
    }

    #[test]
    fn test_verify() {
        let key = EthKey::from_string("//bank"); // may panic
        let msg = "Message";
        let signature = key.sign(msg);
        assert!(key.verify(&signature, msg));

        let secret = "0x79afbf7147841fca72b45a1978dd7669470ba67abbe5c220062924380c9c364b";
        let key = EthKey::from_string(secret); // may panic
        let msg = "Message for ECDSA signing";
        let signature = key.sign(msg);
        assert!(key.verify(&signature, msg));

        let secret = "0x1efac25af294f4a5e23d0f0c9372dbc485358d9991c7111fe9e050c3458e324b";
        let key = EthKey::from_string(secret); // may panic
        let msg = "\u{19}Ethereum Signed Message:\n86Pay EQ to the account:8a1616edd811b144840b82eed04d31eed1bc6db7b71f8cdd27cefc2124c60d08";
        let signature = key.sign(msg);
        assert!(key.verify(&signature, msg));
    }
}
