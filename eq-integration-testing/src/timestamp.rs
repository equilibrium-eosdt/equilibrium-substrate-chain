use codec::Encode;
use core::marker::PhantomData;
use frame_support::Parameter;
use sp_runtime::traits::{AtLeast32Bit, Member, Scale};
use std::fmt::Debug;
use substrate_subxt::system::{System, SystemEventsDecoder};
use substrate_subxt_proc_macro::{module, Store};

#[module]
pub trait Timestamp: System {
    type Moment: Parameter
        + Default
        + AtLeast32Bit
        + Member
        + Scale<Self::BlockNumber, Output = Self::Moment>
        + Copy;
}

#[derive(Clone, Debug, Eq, PartialEq, Store, Encode)]
pub struct NowStore<T: Timestamp> {
    #[store(returns = T::Moment)]
    /// Runtime marker.
    pub _runtime: PhantomData<T>,
}
