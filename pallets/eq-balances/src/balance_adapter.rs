#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::traits::{Currency, Get};
use sp_std::marker;

pub use super::imbalances::{NegativeImbalance, PositiveImbalance};

/// Currency trait implementation based on data from CurrencyGetter trait
pub struct BalanceAdapter<T, R, CurrencyGetter>(marker::PhantomData<(T, R, CurrencyGetter)>);

impl<AccountId, T, R, CurrencyGetter> Currency<AccountId> for BalanceAdapter<T, R, CurrencyGetter>
where
    T: super::Trait,
    R: super::EqCurrency<AccountId, T::Balance>,
    CurrencyGetter: Get<eq_primitives::currency::Currency>,
{
    type Balance = T::Balance;
    type PositiveImbalance = PositiveImbalance<T::Balance>;
    type NegativeImbalance = NegativeImbalance<T::Balance>;
    fn total_balance(who: &AccountId) -> Self::Balance {
        R::total_balance(CurrencyGetter::get(), who)
    }
    fn can_slash(who: &AccountId, value: Self::Balance) -> bool {
        R::can_slash(CurrencyGetter::get(), who, value)
    }
    fn total_issuance() -> Self::Balance {
        R::currency_total_issuance(CurrencyGetter::get())
    }
    fn minimum_balance() -> Self::Balance {
        R::currency_minimum_balance(CurrencyGetter::get())
    }
    fn burn(amount: Self::Balance) -> Self::PositiveImbalance {
        R::burn(CurrencyGetter::get(), amount)
    }
    fn issue(amount: Self::Balance) -> Self::NegativeImbalance {
        R::issue(CurrencyGetter::get(), amount)
    }
    fn free_balance(who: &AccountId) -> Self::Balance {
        R::free_balance(CurrencyGetter::get(), who)
    }
    fn ensure_can_withdraw(
        who: &AccountId,
        _amount: Self::Balance,
        reasons: frame_support::traits::WithdrawReasons,
        new_balance: Self::Balance,
    ) -> sp_runtime::DispatchResult {
        R::ensure_can_withdraw(CurrencyGetter::get(), who, _amount, reasons, new_balance)
    }
    fn transfer(
        source: &AccountId,
        dest: &AccountId,
        value: Self::Balance,
        existence_requirement: frame_support::traits::ExistenceRequirement,
    ) -> sp_runtime::DispatchResult {
        R::currency_transfer(
            CurrencyGetter::get(),
            source,
            dest,
            value,
            existence_requirement,
        )
    }
    fn slash(who: &AccountId, value: Self::Balance) -> (Self::NegativeImbalance, Self::Balance) {
        R::slash(CurrencyGetter::get(), who, value)
    }
    fn deposit_into_existing(
        who: &AccountId,
        value: Self::Balance,
    ) -> Result<Self::PositiveImbalance, sp_runtime::DispatchError> {
        R::deposit_into_existing(CurrencyGetter::get(), who, value)
    }
    fn deposit_creating(who: &AccountId, value: Self::Balance) -> Self::PositiveImbalance {
        R::deposit_creating(CurrencyGetter::get(), who, value)
    }
    fn withdraw(
        who: &AccountId,
        value: Self::Balance,
        reasons: frame_support::traits::WithdrawReasons,
        liveness: frame_support::traits::ExistenceRequirement,
    ) -> Result<Self::NegativeImbalance, sp_runtime::DispatchError> {
        R::withdraw(CurrencyGetter::get(), who, value, reasons, liveness)
    }
    fn make_free_balance_be(
        who: &AccountId,
        balance: Self::Balance,
    ) -> frame_support::traits::SignedImbalance<Self::Balance, Self::PositiveImbalance> {
        R::make_free_balance_be(CurrencyGetter::get(), who, balance)
    }
}
