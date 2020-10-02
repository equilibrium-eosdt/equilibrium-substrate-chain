#![cfg_attr(not(feature = "std"), no_std)]
//! Module to process claims from Ethereum addresses.

mod benchmarking;
mod benchmarks;
mod mock;
mod secp_utils;

use codec::{Decode, Encode};
use eq_primitives::AccountGetter;
use eq_utils::log::eq_log;
#[allow(unused_imports)]
use frame_support::debug;
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage,
    dispatch::IsSubType,
    ensure,
    traits::{Currency, EnsureOrigin, Get, VestingSchedule},
    weights::{DispatchClass, Pays, Weight},
};
use frame_system::{ensure_none, ensure_root, ensure_signed};
#[cfg(feature = "std")]
use serde::{self, Deserialize, Deserializer, Serialize, Serializer};
use sp_io::{crypto::secp256k1_ecdsa_recover, hashing::keccak_256};
#[cfg(feature = "std")]
use sp_runtime::traits::Zero;
use sp_runtime::{
    traits::{CheckedSub, DispatchInfoOf, Saturating, SignedExtension},
    transaction_validity::{
        InvalidTransaction, TransactionLongevity, TransactionSource, TransactionValidity,
        TransactionValidityError, ValidTransaction,
    },
    DispatchResult, RuntimeDebug,
};
use sp_std::{fmt::Debug, prelude::*};

pub trait WeightInfo {
    fn claim(u: u32) -> Weight;
    fn mint_claim(c: u32) -> Weight;
    fn claim_attest(u: u32) -> Weight;
    fn attest(u: u32) -> Weight;
    fn validate_unsigned_claim(c: u32) -> Weight;
    fn validate_unsigned_claim_attest(c: u32) -> Weight;
    fn validate_prevalidate_attests(c: u32) -> Weight;
    fn keccak256(i: u32) -> Weight;
    fn eth_recover(i: u32) -> Weight;
}

type CurrencyOf<T> = <<T as Trait>::VestingSchedule as VestingSchedule<
    <T as frame_system::Trait>::AccountId,
>>::Currency;
type BalanceOf<T> = <CurrencyOf<T> as Currency<<T as frame_system::Trait>::AccountId>>::Balance;

#[repr(u8)]
pub enum ValidityError {
    /// The Ethereum signature is invalid.
    InvalidEthereumSignature = 0,
    /// The signer has no claim.
    SignerHasNoClaim = 1,
    /// No permission to execute the call.
    NoPermission = 2,
    /// An invalid statement was made for a claim.
    InvalidStatement = 3,
}

impl From<ValidityError> for u8 {
    fn from(err: ValidityError) -> Self {
        err as u8
    }
}

/// Configuration trait.
pub trait Trait: frame_system::Trait {
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
    type VestingSchedule: VestingSchedule<Self::AccountId, Moment = Self::BlockNumber>;
    type Prefix: Get<&'static [u8]>;
    type MoveClaimOrigin: EnsureOrigin<Self::Origin>;
    type VestingAccountGetter: AccountGetter<Self::AccountId>;
    type WeightInfo: WeightInfo;
}

/// An Ethereum address (i.e. 20 bytes, used to represent an Ethereum account).
///
/// This gets serialized to the 0x-prefixed hex representation.
#[derive(
    Clone, Copy, PartialEq, PartialOrd, Ord, Eq, Encode, Decode, Default, RuntimeDebug, Hash,
)]
pub struct EthereumAddress([u8; 20]);

#[cfg(feature = "std")]
impl Serialize for EthereumAddress {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let hex: String = rustc_hex::ToHex::to_hex(&self.0[..]);
        serializer.serialize_str(&format!("0x{}", hex))
    }
}

#[cfg(feature = "std")]
impl<'de> Deserialize<'de> for EthereumAddress {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let base_string = String::deserialize(deserializer)?;
        let offset = if base_string.starts_with("0x") { 2 } else { 0 };
        let s = &base_string[offset..];
        if s.len() != 40 {
            Err(serde::de::Error::custom(
                "Bad length of Ethereum address (should be 42 including '0x')",
            ))?;
        }
        let raw: Vec<u8> = rustc_hex::FromHex::from_hex(s)
            .map_err(|e| serde::de::Error::custom(format!("{:?}", e)))?;
        let mut r = Self::default();
        r.0.copy_from_slice(&raw);
        Ok(r)
    }
}

