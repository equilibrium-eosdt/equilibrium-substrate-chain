use std::cmp::Eq;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

pub trait PubKey {
    type AccountId;

    fn acc_id(&self) -> Self::AccountId;
}

pub trait KeyPair: PubKey {
    fn key_pair(&self) -> sp_core::sr25519::Pair;
}

pub struct Nonces<K> {
    nonces: HashMap<K, u32>,
}

impl<K> Nonces<K> {
    pub fn new() -> Nonces<K> {
        Nonces {
            nonces: HashMap::<K, u32>::new(),
        }
    }
}

// pub type DevNonces = Nonces<AccountKey>;

pub trait NonceManager {
    type AccountKey: Copy + Debug;

    fn is_initialized(&self, key: Self::AccountKey) -> bool;
    fn init_nonce(&mut self, key: Self::AccountKey, nonce: u32);
    fn get_nonce_and_inc(&mut self, key: Self::AccountKey) -> u32;
}

impl<AccountKey: Copy + Debug + Eq + Hash> NonceManager for Nonces<AccountKey> {
    type AccountKey = AccountKey;

    fn is_initialized(&self, key: Self::AccountKey) -> bool {
        self.nonces.get(&key).is_some()
    }

    fn init_nonce(&mut self, key: Self::AccountKey, nonce: u32) {
        if self.nonces.insert(key, nonce).is_some() {
            panic!("nonce for key {:?} has already been initialized", key);
        }
    }

    fn get_nonce_and_inc(&mut self, key: Self::AccountKey) -> u32 {
        let value = self.nonces.get_mut(&key);

        if let Some(value) = value {
            let curr_nonce = *value;
            *value = *value + 1;
            curr_nonce
        } else {
            panic!("nonce for key {:?} is not initialized", key);
        }
    }
}
