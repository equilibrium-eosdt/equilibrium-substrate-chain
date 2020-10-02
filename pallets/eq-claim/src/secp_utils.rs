#![cfg(any(test, feature = "runtime-benchmarks"))]
use crate::EcdsaSignature;
use crate::EthereumAddress;
use crate::*;
use secp256k1;
use sp_io::hashing::keccak_256;

pub fn public(secret: &secp256k1::SecretKey) -> secp256k1::PublicKey {
    secp256k1::PublicKey::from_secret_key(secret)
}
pub fn eth(secret: &secp256k1::SecretKey) -> EthereumAddress {
    let mut res = EthereumAddress::default();
    res.0
        .copy_from_slice(&keccak_256(&public(secret).serialize()[1..65])[12..]);
    res
}
pub fn sig<T: Trait>(secret: &secp256k1::SecretKey, what: &[u8], extra: &[u8]) -> EcdsaSignature {
    let msg = keccak_256(&<super::Module<T>>::ethereum_signable_message(
        &to_ascii_hex(what)[..],
        extra,
    ));
    let (sig, recovery_id) = secp256k1::sign(&secp256k1::Message::parse(&msg), secret);
    let mut r = [0u8; 65];
    r[0..64].copy_from_slice(&sig.serialize()[..]);
    r[64] = recovery_id.serialize();
    EcdsaSignature(r)
}