#[derive(Encode, Decode, Clone)]
pub struct EcdsaSignature(pub [u8; 65]);

impl AsRef<[u8; 65]> for EcdsaSignature {
    fn as_ref(&self) -> &[u8; 65] {
        &self.0
    }
}

impl AsRef<[u8]> for EcdsaSignature {
    fn as_ref(&self) -> &[u8] {
        &self.0[..]
    }
}

impl PartialEq for EcdsaSignature {
    fn eq(&self, other: &Self) -> bool {
        &self.0[..] == &other.0[..]
    }
}

impl sp_std::fmt::Debug for EcdsaSignature {
    fn fmt(&self, f: &mut sp_std::fmt::Formatter<'_>) -> sp_std::fmt::Result {
        write!(f, "EcdsaSignature({:?})", &self.0[..])
    }
}

decl_event!(
    pub enum Event<T>
    where
        Balance = BalanceOf<T>,
        AccountId = <T as frame_system::Trait>::AccountId,
    {
        /// Someone claimed some tokens.
        Claimed(AccountId, EthereumAddress, Balance),
    }
);

decl_error! {
    pub enum Error for Module<T: Trait> {
        /// Invalid Ethereum signature.
        InvalidEthereumSignature,
        /// Ethereum address has no claim.
        SignerHasNoClaim,
        /// Account ID sending tx has no claim.
        SenderHasNoClaim,
        /// There's not enough in the pot to pay out some unvested amount. Generally implies a logic
        /// error.
        PotUnderflow,
        /// A needed statement was not included.
        InvalidStatement,
        /// The account already has a vested balance.
        VestedBalanceExists,
    }
}

