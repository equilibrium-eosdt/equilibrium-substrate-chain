#![cfg(test)]

use super::Error;
use crate::mock::{new_test_ext, ModuleBalances, ModuleVesting, Origin, System, Test};
use eq_balances::{currency, BalanceGetter, BalanceSetter, SignedBalance};
use eq_utils::fx64;
use frame_support::{assert_err, assert_ok};
use sp_arithmetic::{FixedI64, FixedPointNumber};

fn set_pos_balance_with_agg_unsafe(who: &u64, currency: &currency::Currency, amount: FixedI64) {
    let balance = SignedBalance::Positive(amount.into_inner() as u64);
    <ModuleBalances as BalanceSetter<u64, u64>>::set_balance_with_agg_unsafe(
        who, currency, balance,
    );
}

#[test]
fn vested_transfer_amount_low() {
    new_test_ext().execute_with(|| {
        let module_account_id = ModuleVesting::account_id();
        let account_id = 1;
        set_pos_balance_with_agg_unsafe(&account_id, &currency::Currency::Eq, fx64!(100, 0));
        assert_err!(
            ModuleVesting::vested_transfer(
                Origin::signed(account_id),
                2,
                super::VestingInfo {
                    locked: fx64!(0, 5).into_inner() as u64,
                    per_block: fx64!(0, 5).into_inner() as u64,
                    starting_block: 1
                }
            ),
            Error::<Test>::AmountLow
        );

        assert_eq!(ModuleVesting::vesting(1), Option::None);
        assert_eq!(ModuleVesting::vesting(2), Option::None);
        assert_eq!(
            <ModuleBalances as BalanceGetter<u64, u64>>::get_balance(&1, &currency::Currency::Eq),
            eq_balances::SignedBalance::Positive(fx64!(100, 0).into_inner() as u64)
        );
        assert_eq!(
            <ModuleBalances as BalanceGetter<u64, u64>>::get_balance(
                &module_account_id,
                &currency::Currency::Eq
            ),
            eq_balances::SignedBalance::Positive(fx64!(0, 0).into_inner() as u64)
        );
        assert_eq!(
            <ModuleBalances as BalanceGetter<u64, u64>>::get_balance(&2, &currency::Currency::Eq),
            eq_balances::SignedBalance::Positive(0)
        );
    });
}

#[test]
fn vested_transfer_ok() {
    new_test_ext().execute_with(|| {
        let module_account_id = ModuleVesting::account_id();
        let account_id = 1;
        set_pos_balance_with_agg_unsafe(&account_id, &currency::Currency::Eq, fx64!(100, 0));

        let vesting_info = super::VestingInfo {
            locked: fx64!(10, 0).into_inner() as u64,
            per_block: fx64!(1, 0).into_inner() as u64,
            starting_block: 10,
        };

        assert_ok!(ModuleVesting::vested_transfer(
            Origin::signed(account_id),
            2,
            vesting_info
        ));

        assert_eq!(ModuleVesting::vesting(1), Option::None);
        assert_eq!(ModuleVesting::vesting(2), Option::Some(vesting_info));

        assert_eq!(
            <ModuleBalances as BalanceGetter<u64, u64>>::get_balance(&1, &currency::Currency::Eq),
            eq_balances::SignedBalance::Positive(fx64!(90, 0).into_inner() as u64)
        );
        assert_eq!(
            <ModuleBalances as BalanceGetter<u64, u64>>::get_balance(
                &module_account_id,
                &currency::Currency::Eq
            ),
            eq_balances::SignedBalance::Positive(fx64!(10, 0).into_inner() as u64)
        );
        assert_eq!(
            <ModuleBalances as BalanceGetter<u64, u64>>::get_balance(&2, &currency::Currency::Eq),
            eq_balances::SignedBalance::Positive(0)
        );
    });
}

