use codec::{Codec, Encode};
use core::marker::PhantomData;
use frame_support::Parameter;
use sp_runtime::traits::{AtLeast32BitUnsigned, MaybeSerializeDeserialize, Member};
use std::fmt::Debug;
use substrate_subxt::system::{System, SystemEventsDecoder};
use substrate_subxt_proc_macro::{module, Call, Store};

#[module]
pub trait EqVesting: System {
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
}

#[derive(Clone, Debug, Eq, PartialEq, Store, Encode)]
pub struct VestingStore<T: EqVesting> {
    #[store(returns = Option<eq_vesting::VestingInfo<<T as EqVesting>::Balance, T::BlockNumber>>)]
    pub account_id: T::AccountId,
}

#[derive(Clone, Debug, Eq, PartialEq, Store, Encode)]
pub struct VestedStore<T: EqVesting> {
    #[store(returns = Option<<T as EqVesting>::Balance>)]
    pub account_id: T::AccountId,
}

#[derive(Clone, Debug, PartialEq, Call, Encode)]
pub struct VestCall<T: EqVesting> {
    pub _runtime: PhantomData<T>,
}

#[derive(Clone, Debug, PartialEq, Call, Encode)]
pub struct VestOtherCall<T: EqVesting> {
    pub account_id: T::AccountId,
}

#[derive(Clone, Debug, PartialEq, Call, Encode)]
pub struct VestedTransferCall<T: EqVesting> {
    pub account_id: T::AccountId,
    pub schedule: eq_vesting::VestingInfo<<T as EqVesting>::Balance, T::BlockNumber>,
}

#[derive(Clone, Debug, PartialEq, Call, Encode)]
pub struct ForcedVestedTransferCall<T: EqVesting> {
    pub account_id: T::AccountId,
}