decl_storage! {
    // A macro for the Storage trait, and its implementation, for this module.
    // This allows for type-safe usage of the Substrate storage database, so you can
    // keep things around between blocks.
    trait Store for Module<T: Trait> as Claims {
        Claims get(fn claims) build(|config: &GenesisConfig<T>| {
            config.claims.iter().map(|(a, b, _, _)| (a.clone(), b.clone())).collect::<Vec<_>>()
        }): map hasher(identity) EthereumAddress => Option<BalanceOf<T>>;
        Total get(fn total) build(|config: &GenesisConfig<T>| {
            config.claims.iter().fold(Zero::zero(), |acc: BalanceOf<T>, &(_, b, _, _)| acc + b)
        }): BalanceOf<T>;
        /// Vesting schedule for a claim.
        /// First balance is the total amount that should be held for vesting.
        /// Second balance is how much should be unlocked per block.
        /// The block number is when the vesting should start.
        Vesting get(fn vesting) config():
            map hasher(identity) EthereumAddress
            => Option<(BalanceOf<T>, BalanceOf<T>, T::BlockNumber)>;

        /// The statement kind that must be signed, if any.
        Signing build(|config: &GenesisConfig<T>| {
            config.claims.iter()
                .filter_map(|(a, _, _, s)| Some((a.clone(), s.clone())))
                .collect::<Vec<_>>()
        }): map hasher(identity) EthereumAddress => bool;

        /// Pre-claimed Ethereum accounts, by the Account ID that they are claimed to.
        Preclaims build(|config: &GenesisConfig<T>| {
            config.claims.iter()
                .filter_map(|(a, _, i, _)| Some((i.clone()?, a.clone())))
                .collect::<Vec<_>>()
        }): map hasher(identity) T::AccountId => Option<EthereumAddress>;
    }
    add_extra_genesis {
        config(claims): Vec<(EthereumAddress, BalanceOf<T>, Option<T::AccountId>, bool)>;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        /// The Prefix that is used in signed Ethereum messages for this network
        const Prefix: &[u8] = T::Prefix::get();

        /// Deposit one of this module's events by using the default implementation.
        fn deposit_event() = default;

        /// Make a claim to collect your tokens.
        ///
        /// The dispatch origin for this call must be _None_.
        ///
        /// Unsigned Validation:
        /// A call to claim is deemed valid if the signature provided matches
        /// the expected signed message of:
        ///
        /// > Ethereum Signed Message:
        /// > (configured prefix string)(address)
        ///
        /// and `address` matches the `dest` account.
        ///
        /// Parameters:
        /// - `dest`: The destination account to payout the claim.
        /// - `ethereum_signature`: The signature of an ethereum signed message
        ///    matching the format described above.
        ///
        /// <weight>
        /// The weight of this call is invariant over the input parameters.
        /// - One `eth_recover` operation which involves a keccak hash and a
        ///   ecdsa recover.
        /// - Three storage reads to check if a claim exists for the user, to
        ///   get the current pot size, to see if there exists a vesting schedule.
        /// - Up to one storage write for adding a new vesting schedule.
        /// - One `deposit_creating` Currency call.
        /// - One storage write to update the total.
        /// - Two storage removals for vesting and claims information.
        /// - One deposit event.
        ///
        /// Total Complexity: O(1)
        /// ----------------------------
        /// Base Weight: 269.7 µs
        /// DB Weight:
        /// - Read: Signing, Claims, Total, Claims Vesting, Vesting Vesting, Balance Lock, Account
        /// - Write: Vesting Vesting, Account, Balance Lock, Total, Claim, Claims Vesting, Signing
        /// Validate Unsigned: +188.7 µs
        /// </weight>
        #[weight = T::WeightInfo::claim(1)]
        fn claim(origin, dest: T::AccountId, ethereum_signature: EcdsaSignature) {
            ensure_none(origin)?;

            let data = dest.using_encoded(to_ascii_hex);
            let signer = Self::eth_recover(&ethereum_signature, &data, &[][..])
                .ok_or(Error::<T>::InvalidEthereumSignature)?;
            ensure!(Signing::get(&signer) == false, Error::<T>::InvalidStatement);

            Self::process_claim(signer, dest)?;
        }

        /// Mint a new claim to collect tokens.
        ///
        /// The dispatch origin for this call must be _Root_.
        ///
        /// Parameters:
        /// - `who`: The Ethereum address allowed to collect this claim.
        /// - `value`: The number of tokens that will be claimed.
        /// - `vesting_schedule`: An optional vesting schedule for these tokens.
        ///
        /// <weight>
        /// The weight of this call is invariant over the input parameters.
        /// - One storage mutate to increase the total claims available.
        /// - One storage write to add a new claim.
        /// - Up to one storage write to add a new vesting schedule.
        ///
        /// Total Complexity: O(1)
        /// ---------------------
        /// Base Weight: 10.46 µs
        /// DB Weight:
        /// - Reads: Total
        /// - Writes: Total, Claims
        /// - Maybe Write: Vesting, Statement
        /// </weight>
        #[weight = T::WeightInfo::mint_claim(5_000)]
        fn mint_claim(origin,
            who: EthereumAddress,
            value: BalanceOf<T>,
            vesting_schedule: Option<(BalanceOf<T>, BalanceOf<T>, T::BlockNumber)>,
            statement: bool,
        ) {
            ensure_root(origin)?;

            if vesting_schedule != None && value < vesting_schedule.unwrap().0 {
                eq_log!(
                    "mint_claim error: value {:?} < vesting_schedule.locked {:?}",
                    value,
                    vesting_schedule.unwrap().0
                );
                ensure!(false, Error::<T>::InvalidStatement);
            }

            <Total<T>>::mutate(|t| *t += value);
            <Claims<T>>::insert(who, value);
            if let Some(vs) = vesting_schedule {
                <Vesting<T>>::insert(who, vs);
            }
            if statement {
                Signing::insert(who, statement);
            }
        }

        /// Make a claim to collect your tokens by signing a statement.
        ///
        /// The dispatch origin for this call must be _None_.
        ///
        /// Unsigned Validation:
        /// A call to `claim_attest` is deemed valid if the signature provided matches
        /// the expected signed message of:
        ///
        /// > Ethereum Signed Message:
        /// > (configured prefix string)(address)(statement)
        ///
        /// and `address` matches the `dest` account; the `statement` must match that which is
        /// expected according to your purchase arrangement.
        ///
        /// Parameters:
        /// - `dest`: The destination account to payout the claim.
        /// - `ethereum_signature`: The signature of an ethereum signed message
        ///    matching the format described above.
        /// - `statement`: The identity of the statement which is being attested to in the signature.
        ///
        /// <weight>
        /// The weight of this call is invariant over the input parameters.
        /// - One `eth_recover` operation which involves a keccak hash and a
        ///   ecdsa recover.
        /// - Four storage reads to check if a claim exists for the user, to
        ///   get the current pot size, to see if there exists a vesting schedule, to get the
        ///   required statement.
        /// - Up to one storage write for adding a new vesting schedule.
        /// - One `deposit_creating` Currency call.
        /// - One storage write to update the total.
        /// - Two storage removals for vesting and claims information.
        /// - One deposit event.
        ///
        /// Total Complexity: O(1)
        /// ----------------------------
        /// Base Weight: 270.2 µs
        /// DB Weight:
        /// - Read: Signing, Claims, Total, Claims Vesting, Vesting Vesting, Balance Lock, Account
        /// - Write: Vesting Vesting, Account, Balance Lock, Total, Claim, Claims Vesting, Signing
        /// Validate Unsigned: +190.1 µs
        /// </weight>
        #[weight = T::WeightInfo::claim_attest(1)]
        fn claim_attest(origin,
            dest: T::AccountId,
            ethereum_signature: EcdsaSignature,
            statement: Vec<u8>,
        ) {
            ensure_none(origin)?;

            let data = dest.using_encoded(to_ascii_hex);
            let signer = Self::eth_recover(&ethereum_signature, &data, &statement)
                .ok_or(Error::<T>::InvalidEthereumSignature)?;
            let s = Signing::get(signer);
            if s {
                ensure!(get_statement_text() == &statement[..], Error::<T>::InvalidStatement);
            }
            Self::process_claim(signer, dest)?;
        }

        /// Attest to a statement, needed to finalize the claims process.
        ///
        /// WARNING: Insecure unless your chain includes `PrevalidateAttests` as a `SignedExtension`.
        ///
        /// Unsigned Validation:
        /// A call to attest is deemed valid if the sender has a `Preclaim` registered
        /// and provides a `statement` which is expected for the account.
        ///
        /// Parameters:
        /// - `statement`: The identity of the statement which is being attested to in the signature.
        ///
        /// <weight>
        /// Total Complexity: O(1)
        /// ----------------------------
        /// Base Weight: 93.3 µs
        /// DB Weight:
        /// - Read: Preclaims, Signing, Claims, Total, Claims Vesting, Vesting Vesting, Balance Lock, Account
        /// - Write: Vesting Vesting, Account, Balance Lock, Total, Claim, Claims Vesting, Signing, Preclaims
        /// Validate PreValidateAttests: +8.631 µs
        /// </weight>
        #[weight = (T::WeightInfo::attest(1), DispatchClass::Normal, Pays::No)]
        fn attest(origin, statement: Vec<u8>) {
            let who = ensure_signed(origin)?;
            let signer = Preclaims::<T>::get(&who).ok_or(Error::<T>::SenderHasNoClaim)?;
            let s = Signing::get(signer);
            if s {
                ensure!(get_statement_text() == &statement[..], Error::<T>::InvalidStatement);
            }
            Self::process_claim(signer, who.clone())?;
            Preclaims::<T>::remove(&who);
        }

        #[weight = (
            T::DbWeight::get().reads_writes(4, 4) + 100_000_000_000,
            DispatchClass::Normal,
            Pays::No
        )]
        fn move_claim(origin,
            old: EthereumAddress,
            new: EthereumAddress,
            maybe_preclaim: Option<T::AccountId>,
        ) {
            T::MoveClaimOrigin::try_origin(origin).map(|_| ()).or_else(ensure_root)?;

            Claims::<T>::take(&old).map(|c| Claims::<T>::insert(&new, c));
            Vesting::<T>::take(&old).map(|c| Vesting::<T>::insert(&new, c));
            let s = Signing::take(&old);
            Signing::insert(&new, s);
            maybe_preclaim.map(|preclaim| Preclaims::<T>::mutate(&preclaim, |maybe_o|
                if maybe_o.as_ref().map_or(false, |o| o == &old) { *maybe_o = Some(new) }
            ));
        }
    }
}