#[test]
fn vested_transfer_already_exists() {
    new_test_ext().execute_with(|| {
        let module_account_id = ModuleVesting::account_id();
        let account_id = 1;
        set_pos_balance_with_agg_unsafe(&account_id, &currency::Currency::Eq, fx64!(100, 0));
        assert_ok!(ModuleVesting::vested_transfer(
            Origin::signed(account_id),
            2,
            super::VestingInfo {
                locked: fx64!(10, 0).into_inner() as u64,
                per_block: fx64!(1, 0).into_inner() as u64,
                starting_block: 10
            }
        ));
        assert_err!(
            ModuleVesting::vested_transfer(
                Origin::signed(account_id),
                2,
                super::VestingInfo {
                    locked: fx64!(10, 0).into_inner() as u64,
                    per_block: fx64!(1, 0).into_inner() as u64,
                    starting_block: 1
                }
            ),
            Error::<Test>::ExistingVestingSchedule
        );

        assert_eq!(ModuleVesting::vesting(1), Option::None);

        assert_eq!(
            <ModuleBalances as BalanceGetter<u64, u64>>::get_balance(&1, &currency::Currency::Eq),
            eq_balances::SignedBalance::Positive(fx64!(90, 0).into_inner() as u64)
        );
        assert_eq!(
            <ModuleBalances as BalanceGetter<u64, u64>>::get_balance(
                &module_account_id,
                &currency::Currency::Eq
            ),
            eq_balances::SignedBalance::Positive(fx64!(10, 0).into_inner() as u64)
        );
        assert_eq!(
            <ModuleBalances as BalanceGetter<u64, u64>>::get_balance(&2, &currency::Currency::Eq),
            eq_balances::SignedBalance::Positive(0)
        );
    });
}

#[test]
fn vest_no_vesting() {
    new_test_ext().execute_with(|| {
        let module_account_id = ModuleVesting::account_id();
        let account_id = 1;

        assert_err!(
            ModuleVesting::vest(Origin::signed(account_id),),
            Error::<Test>::NotVesting
        );

        assert_eq!(ModuleVesting::vesting(1), Option::None);

        assert_eq!(
            <ModuleBalances as BalanceGetter<u64, u64>>::get_balance(&1, &currency::Currency::Eq),
            eq_balances::SignedBalance::Positive(fx64!(0, 0).into_inner() as u64)
        );
        assert_eq!(
            <ModuleBalances as BalanceGetter<u64, u64>>::get_balance(
                &module_account_id,
                &currency::Currency::Eq
            ),
            eq_balances::SignedBalance::Positive(fx64!(0, 0).into_inner() as u64)
        );
        assert_eq!(
            <ModuleBalances as BalanceGetter<u64, u64>>::get_balance(&2, &currency::Currency::Eq),
            eq_balances::SignedBalance::Positive(0)
        );
    });
}

#[test]
fn vest_before_start() {
    new_test_ext().execute_with(|| {
        let module_account_id = ModuleVesting::account_id();
        let account_id = 1;
        set_pos_balance_with_agg_unsafe(&account_id, &currency::Currency::Eq, fx64!(100, 0));

        System::set_block_number(1);

        let vesting_info = super::VestingInfo {
            locked: fx64!(10, 0).into_inner() as u64,
            per_block: fx64!(1, 0).into_inner() as u64,
            starting_block: 10,
        };

        assert_ok!(ModuleVesting::vested_transfer(
            Origin::signed(account_id),
            2,
            vesting_info
        ));

        System::set_block_number(9);

        assert_ok!(ModuleVesting::vest(Origin::signed(2),));

        assert_eq!(
            <ModuleBalances as BalanceGetter<u64, u64>>::get_balance(&1, &currency::Currency::Eq),
            eq_balances::SignedBalance::Positive(fx64!(90, 0).into_inner() as u64)
        );
        assert_eq!(
            <ModuleBalances as BalanceGetter<u64, u64>>::get_balance(
                &module_account_id,
                &currency::Currency::Eq
            ),
            eq_balances::SignedBalance::Positive(fx64!(10, 0).into_inner() as u64)
        );
        assert_eq!(
            <ModuleBalances as BalanceGetter<u64, u64>>::get_balance(&2, &currency::Currency::Eq),
            eq_balances::SignedBalance::Positive(fx64!(0, 0).into_inner() as u64)
        );
    });
}

