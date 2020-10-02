use codec::Encode;
use std::fmt::Debug;
use substrate_subxt::system::{System, SystemEventsDecoder};
use substrate_subxt_proc_macro::{module, Call, Store};

#[module]
pub trait Whitelists: System {}

#[derive(Clone, Debug, Eq, PartialEq, Store, Encode)]
pub struct WhilteListStore<T: Whitelists> {
    #[store(returns = Option<bool>)]
    pub account_id: T::AccountId,
}

#[derive(Clone, Debug, PartialEq, Call, Encode)]
pub struct AddToWhitelistCall<T: Whitelists> {
    pub account_id: T::AccountId,
}

#[derive(Clone, Debug, PartialEq, Call, Encode)]
pub struct RemoveFromWhitelistCall<T: Whitelists> {
    pub account_id: T::AccountId,
}