/// Convert this to the (English) statement it represents.
pub fn get_statement_text() -> &'static [u8] {
    &b"I hereby agree to the terms of the statement whose SHA-256 multihash is \
            Qmc1XYqT6S39WNp2UeiRUrZichUWUPpGEThDE6dAb3f6Ny. (This may be found at the URL: \
            https://equilibrium.io/tokenswap/docs/token_swap_t&cs.pdf)"[..]
}

/// Converts the given binary data into ASCII-encoded hex. It will be twice the length.
pub fn to_ascii_hex(data: &[u8]) -> Vec<u8> {
    let mut r = Vec::with_capacity(data.len() * 2);
    let mut push_nibble = |n| r.push(if n < 10 { b'0' + n } else { b'a' - 10 + n });
    for &b in data.iter() {
        push_nibble(b / 16);
        push_nibble(b % 16);
    }
    r
}

impl<T: Trait> Module<T> {
    // Constructs the message that Ethereum RPC's `personal_sign` and `eth_sign` would sign.
    fn ethereum_signable_message(what: &[u8], extra: &[u8]) -> Vec<u8> {
        let prefix = T::Prefix::get();
        let mut l = prefix.len() + what.len() + extra.len();
        let mut rev = Vec::new();
        while l > 0 {
            rev.push(b'0' + (l % 10) as u8);
            l /= 10;
        }
        let mut v = b"\x19Ethereum Signed Message:\n".to_vec();
        v.extend(rev.into_iter().rev());
        v.extend_from_slice(&prefix[..]);
        v.extend_from_slice(what);
        v.extend_from_slice(extra);
        v
    }