#[test]
fn vest_temp() {
    new_test_ext().execute_with(|| {
        let module_account_id = ModuleVesting::account_id();
        let account_id = 1;
        set_pos_balance_with_agg_unsafe(&account_id, &currency::Currency::Eq, fx64!(100, 0));

        System::set_block_number(1);

        let vesting_info = super::VestingInfo {
            locked: fx64!(10, 0).into_inner() as u64,
            per_block: fx64!(1, 0).into_inner() as u64,
            starting_block: 10,
        };

        assert_ok!(ModuleVesting::vested_transfer(
            Origin::signed(account_id),
            2,
            vesting_info
        ));

        System::set_block_number(11);

        assert_ok!(ModuleVesting::vest(Origin::signed(2),));

        assert_eq!(
            <ModuleBalances as BalanceGetter<u64, u64>>::get_balance(&1, &currency::Currency::Eq),
            eq_balances::SignedBalance::Positive(fx64!(90, 0).into_inner() as u64)
        );
        assert_eq!(
            <ModuleBalances as BalanceGetter<u64, u64>>::get_balance(
                &module_account_id,
                &currency::Currency::Eq
            ),
            eq_balances::SignedBalance::Positive(fx64!(9, 0).into_inner() as u64)
        );
        assert_eq!(
            <ModuleBalances as BalanceGetter<u64, u64>>::get_balance(&2, &currency::Currency::Eq),
            eq_balances::SignedBalance::Positive(fx64!(1, 0).into_inner() as u64)
        );
    });
}

#[test]
fn vest_temp2() {
    new_test_ext().execute_with(|| {
        let module_account_id = ModuleVesting::account_id();
        let account_id = 1;
        set_pos_balance_with_agg_unsafe(&account_id, &currency::Currency::Eq, fx64!(100, 0));

        System::set_block_number(1);

        let vesting_info = super::VestingInfo {
            locked: fx64!(10, 0).into_inner() as u64,
            per_block: fx64!(1, 0).into_inner() as u64,
            starting_block: 10,
        };

        assert_ok!(ModuleVesting::vested_transfer(
            Origin::signed(account_id),
            2,
            vesting_info
        ));

        System::set_block_number(11);

        assert_ok!(ModuleVesting::vest(Origin::signed(2),));

        assert_eq!(
            <ModuleBalances as BalanceGetter<u64, u64>>::get_balance(&1, &currency::Currency::Eq),
            eq_balances::SignedBalance::Positive(fx64!(90, 0).into_inner() as u64)
        );
        assert_eq!(
            <ModuleBalances as BalanceGetter<u64, u64>>::get_balance(
                &module_account_id,
                &currency::Currency::Eq
            ),
            eq_balances::SignedBalance::Positive(fx64!(9, 0).into_inner() as u64)
        );
        assert_eq!(
            <ModuleBalances as BalanceGetter<u64, u64>>::get_balance(&2, &currency::Currency::Eq),
            eq_balances::SignedBalance::Positive(fx64!(1, 0).into_inner() as u64)
        );

        System::set_block_number(13);

        assert_ok!(ModuleVesting::vest(Origin::signed(2),));

        assert_eq!(
            <ModuleBalances as BalanceGetter<u64, u64>>::get_balance(&1, &currency::Currency::Eq),
            eq_balances::SignedBalance::Positive(fx64!(90, 0).into_inner() as u64)
        );
        assert_eq!(
            <ModuleBalances as BalanceGetter<u64, u64>>::get_balance(
                &module_account_id,
                &currency::Currency::Eq
            ),
            eq_balances::SignedBalance::Positive(fx64!(7, 0).into_inner() as u64)
        );
        assert_eq!(
            <ModuleBalances as BalanceGetter<u64, u64>>::get_balance(&2, &currency::Currency::Eq),
            eq_balances::SignedBalance::Positive(fx64!(3, 0).into_inner() as u64)
        );
    });
}

