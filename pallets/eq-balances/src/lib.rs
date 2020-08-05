#![cfg_attr(not(feature = "std"), no_std)]

mod mock;
mod tests;

pub mod balance_adapter;
mod imbalances;
pub mod signed_balance;

pub use eq_primitives::currency;
use eq_primitives::currency::Currency;

use codec::{Codec, Decode, Encode, FullCodec};
use frame_support::{
    debug, decl_error, decl_event, decl_module, decl_storage,
    dispatch::{DispatchError, DispatchResult},
    ensure,
    storage::IterableStorageDoubleMap,
    storage::IterableStorageMap,
    traits::{
        ExistenceRequirement, Get, Imbalance, OnKilledAccount, SignedImbalance, TryDrop,
        WithdrawReasons,
    },
    Parameter,
};
use impl_trait_for_tuples::impl_for_tuples;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
pub use signed_balance::{SignedBalance, SignedBalance::*};
use sp_runtime::traits::{
    AtLeast32BitUnsigned, Bounded, CheckedAdd, CheckedSub, MaybeSerializeDeserialize, Member,
    Saturating, Zero,
};
use sp_std::prelude::*;
use sp_std::{collections::btree_map::BTreeMap, fmt::Debug, result};
use system::{ensure_root, ensure_signed};

pub use imbalances::{NegativeImbalance, PositiveImbalance};
use sp_arithmetic::{
    traits::Saturating as arifmetic_saturating, FixedI128, FixedI64, FixedPointNumber,
};
use system as frame_system;

pub trait Trait: system::Trait {
    // add enum currency as a type
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
    type TotalIssuance: Get<Self::Balance>; // change for multi currency
    type ExistentialDeposit: Get<Self::Balance>;

    type BalanceChecker: BalanceChecker<Self::Balance, Self::AccountId>;

    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

/// Stores total values of issuance and debt.
#[derive(Encode, Decode, Clone, Default, PartialEq, Debug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct BalancesAggregate<Balance> {
    pub total_issuance: Balance,
    pub total_debt: Balance,
}

decl_storage! {
    trait Store for Module<T: Trait> as EqBalances {
        pub Account: double_map hasher(blake2_128_concat) T::AccountId, hasher(blake2_128_concat) currency::Currency => SignedBalance<T::Balance>;

        pub BalancesAggregates get (fn balances_aggregates): map hasher(blake2_128_concat) currency::Currency => BalancesAggregate<T::Balance>;
    }
    add_extra_genesis {
        config(balances): Vec<(T::AccountId, T::Balance, u8)>;
        // ^^ begin, length, amount liquid at genesis
        build(|config: &GenesisConfig<T>| {

            for &(ref who, free, currency) in config.balances.iter() {
                let currency_typed: currency::Currency = currency.into();
                <Account<T>>::insert(who, currency_typed, SignedBalance::Positive(free));
            }
            <Module<T>>::balances_aggregates_fix();
        });
    }
}

// The pallet's events
decl_event!(
    pub enum Event<T> where
        <T as system::Trait>::AccountId,
        <T as Trait>::Balance
    {
        Transfer(AccountId, AccountId, Currency, Balance),
    }
);

