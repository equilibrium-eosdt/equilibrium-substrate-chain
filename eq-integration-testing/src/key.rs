use crate::keyring::{KeyPair, Nonces, PubKey};
use arraystring::{error, typenum::U10, ArrayString};
use serde::ser::{Serialize, SerializeStruct, SerializeTupleVariant, Serializer};
use sp_core::crypto::Pair as CryptoPair;
use sp_core::sr25519::Pair;
use sp_runtime::traits::AccountIdConversion;
use sp_runtime::{AccountId32, ModuleId};
use std::cmp::Ordering;
use std::collections::HashMap;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum AccountKey {
    Keyring(sp_keyring::sr25519::Keyring),
    Str(&'static str),
    Random(ArrayString10),
}

impl AccountKey {
    /// Constructs account key from random seed string.
    pub fn generate_random() -> Self {
        let seed = generate_random_array_string().expect("should be short random string");
        AccountKey::Random(seed)
    }

    fn to_num(&self) -> usize {
        match self {
            &AccountKey::Keyring(_) => 0,
            &AccountKey::Str(_) => 1,
            &AccountKey::Random(_) => 2,
        }
    }
}

/// Represents fixed-capacity string for testing AccountKey.
type ArrayString10 = ArrayString<U10>;

/// Generates random array string with fixed length, otherwise returns an error.
/// ```rust
/// let s = generate_random_array_string();
/// ```
fn generate_random_array_string() -> Result<ArrayString10, error::OutOfBounds> {
    use rand::{distributions::Alphanumeric, Rng};
    let s: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(10)
        .collect();
    ArrayString10::try_from_str(s)
}

fn keyring_to_string(keyring: sp_keyring::sr25519::Keyring) -> &'static str {
    keyring.into()
}

impl PartialOrd for AccountKey {
    fn partial_cmp(&self, other: &AccountKey) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Serialize for AccountKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            &AccountKey::Keyring(kr) => {
                let mut state =
                    serializer.serialize_tuple_variant("AccountKey", 0, "Keyring", 1)?;
                state.serialize_field(keyring_to_string(kr))?;
                state.end()
            }
            &AccountKey::Str(s) => {
                let mut state = serializer.serialize_tuple_variant("AccountKey", 1, "Str", 1)?;
                state.serialize_field(s)?;
                state.end()
            }
            &AccountKey::Random(s) => {
                let mut state = serializer.serialize_tuple_variant("AccountKey", 2, "Random", 1)?;
                state.serialize_field(s.as_str())?;
                state.end()
            }
        }
    }
}

impl Ord for AccountKey {
    fn cmp(&self, other: &AccountKey) -> Ordering {
        let num_ord = self.to_num().cmp(&other.to_num());

        if num_ord != Ordering::Equal {
            num_ord
        } else {
            match (self, other) {
                (&AccountKey::Keyring(a), &AccountKey::Keyring(b)) => {
                    keyring_to_string(a).cmp(keyring_to_string(b))
                }
                (&AccountKey::Str(a), &AccountKey::Str(b)) => a.cmp(b),
                (&AccountKey::Random(a), &AccountKey::Random(b)) => a.cmp(&b),
                _ => panic!("Not meant to be there"),
            }
        }
    }
}

impl From<sp_keyring::sr25519::Keyring> for AccountKey {
    fn from(x: sp_keyring::sr25519::Keyring) -> AccountKey {
        AccountKey::Keyring(x)
    }
}

impl From<&'static str> for AccountKey {
    fn from(x: &'static str) -> AccountKey {
        AccountKey::Str(x)
    }
}

fn string_to_static_str(s: &String) -> &'static str {
    Box::leak(s.clone().into_boxed_str())
}

impl From<&String> for AccountKey {
    fn from(x: &String) -> AccountKey {
        let s:&'static str = string_to_static_str(x);
        AccountKey::Str(s)
    }
}

impl PubKey for AccountKey {
    type AccountId = AccountId32;

    fn acc_id(&self) -> Self::AccountId {
        match self {
            &AccountKey::Keyring(k) => k.to_account_id(),
            &AccountKey::Str(s) => Pair::from_string(&ensure_canonical_seed(s), None)
                .unwrap()
                .public()
                .into(),
            &AccountKey::Random(s) => Pair::from_string(&ensure_canonical_seed(s.as_str()), None)
                .unwrap()
                .public()
                .into(),
        }
    }
}

impl KeyPair for AccountKey {
    fn key_pair(&self) -> sp_core::sr25519::Pair {
        match self {
            &AccountKey::Keyring(k) => k.pair(),
            &AccountKey::Str(s) => Pair::from_string(&ensure_canonical_seed(s), None).unwrap(),
            &AccountKey::Random(s) => {
                Pair::from_string(&ensure_canonical_seed(s.as_str()), None).unwrap()
            }
        }
    }
}

