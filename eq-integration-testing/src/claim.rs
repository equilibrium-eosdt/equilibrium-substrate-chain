use codec::{Codec, Encode};
use core::marker::PhantomData;
use frame_support::Parameter;
use sp_runtime::traits::{AtLeast32BitUnsigned, MaybeSerializeDeserialize, Member};
use std::fmt::Debug;
use substrate_subxt::system::{System, SystemEventsDecoder};
use substrate_subxt_proc_macro::{module, Call, Store};

#[module]
pub trait Claim: System {
    type Balance: Parameter
        + Member
        + AtLeast32BitUnsigned
        + Codec
        + Default
        + Copy
        + MaybeSerializeDeserialize
        + Debug
        + From<u64>
        + Into<u64>;
    type EthereumAddress: Default + Codec + Member;
}

#[derive(Clone, Debug, Eq, PartialEq, Store, Encode)]
pub struct ClaimsStore<T: Claim> {
    #[store(returns = Option<<T as Claim>::Balance>)]
    pub ethereum_address: eq_claim::EthereumAddress,
    pub _runtime: PhantomData<T>,
}

#[derive(Clone, Debug, Eq, PartialEq, Store, Encode)]
pub struct TotalStore<T: Claim> {
    #[store(returns = <T as Claim>::Balance)]
    pub _runtime: PhantomData<T>,
}

#[derive(Clone, Debug, Eq, PartialEq, Store, Encode)]
pub struct VestingStore<T: Claim> {
    #[store(returns = Option<(<T as Claim>::Balance, <T as Claim>::Balance, T::BlockNumber)>)]
    pub ethereum_address: eq_claim::EthereumAddress,
    pub _runtime: PhantomData<T>,
}

#[derive(Clone, Debug, Eq, PartialEq, Store, Encode)]
pub struct SigningStore<T: Claim> {
    #[store(returns = bool)]
    pub ethereum_address: eq_claim::EthereumAddress,
    pub _runtime: PhantomData<T>,
}

#[derive(Clone, Debug, Eq, PartialEq, Store, Encode)]
pub struct PreclaimsStore<T: Claim> {
    #[store(returns = Option<eq_claim::EthereumAddress>)]
    pub account_id: T::AccountId,
}

#[derive(Clone, Debug, PartialEq, Call, Encode)]
pub struct ClaimCall<T: Claim> {
    pub dest: T::AccountId,
    pub ethereum_signature: eq_claim::EcdsaSignature,
}

#[derive(Clone, Debug, PartialEq, Call, Encode)]
pub struct MintClaimCall<T: Claim> {
    pub who: eq_claim::EthereumAddress,
    pub value: <T as Claim>::Balance,
    pub vesting_schedule: Option<(<T as Claim>::Balance, <T as Claim>::Balance, T::BlockNumber)>,
    pub statement: bool,
}

#[derive(Clone, Debug, PartialEq, Call, Encode)]
pub struct ClaimAttestCall<T: Claim> {
    pub dest: T::AccountId,
    pub ethereum_signature: eq_claim::EcdsaSignature,
    pub statement: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Call, Encode)]
pub struct AttestCall<T: Claim> {
    pub statement: Vec<u8>,
    pub _runtime: PhantomData<T>,
}

#[derive(Clone, Debug, PartialEq, Call, Encode)]
pub struct MoveClaimCall<T: Claim> {
    pub maybe_preclaim: Option<T::AccountId>,
}