// The pallet's errors
decl_error! {
    pub enum Error for Module<T: Trait> {
        /// Vesting balance too high to send value
        VestingBalance,
        /// Account liquidity restrictions prevent withdrawal
        LiquidityRestrictions,
        /// Got an overflow after adding
        Overflow,
        /// Balance too low to send value
        InsufficientBalance,
        /// Value too low to create account due to existential deposit
        ExistentialDeposit,
        /// Transfer/payment would kill account
        KeepAlive,
        /// A vesting schedule already exists for this account
        ExistingVestingSchedule,
        /// Beneficiary account must pre-exist
        DeadAccount,

        NotAllowedToChangeBalance,

    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin
    {
        type Error = Error<T>;

        fn deposit_event() = default;

        /// Performs transfer to the specified account
        #[weight = 10_000]
        pub fn transfer(origin, currency: currency::Currency, to: <T as system::Trait>::AccountId, value: T::Balance) -> DispatchResult
        {
            let from = ensure_signed(origin)?;
            Self::currency_transfer(currency, &from, &to, value, ExistenceRequirement::AllowDeath)
        }

        /// Performs deposit to the specified account
        #[weight = 10_000]
        pub fn deposit(origin, currency: currency::Currency, to: <T as system::Trait>::AccountId, value: T::Balance) -> DispatchResult
        {
            ensure_root(origin)?;
            if <Account<T>>::contains_key(&to, &currency) {
                Self::deposit_into_existing(currency, &to, value)?;
            } else {
                Self::deposit_creating(currency, &to, value);
            }
            Ok(())
        }

        /// Performs burn from the specified account
        #[weight = 10_000]
        pub fn burn(origin, currency: currency::Currency, from: <T as system::Trait>::AccountId, value: T::Balance) -> DispatchResult
        {
            ensure_root(origin)?;
            Self::withdraw(currency, &from, value, WithdrawReasons::all(), ExistenceRequirement::AllowDeath)?;
            Ok(())
        }
    }
}

/// Contains several operations to modify balances
pub trait BalanceSetter<AccountId, Balance>
where
    Balance: Debug + Member,
{
    fn set_balance_unsafe(
        who: &AccountId,
        currency: &currency::Currency,
        value: SignedBalance<Balance>,
    );

    fn add_balance_unsafe(
        who: &AccountId,
        currency: &currency::Currency,
        value: &SignedBalance<Balance>,
    );

    fn sub_balance_unsafe(
        who: &AccountId,
        currency: &currency::Currency,
        value: &SignedBalance<Balance>,
    );

    fn remove_balance_unsafe(who: &AccountId, currency: &currency::Currency);

    fn set_balance_with_agg_unsafe(
        who: &AccountId,
        currency: &currency::Currency,
        value: SignedBalance<Balance>,
    );

    fn remove_balance_with_agg_unsafe(who: &AccountId, currency: &currency::Currency);
}

impl<T: Trait> BalanceSetter<T::AccountId, T::Balance> for Module<T> {
    fn set_balance_unsafe(
        who: &T::AccountId,
        currency: &currency::Currency,
        value: SignedBalance<T::Balance>,
    ) {
        <Account<T>>::mutate(who, currency, |balance| {
            *balance = value;
        });
    }

    fn add_balance_unsafe(
        who: &T::AccountId,
        currency: &currency::Currency,
        value: &SignedBalance<T::Balance>,
    ) {
        <Account<T>>::mutate(who, currency, |balance| match value {
            Positive(p) => {
                *balance = balance.add_balance(*p).unwrap();
            }
            Negative(n) => {
                *balance = balance.sub_balance(*n).unwrap();
            }
        });
    }

    fn sub_balance_unsafe(
        who: &T::AccountId,
        currency: &currency::Currency,
        value: &SignedBalance<T::Balance>,
    ) {
        <Account<T>>::mutate(who, currency, |balance| match value {
            Positive(p) => {
                *balance = balance.sub_balance(*p).unwrap();
            }
            Negative(n) => {
                *balance = balance.add_balance(*n).unwrap();
            }
        });
    }

    fn remove_balance_unsafe(who: &T::AccountId, currency: &currency::Currency) {
        <Account<T>>::remove(who, currency);
    }

    fn set_balance_with_agg_unsafe(
        who: &T::AccountId,
        currency: &currency::Currency,
        value: SignedBalance<T::Balance>,
    ) {
        <Account<T>>::mutate(who, currency, |balance| {
            Self::balances_aggregates_sub(&currency, &balance);
            *balance = value;
            Self::balances_aggregates_add(&currency, balance);
        });
    }
    fn remove_balance_with_agg_unsafe(who: &T::AccountId, currency: &currency::Currency) {
        let balance = <Account<T>>::get(who, currency);
        Self::balances_aggregates_sub(&currency, &balance);
        <Account<T>>::remove(who, currency);
    }
}

/// Checks if specified balance can be changed.
pub trait BalanceChecker<Balance, AccountId>
where
    Balance: Member + Debug,
{
    fn can_change_balance(
        _who: &AccountId,
        _currency: &currency::Currency,
        _change: &SignedBalance<Balance>,
    ) -> bool;
}

#[impl_for_tuples(5)]
impl<Balance: Member + Debug, AccountId> BalanceChecker<Balance, AccountId> for Tuple {
    fn can_change_balance(
        who: &AccountId,
        currency: &currency::Currency,
        change: &SignedBalance<Balance>,
    ) -> bool {
        let mut res: bool = true;
        for_tuples!( #( res &= Tuple::can_change_balance(&who, &currency, &change); )* );
        res
    }
}

/// Manages balances in different currencies
pub trait EqCurrency<AccountId, Balance>
where
    Balance: Member
        + AtLeast32BitUnsigned
        + FullCodec
        + Copy
        + MaybeSerializeDeserialize
        + Debug
        + Default,
{
    fn total_balance(currency: currency::Currency, who: &AccountId) -> Balance;
    fn debt(currency: currency::Currency, who: &AccountId) -> Balance;
    fn can_slash(currency: currency::Currency, who: &AccountId, value: Balance) -> bool;
    fn currency_total_issuance(currency: currency::Currency) -> Balance;
    fn currency_minimum_balance(currency: currency::Currency) -> Balance;
    fn burn(currency: currency::Currency, amount: Balance) -> PositiveImbalance<Balance>;
    fn issue(currency: currency::Currency, amount: Balance) -> NegativeImbalance<Balance>;
    fn free_balance(currency: currency::Currency, who: &AccountId) -> Balance;
    fn ensure_can_withdraw(
        currency: currency::Currency,
        who: &AccountId,
        amount: Balance,
        reasons: WithdrawReasons,
        new_balance: Balance,
    ) -> DispatchResult;
    fn currency_transfer(
        currency: currency::Currency,
        transactor: &AccountId,
        dest: &AccountId,
        value: Balance,
        existence_requirement: ExistenceRequirement,
    ) -> DispatchResult;
    fn slash(
        currency: currency::Currency,
        who: &AccountId,
        value: Balance,
    ) -> (NegativeImbalance<Balance>, Balance);
    fn deposit_into_existing(
        currency: currency::Currency,
        who: &AccountId,
        value: Balance,
    ) -> Result<PositiveImbalance<Balance>, DispatchError>;
    fn deposit_creating(
        currency: currency::Currency,
        who: &AccountId,
        value: Balance,
    ) -> PositiveImbalance<Balance>;
    fn withdraw(
        currency: currency::Currency,
        who: &AccountId,
        value: Balance,
        reasons: WithdrawReasons,
        liveness: ExistenceRequirement,
    ) -> Result<NegativeImbalance<Balance>, DispatchError>;
    fn make_free_balance_be(
        currency: currency::Currency,
        who: &AccountId,
        value: Balance,
    ) -> SignedImbalance<Balance, PositiveImbalance<Balance>>;
    fn balances_aggregates_fix();
    fn balances_aggregates_sub(currency: &currency::Currency, balance: &SignedBalance<Balance>);
    fn balances_aggregates_add(currency: &currency::Currency, balance: &SignedBalance<Balance>);
    fn balances_aggregates_get(currency: &currency::Currency) -> BalancesAggregate<Balance>;
}

impl<T: Trait> EqCurrency<T::AccountId, T::Balance> for Module<T> {
    fn total_balance(currency: currency::Currency, who: &T::AccountId) -> T::Balance {
        let balance = <Account<T>>::get(&who, &currency);
        match balance {
            SignedBalance::Positive(balance) => balance,
            SignedBalance::Negative(_) => T::Balance::zero(),
        }
    }

    fn debt(currency: currency::Currency, who: &T::AccountId) -> T::Balance {
        let balance = <Account<T>>::get(&who, &currency);
        match balance {
            SignedBalance::Negative(balance) => balance,
            SignedBalance::Positive(_) => T::Balance::zero(),
        }
    }

    fn can_slash(currency: currency::Currency, who: &T::AccountId, value: T::Balance) -> bool {
        panic!("NotImplementedPanic: can_slash!");
    }

    fn currency_total_issuance(_currency: currency::Currency) -> T::Balance {
        Self::balances_aggregates(_currency).total_issuance
    }

    fn currency_minimum_balance(_currency: currency::Currency) -> T::Balance {
        T::ExistentialDeposit::get()
    }

    fn burn(
        _currency: currency::Currency,
        mut amount: T::Balance,
    ) -> PositiveImbalance<T::Balance> {
        panic!("NotImplementedPanic: burn!");
    }

    fn issue(
        _currency: currency::Currency,
        mut amount: T::Balance,
    ) -> NegativeImbalance<T::Balance> {
        panic!("NotImplementedPanic: issue!");
    }

    fn free_balance(currency: currency::Currency, who: &T::AccountId) -> T::Balance {
        panic!("NotImplementedPanic: free_balance!");
    }

    fn ensure_can_withdraw(
        currency: currency::Currency,
        who: &T::AccountId,
        amount: T::Balance,
        _reasons: WithdrawReasons,
        _new_balance: T::Balance,
    ) -> DispatchResult {
        ensure!(
            T::BalanceChecker::can_change_balance(
                &who,
                &currency,
                &SignedBalance::Negative(amount)
            ),
            Error::<T>::NotAllowedToChangeBalance
        );
        Ok(())
    }

    fn currency_transfer(
        currency: currency::Currency,
        transactor: &T::AccountId,
        dest: &T::AccountId,
        value: T::Balance,
        _existence_requirement: ExistenceRequirement,
    ) -> DispatchResult {
        if value.is_zero() || transactor == dest {
            return Ok(());
        }

        <Account<T>>::mutate(transactor, &currency, |from_account| -> DispatchResult {
            <Account<T>>::mutate(dest, &currency, |to_account| -> DispatchResult {
                ensure!(
                    T::BalanceChecker::can_change_balance(
                        &transactor,
                        &currency,
                        &SignedBalance::Negative(value)
                    ),
                    Error::<T>::NotAllowedToChangeBalance
                );
                ensure!(
                    T::BalanceChecker::can_change_balance(
                        &dest,
                        &currency,
                        &SignedBalance::Positive(value)
                    ),
                    Error::<T>::NotAllowedToChangeBalance
                );

                Self::balances_aggregates_sub(&currency, &from_account);
                Self::balances_aggregates_sub(&currency, &to_account);

                *from_account = from_account
                    .sub_balance(value)
                    .ok_or(Error::<T>::Overflow)?;
                *to_account = to_account.add_balance(value).ok_or(Error::<T>::Overflow)?;

                Self::balances_aggregates_add(&currency, &from_account);
                Self::balances_aggregates_add(&currency, &to_account);

                Ok(())
            })?;

            Self::deposit_event(RawEvent::Transfer(
                transactor.clone(),
                dest.clone(),
                currency.clone(),
                value,
            ));
            Ok(())
        })
    }

    fn slash(
        _currency: currency::Currency,
        _who: &T::AccountId,
        value: T::Balance,
    ) -> (NegativeImbalance<T::Balance>, T::Balance) {
        panic!("NotImplementedPanic: slash!");
    }

    fn deposit_into_existing(
        currency: currency::Currency,
        who: &T::AccountId,
        value: T::Balance,
    ) -> Result<PositiveImbalance<T::Balance>, DispatchError> {
        if value.is_zero() {
            return Ok(PositiveImbalance::zero());
        }

        <Account<T>>::mutate(
            &who,
            &currency,
            |to_account| -> Result<PositiveImbalance<T::Balance>, DispatchError> {
                ensure!(
                    T::BalanceChecker::can_change_balance(
                        &who,
                        &currency,
                        &SignedBalance::Positive(value)
                    ),
                    Error::<T>::NotAllowedToChangeBalance
                );
                Self::balances_aggregates_sub(&currency, &to_account);
                *to_account = to_account.add_balance(value).ok_or(Error::<T>::Overflow)?;
                Self::balances_aggregates_add(&currency, &to_account);
                Ok(PositiveImbalance::new(value))
            },
        )
    }

    fn deposit_creating(
        currency: currency::Currency,
        who: &T::AccountId,
        value: T::Balance,
    ) -> PositiveImbalance<T::Balance> {
        if value.is_zero() {
            return PositiveImbalance::zero();
        }

        <Account<T>>::mutate(
            &who,
            &currency,
            |to_account| -> Result<PositiveImbalance<T::Balance>, PositiveImbalance<T::Balance>> {
                if T::BalanceChecker::can_change_balance(
                    &who,
                    &currency,
                    &SignedBalance::Positive(value),
                ) {
                    Self::balances_aggregates_sub(&currency, &to_account);
                    *to_account = to_account
                        .add_balance(value)
                        .ok_or(PositiveImbalance::zero())?;
                    Self::balances_aggregates_add(&currency, &to_account);
                    Ok(PositiveImbalance::new(value))
                } else {
                    Ok(PositiveImbalance::zero())
                }
            },
        )
        .unwrap_or_else(|x| x)
    }

    fn withdraw(
        currency: currency::Currency,
        _who: &T::AccountId,
        value: T::Balance,
        _reasons: WithdrawReasons,
        _liveness: ExistenceRequirement,
    ) -> result::Result<NegativeImbalance<T::Balance>, DispatchError> {
        if value.is_zero() {
            return Ok(NegativeImbalance::zero());
        }

        <Account<T>>::mutate(
            _who,
            &currency,
            |to_account| -> Result<NegativeImbalance<T::Balance>, DispatchError> {
                ensure!(
                    T::BalanceChecker::can_change_balance(
                        &_who,
                        &currency,
                        &SignedBalance::Negative(value)
                    ),
                    Error::<T>::NotAllowedToChangeBalance
                );
                Self::balances_aggregates_sub(&currency, &to_account);
                *to_account = to_account.sub_balance(value).ok_or(Error::<T>::Overflow)?;
                Self::balances_aggregates_add(&currency, &to_account);
                Ok(NegativeImbalance::new(value))
            },
        )
    }

    /// Force the new free balance of a target account `who` to some new value `balance`.
    fn make_free_balance_be(
        currency: currency::Currency,
        who: &T::AccountId,
        value: T::Balance,
    ) -> SignedImbalance<T::Balance, PositiveImbalance<T::Balance>> {
        <Account<T>>::mutate(
            who,
            &currency,
            |account| -> Result<SignedImbalance<T::Balance, PositiveImbalance<T::Balance>>, ()> {
                let imbalance = match account {
                    SignedBalance::Positive(balance) => {
                        let a_balance = balance.clone();
                        if value > a_balance {
                            SignedImbalance::Positive(PositiveImbalance::new(value - a_balance))
                        } else {
                            SignedImbalance::Negative(NegativeImbalance::new(a_balance - value))
                        }
                    }
                    SignedBalance::Negative(balance) => {
                        let a_balance = balance.clone();
                        SignedImbalance::Positive(PositiveImbalance::new(value + a_balance))
                    }
                };

                ensure!(
                    T::BalanceChecker::can_change_balance(
                        &who,
                        &currency,
                        &SignedBalance::from(&imbalance)
                    ),
                    ()
                );

                *account = SignedBalance::Positive(value);

                Ok(imbalance)
            },
        )
        .unwrap_or(SignedImbalance::Positive(PositiveImbalance::zero()))
    }

    fn balances_aggregates_fix() {
        for currency in currency::Currency::iterator_with_usd() {
            <BalancesAggregates<T>>::mutate(currency, |currency_aggregate| {
                currency_aggregate.total_issuance = T::Balance::zero();
                currency_aggregate.total_debt = T::Balance::zero();
            });
        }
        for (_a, c, b) in <Account<T>>::iter() {
            <BalancesAggregates<T>>::mutate(c, |currency_aggregate| match b {
                Positive(p) => {
                    currency_aggregate.total_issuance += p;
                }
                Negative(n) => {
                    currency_aggregate.total_debt += n;
                }
            });
        }
    }
    fn balances_aggregates_sub(currency: &currency::Currency, balance: &SignedBalance<T::Balance>) {
        <BalancesAggregates<T>>::mutate(currency, |currency_aggregate| match balance {
            Positive(p) => {
                currency_aggregate.total_issuance -= *p;
            }
            Negative(n) => {
                currency_aggregate.total_debt -= *n;
            }
        });
    }
    fn balances_aggregates_add(currency: &currency::Currency, balance: &SignedBalance<T::Balance>) {
        <BalancesAggregates<T>>::mutate(currency, |currency_aggregate| match balance {
            Positive(p) => {
                currency_aggregate.total_issuance += *p;
            }
            Negative(n) => {
                currency_aggregate.total_debt += *n;
            }
        });
    }
    fn balances_aggregates_get(currency: &currency::Currency) -> BalancesAggregate<T::Balance> {
        <BalancesAggregates<T>>::get(currency)
    }
}

impl<T: Trait> OnKilledAccount<T::AccountId> for Module<T> {
    fn on_killed_account(who: &T::AccountId) {
        Account::<T>::remove_prefix(who);
    }
}