    // Attempts to recover the Ethereum address from a message signature signed by using
    // the Ethereum RPC's `personal_sign` and `eth_sign`.
    fn eth_recover(s: &EcdsaSignature, what: &[u8], extra: &[u8]) -> Option<EthereumAddress> {
        let msg = keccak_256(&Self::ethereum_signable_message(what, extra));
        let mut res = EthereumAddress::default();
        res.0
            .copy_from_slice(&keccak_256(&secp256k1_ecdsa_recover(&s.0, &msg).ok()?[..])[12..]);
        Some(res)
    }

    fn process_claim(signer: EthereumAddress, dest: T::AccountId) -> DispatchResult {
        let balance_due = <Claims<T>>::get(&signer).ok_or(Error::<T>::SignerHasNoClaim)?;

        let new_total = Self::total()
            .checked_sub(&balance_due)
            .ok_or(Error::<T>::PotUnderflow)?;

        let vesting = Vesting::<T>::get(&signer);
        if vesting.is_some() && T::VestingSchedule::vesting_balance(&dest).is_some() {
            return Err(Error::<T>::VestedBalanceExists.into());
        }

        // Check if this claim should have a vesting schedule.
        if let Some(vs) = vesting {
            let initial_balance = balance_due.saturating_sub(vs.0);
            CurrencyOf::<T>::deposit_creating(&dest, initial_balance);
            let vesting_account_id = T::VestingAccountGetter::get_account_id();

            #[allow(unused_must_use)]
            {
                CurrencyOf::<T>::deposit_into_existing(&vesting_account_id, vs.0);
            }

            // This can only fail if the account already has a vesting schedule,
            // but this is checked above.
            T::VestingSchedule::add_vesting_schedule(&dest, vs.0, vs.1, vs.2)
                .expect("No other vesting schedule exists, as checked above; qed");
        } else {
            CurrencyOf::<T>::deposit_creating(&dest, balance_due);
        }

        <Total<T>>::put(new_total);
        <Claims<T>>::remove(&signer);
        <Vesting<T>>::remove(&signer);
        Signing::remove(&signer);

        // Let's deposit an event to let the outside world know this happened.
        Self::deposit_event(RawEvent::Claimed(dest, signer, balance_due));

        Ok(())
    }
}

impl<T: Trait> sp_runtime::traits::ValidateUnsigned for Module<T> {
    type Call = Call<T>;

