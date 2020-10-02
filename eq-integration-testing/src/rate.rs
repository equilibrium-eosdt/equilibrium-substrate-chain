use codec::Encode;
use core::marker::PhantomData;
use std::fmt::Debug;
use substrate_subxt::system::{System, SystemEventsDecoder};
use substrate_subxt_proc_macro::{module, Call, Store};

#[module]
pub trait EqRate: System {}

#[derive(Clone, Debug, Eq, PartialEq, Store, Encode)]
pub struct NowMillisOffsetStore<T: EqRate> {
    #[store(returns = u64)]
    pub _runtime: PhantomData<T>,
}

#[derive(Clone, Debug, PartialEq, Call, Encode)]
pub struct SetNowMillisOffset<T: EqRate> {
    pub offset: u64,
    pub _runtime: PhantomData<T>,
}