pub type DevNonces = Nonces<AccountKey>;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum DevPubKeyId {
    ModuleId([u8; 8]),
    AccKey(AccountKey),
    External(u32),
    WellKnown(&'static str),
}

impl DevPubKeyId {
    pub fn is_internal(&self) -> bool {
        match self {
            &DevPubKeyId::External(_) => false,
            _ => true,
        }
    }
}

impl serde::Serialize for DevPubKeyId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            &DevPubKeyId::ModuleId(bs) => {
                let mut state =
                    serializer.serialize_tuple_variant("DevPubKeyId", 0, "ModuleId", 1)?;
                state.serialize_field(&String::from_utf8_lossy(&bs))?;
                state.end()
            }
            &DevPubKeyId::AccKey(k) => {
                let mut state =
                    serializer.serialize_tuple_variant("DevPubKeyId", 1, "AccKey", 1)?;
                state.serialize_field(&k)?;
                state.end()
            }
            &DevPubKeyId::External(e) => {
                let mut state =
                    serializer.serialize_tuple_variant("DevPubKeyId", 2, "External", 1)?;
                state.serialize_field(&e)?;
                state.end()
            }
            &DevPubKeyId::WellKnown(s) => {
                let mut state =
                    serializer.serialize_tuple_variant("DevPubKeyId", 3, "WellKnown", 1)?;
                state.serialize_field(s)?;
                state.end()
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct DevPubKey {
    id: DevPubKeyId,
    acc_id: AccountId32,
}

impl Serialize for DevPubKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("DevPubKey", 2)?;
        state.serialize_field("id", &self.id)?;
        state.serialize_field("acc_id", &self.acc_id.to_string())?;
        state.end()
    }
}

impl DevPubKey {
    pub fn well_known(name: &'static str, acc_id: AccountId32) -> DevPubKey {
        DevPubKey {
            id: DevPubKeyId::WellKnown(name),
            acc_id,
        }
    }

    pub fn id(&self) -> DevPubKeyId {
        self.id
    }
}

impl PubKey for DevPubKey {
    type AccountId = AccountId32;

    fn acc_id(&self) -> Self::AccountId {
        self.acc_id.clone()
    }
}

impl From<ModuleId> for DevPubKey {
    fn from(x: ModuleId) -> DevPubKey {
        DevPubKey {
            id: DevPubKeyId::ModuleId(x.0),
            acc_id: x.into_account(),
        }
    }
}

impl From<AccountKey> for DevPubKey {
    fn from(x: AccountKey) -> DevPubKey {
        DevPubKey {
            id: DevPubKeyId::AccKey(x),
            acc_id: x.acc_id(),
        }
    }
}

#[derive(Eq, PartialEq)]
pub enum PubKeyStoreRegisterResult {
    AddedNew,
    ExistsNotModified,
    ExistsReplaced,
}

#[derive(Debug)]
pub struct PubKeyStore {
    items: HashMap<AccountId32, DevPubKeyId>,
    next_id: u32,
}

impl PubKeyStore {
    pub fn new() -> PubKeyStore {
        PubKeyStore {
            items: HashMap::new(),
            next_id: 0,
        }
    }

    pub fn register(&mut self, pub_key: DevPubKey) -> PubKeyStoreRegisterResult {
        let DevPubKey { id, acc_id } = pub_key;
        let item = self.items.get(&acc_id);

        if let Some(existing_id) = item {
            match (existing_id, id) {
                (_, DevPubKeyId::External(_)) => PubKeyStoreRegisterResult::ExistsNotModified,
                _ => PubKeyStoreRegisterResult::ExistsReplaced,
            }
        } else {
            self.items.insert(acc_id, id);
            PubKeyStoreRegisterResult::AddedNew
        }
    }

    pub fn register_external(&mut self, acc_id: AccountId32) {
        let id = DevPubKeyId::External(self.next_id);
        let result = self.register(DevPubKey { id, acc_id });

        if result == PubKeyStoreRegisterResult::AddedNew
            || result == PubKeyStoreRegisterResult::ExistsReplaced
        {
            self.next_id = self.next_id + 1;
        }
    }

    pub fn get_id(&self, acc_id: &AccountId32) -> Option<DevPubKeyId> {
        self.items.get(acc_id).map(|&x| x)
    }

    pub fn dump(&self) -> Vec<DevPubKey> {
        let mut items: Vec<_> = self
            .items
            .iter()
            .map(|(acc_id, &id)| DevPubKey {
                id,
                acc_id: acc_id.clone(),
            })
            .collect();

        items.sort_by(|a, b| a.id.cmp(&b.id));

        items
    }
}

/// Prepends "//" to the string if absent.
fn ensure_canonical_seed(s: &str) -> String {
    if !s.starts_with("//") && !s.starts_with("0x") {
        format!("//{}", s)
    } else {
        String::from(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canonical_seed() {
        assert_eq!(ensure_canonical_seed(""), "//");
        assert_eq!(ensure_canonical_seed("aa"), "//aa");
        assert_eq!(ensure_canonical_seed("//bbb"), "//bbb");
    }

    #[test]
    fn test_random_array_string() {
        let seed = generate_random_array_string();
        assert!(seed.is_ok());
        let seed = seed.unwrap();
        assert_eq!(seed.len(), 10);
        assert!(seed.chars().all(char::is_alphanumeric));
    }

    #[test]
    fn test_random_account_key() {
        let key = AccountKey::generate_random(); // might panic
        if let AccountKey::Random(s) = key {
            assert_eq!(s.len(), 10);
            assert!(s.chars().all(char::is_alphanumeric));
        } else {
            assert!(false);
        }
    }
}
