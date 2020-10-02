use codec::{Codec, Decode, Encode};
use core::marker::PhantomData;
use frame_support::Parameter;
use sp_runtime::traits::{AtLeast32Bit, MaybeSerializeDeserialize, Member};
use std::fmt::Debug;
use substrate_subxt::system::{System, SystemEventsDecoder};
use substrate_subxt_proc_macro::{module, Call, Event, Store};

#[module]
pub trait Balances: System {
    type Balance: Parameter
        + Member
        + AtLeast32Bit
        + Codec
        + Default
        + Copy
        + MaybeSerializeDeserialize
        + Debug
        + From<u64>
        + Into<u64>;
    type Currency: Default + Codec + Member;
}

#[derive(Clone, Debug, Eq, PartialEq, Store, Encode)]
pub struct AccountStore<T: Balances> {
    #[store(returns = eq_balances::SignedBalance<<T as Balances>::Balance>)]
    pub account_id: T::AccountId,
    pub currency: eq_balances::currency::Currency,
}

#[derive(Clone, Debug, Eq, PartialEq, Store, Encode)]
pub struct BalancesAggregatesStore<T: Balances> {
    #[store(returns = eq_balances::BalancesAggregate<<T as Balances>::Balance>)]
    pub currency: eq_balances::currency::Currency,
    pub _runtime: PhantomData<T>,
}

#[derive(Clone, Debug, PartialEq, Call, Encode)]
pub struct TransferCall<T: Balances> {
    pub currency: eq_balances::currency::Currency,
    pub to: <T as System>::Address,
    pub amount: <T as Balances>::Balance,
}

#[derive(Clone, Debug, Eq, PartialEq, Event, Decode)]
pub struct TransferEvent<T: Balances> {
    pub from: <T as System>::AccountId,
    pub to: <T as System>::AccountId,
    pub currency: eq_balances::currency::Currency,
    pub amount: <T as Balances>::Balance,
}

#[derive(Clone, Debug, PartialEq, Call, Encode)]
pub struct DepositCall<T: Balances> {
    pub currency: eq_balances::currency::Currency,
    pub to: <T as System>::Address,
    pub amount: <T as Balances>::Balance,
}

#[derive(Clone, Debug, PartialEq, Call, Encode)]
pub struct BurnCall<T: Balances> {
    pub currency: eq_balances::currency::Currency,
    pub from: <T as System>::Address,
    pub amount: <T as Balances>::Balance,
}