    fn validate_unsigned(_source: TransactionSource, call: &Self::Call) -> TransactionValidity {
        const PRIORITY: u64 = 100;

        let (maybe_signer, maybe_statement) = match call {
            // <weight>
            // Base Weight: 188.7 µs (includes the full logic of `validate_unsigned`)
            // DB Weight: 2 Read (Claims, Signing)
            // </weight>
            Call::claim(account, ethereum_signature) => {
                let data = account.using_encoded(to_ascii_hex);
                (Self::eth_recover(&ethereum_signature, &data, &[][..]), None)
            }
            // <weight>
            // Base Weight: 190.1 µs (includes the full logic of `validate_unsigned`)
            // DB Weight: 2 Read (Claims, Signing)
            // </weight>
            Call::claim_attest(account, ethereum_signature, statement) => {
                let data = account.using_encoded(to_ascii_hex);
                (
                    Self::eth_recover(&ethereum_signature, &data, &statement),
                    Some(statement.as_slice()),
                )
            }
            _ => return Err(InvalidTransaction::Call.into()),
        };

        let signer = maybe_signer.ok_or(InvalidTransaction::Custom(
            ValidityError::InvalidEthereumSignature.into(),
        ))?;

        let e = InvalidTransaction::Custom(ValidityError::SignerHasNoClaim.into());
        ensure!(<Claims<T>>::contains_key(&signer), e);

        let e = InvalidTransaction::Custom(ValidityError::InvalidStatement.into());
        let s = Signing::get(signer);
        if s {
            ensure!(Some(get_statement_text()) == maybe_statement, e)
        } else {
            ensure!(maybe_statement.is_none(), e)
        }

        Ok(ValidTransaction {
            priority: PRIORITY,
            requires: vec![],
            provides: vec![("claims", signer).encode()],
            longevity: TransactionLongevity::max_value(),
            propagate: true,
        })
    }
}

/// Validate `attest` calls prior to execution. Needed to avoid a DoS attack since they are
/// otherwise free to place on chain.
#[derive(Encode, Decode, Clone, Eq, PartialEq)]
pub struct PrevalidateAttests<T: Trait + Send + Sync>(sp_std::marker::PhantomData<T>)
where
    <T as frame_system::Trait>::Call: IsSubType<Call<T>>;

impl<T: Trait + Send + Sync> Debug for PrevalidateAttests<T>
where
    <T as frame_system::Trait>::Call: IsSubType<Call<T>>,
{
    #[cfg(feature = "std")]
    fn fmt(&self, f: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
        write!(f, "PrevalidateAttests")
    }

    #[cfg(not(feature = "std"))]
    fn fmt(&self, _: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
        Ok(())
    }
}

impl<T: Trait + Send + Sync> PrevalidateAttests<T>
where
    <T as frame_system::Trait>::Call: IsSubType<Call<T>>,
{
    /// Create new `SignedExtension` to check runtime version.
    pub fn new() -> Self {
        Self(sp_std::marker::PhantomData)
    }
}

impl<T: Trait + Send + Sync> SignedExtension for PrevalidateAttests<T>
where
    <T as frame_system::Trait>::Call: IsSubType<Call<T>>,
{
    type AccountId = T::AccountId;
    type Call = <T as frame_system::Trait>::Call;
    type AdditionalSigned = ();
    type Pre = ();
    const IDENTIFIER: &'static str = "PrevalidateAttests";

    fn additional_signed(&self) -> Result<Self::AdditionalSigned, TransactionValidityError> {
        Ok(())
    }

    // <weight>
    // Base Weight: 8.631 µs
    // DB Weight: 2 Read (Preclaims, Signing)
    // </weight>
    fn validate(
        &self,
        who: &Self::AccountId,
        call: &Self::Call,
        _info: &DispatchInfoOf<Self::Call>,
        _len: usize,
    ) -> TransactionValidity {
        if let Some(local_call) = call.is_sub_type() {
            if let Call::attest(attested_statement) = local_call {
                let signer = Preclaims::<T>::get(who).ok_or(InvalidTransaction::Custom(
                    ValidityError::SignerHasNoClaim.into(),
                ))?;
                let s = Signing::get(signer);
                if s {
                    let e = InvalidTransaction::Custom(ValidityError::InvalidStatement.into());
                    ensure!(&attested_statement[..] == get_statement_text(), e);
                }
            }
        }
        Ok(ValidTransaction::default())
    }
}
