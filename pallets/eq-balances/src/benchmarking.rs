#![cfg(feature = "runtime-benchmarks")]

use super::*;

use crate::Module as Balance;
use eq_primitives::currency;
use frame_benchmarking::{account, benchmarks};
use frame_system::{Module as System, RawOrigin};
use sp_runtime::traits::Bounded;

const SEED: u32 = 0;

benchmarks! {
    _ { }

    transfer {
        let b in 0 .. 100;
        let caller = account("caller", 0, SEED);
        Balance::<T>::make_free_balance_be(currency::Currency::Btc ,&caller, T::Balance::max_value());
        let to = account("to", 0, SEED);
    }: _ (RawOrigin::Signed(caller), currency::Currency::Btc, to, From::<u64>::from(1_000_000_000 as u64))

    deposit {
        let b in 0 .. 100;
        let to = account("to", 0, SEED);
    }: _ (RawOrigin::Root, currency::Currency::Btc, to, From::<u64>::from(1_000_000_000 as u64))

    burn {
        let b in 0 .. 100;
        let from = account("from", 0, SEED);
        Balance::<T>::make_free_balance_be(currency::Currency::Btc ,&from, From::<u64>::from(1_000_000_000 as u64));
    }: _ (RawOrigin::Root, currency::Currency::Btc, from, From::<u64>::from(1_000_000_000 as u64))
}