#[test]
fn vest_all() {
    new_test_ext().execute_with(|| {
        let module_account_id = ModuleVesting::account_id();
        let account_id = 1;
        set_pos_balance_with_agg_unsafe(&account_id, &currency::Currency::Eq, fx64!(100, 0));

        System::set_block_number(1);

        let vesting_info = super::VestingInfo {
            locked: fx64!(10, 0).into_inner() as u64,
            per_block: fx64!(1, 0).into_inner() as u64,
            starting_block: 10,
        };

        assert_ok!(ModuleVesting::vested_transfer(
            Origin::signed(account_id),
            2,
            vesting_info
        ));

        System::set_block_number(11);

        assert_ok!(ModuleVesting::vest(Origin::signed(2),));

        assert_eq!(
            <ModuleBalances as BalanceGetter<u64, u64>>::get_balance(&1, &currency::Currency::Eq),
            eq_balances::SignedBalance::Positive(fx64!(90, 0).into_inner() as u64)
        );
        assert_eq!(
            <ModuleBalances as BalanceGetter<u64, u64>>::get_balance(
                &module_account_id,
                &currency::Currency::Eq
            ),
            eq_balances::SignedBalance::Positive(fx64!(9, 0).into_inner() as u64)
        );
        assert_eq!(
            <ModuleBalances as BalanceGetter<u64, u64>>::get_balance(&2, &currency::Currency::Eq),
            eq_balances::SignedBalance::Positive(fx64!(1, 0).into_inner() as u64)
        );

        System::set_block_number(21);

        assert_ok!(ModuleVesting::vest(Origin::signed(2),));

        assert_eq!(
            <ModuleBalances as BalanceGetter<u64, u64>>::get_balance(&1, &currency::Currency::Eq),
            eq_balances::SignedBalance::Positive(fx64!(90, 0).into_inner() as u64)
        );
        assert_eq!(
            <ModuleBalances as BalanceGetter<u64, u64>>::get_balance(
                &module_account_id,
                &currency::Currency::Eq
            ),
            eq_balances::SignedBalance::Positive(fx64!(0, 0).into_inner() as u64)
        );
        assert_eq!(
            <ModuleBalances as BalanceGetter<u64, u64>>::get_balance(&2, &currency::Currency::Eq),
            eq_balances::SignedBalance::Positive(fx64!(10, 0).into_inner() as u64)
        );
        assert_eq!(ModuleVesting::vesting(2), Option::None);
    });
}

#[test]
fn vest_all_init() {
    new_test_ext().execute_with(|| {
        let module_account_id = ModuleVesting::account_id();
        let account_id = 1;
        set_pos_balance_with_agg_unsafe(&account_id, &currency::Currency::Eq, fx64!(100, 0));

        System::set_block_number(100);

        let vesting_info = super::VestingInfo {
            locked: fx64!(10, 0).into_inner() as u64,
            per_block: fx64!(1, 0).into_inner() as u64,
            starting_block: 10,
        };

        assert_ok!(ModuleVesting::vested_transfer(
            Origin::signed(account_id),
            2,
            vesting_info
        ));

        assert_eq!(
            <ModuleBalances as BalanceGetter<u64, u64>>::get_balance(&1, &currency::Currency::Eq),
            eq_balances::SignedBalance::Positive(fx64!(90, 0).into_inner() as u64)
        );
        assert_eq!(
            <ModuleBalances as BalanceGetter<u64, u64>>::get_balance(
                &module_account_id,
                &currency::Currency::Eq
            ),
            eq_balances::SignedBalance::Positive(fx64!(0, 0).into_inner() as u64)
        );
        assert_eq!(
            <ModuleBalances as BalanceGetter<u64, u64>>::get_balance(&2, &currency::Currency::Eq),
            eq_balances::SignedBalance::Positive(fx64!(10, 0).into_inner() as u64)
        );
        assert_eq!(ModuleVesting::vesting(2), Option::None);
    });
}
