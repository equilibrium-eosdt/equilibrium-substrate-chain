#![cfg(test)]

use super::*;
use crate::mock::{new_test_ext, ModuleBalances, Origin};
use frame_support::assert_ok;
use frame_support::traits::{WithdrawReason, WithdrawReasons};

#[test]
fn no_balances() {
    new_test_ext().execute_with(|| {
        let account_id_1: u64 = 1;
        let account_id_2: u64 = 2;

        assert_ok!(ModuleBalances::transfer(
            Origin::signed(account_id_1),
            currency::Currency::Usd,
            account_id_2,
            10
        ));

        assert_eq!(
            ModuleBalances::total_balance(currency::Currency::Usd, &account_id_1),
            0
        );
        assert_eq!(
            ModuleBalances::debt(currency::Currency::Usd, &account_id_1),
            10
        );
        assert_eq!(
            ModuleBalances::total_balance(currency::Currency::Usd, &account_id_2),
            10
        );
        assert_eq!(
            ModuleBalances::debt(currency::Currency::Usd, &account_id_2),
            0
        );
    });
}

fn check_balance_and_debt(
    who: &u64,
    balance: u64,
    debt: u64,
    currency: currency::Currency,
    line: u32,
) {
    assert_eq(
        "balance",
        ModuleBalances::total_balance(currency, &who),
        balance,
        line,
    );
    assert_eq("debt", ModuleBalances::debt(currency, &who), debt, line);
}

fn check_balances_aggregates(
    currency: currency::Currency,
    positive: u64,
    negative: u64,
    line: u32,
) {
    let ba = ModuleBalances::balances_aggregates_get(&currency);
    assert_eq("positive", ba.total_issuance, positive, line);
    assert_eq("negative", ba.total_debt, negative, line);
}
use std::cell::RefCell;
thread_local! {
    static ERROR_COUNT: RefCell<u64> = RefCell::new(0);
}

use frame_support::dispatch::fmt::Display;
fn assert_eq<T: PartialOrd + Display>(msg: &str, left: T, right: T, line: u32) {
    if left == right {
        return;
    }
    ERROR_COUNT.with(|v| v.replace_with(|x| *x + 1));
    print!("\x1b[0;31m{}:\x1b[0m", msg);
    println!(
        "left: {}, right: {}, file: {}:{}",
        left,
        right,
        file!(),
        line
    );
}

use currency::Currency::*;
#[test]
fn test_aggregates_balances() {
    new_test_ext().execute_with(|| {
        let account_id_10: u64 = 10;
        let account_id_20: u64 = 20;
        let account_id_30: u64 = 30;

        let current_line = line!();
        println!("defined on line: {}", current_line);

        check_balance_and_debt(&account_id_10, 10_000_000_000, 0, Usd, line!());
        check_balance_and_debt(&account_id_20, 20_000_000_000, 0, Usd, line!());
        check_balance_and_debt(&account_id_30, 30_000_000_000, 0, Usd, line!());
        check_balances_aggregates(Usd, 60_000_000_000, 0, line!());

        assert_ok!(ModuleBalances::transfer(
            Origin::signed(account_id_10),
            Usd,
            account_id_20,
            25_000_000_000
        ));

        check_balance_and_debt(&account_id_10, 0, 15_000_000_000, Usd, line!());
        check_balance_and_debt(&account_id_20, 45_000_000_000, 0, Usd, line!());
        check_balance_and_debt(&account_id_30, 30_000_000_000, 0, Usd, line!());
        check_balances_aggregates(Usd, 75_000_000_000, 15_000_000_000, line!());

        assert_ok!(ModuleBalances::transfer(
            Origin::signed(account_id_20),
            Usd,
            account_id_30,
            57_000_000_000
        ));

        check_balance_and_debt(&account_id_10, 0, 15_000_000_000, Usd, line!());
        check_balance_and_debt(&account_id_20, 0, 12_000_000_000, Usd, line!());
        check_balance_and_debt(&account_id_30, 87_000_000_000, 0, Usd, line!());
        check_balances_aggregates(Usd, 87_000_000_000, 27_000_000_000, line!());

        assert_ok!(ModuleBalances::transfer(
            Origin::signed(account_id_30),
            Usd,
            account_id_10,
            90_000_000_000
        ));

        check_balance_and_debt(&account_id_10, 75_000_000_000, 0, Usd, line!());
        check_balance_and_debt(&account_id_20, 0, 12_000_000_000, Usd, line!());
        check_balance_and_debt(&account_id_30, 0, 3_000_000_000, Usd, line!());
        check_balances_aggregates(Usd, 75_000_000_000, 15_000_000_000, line!());
    });
}

#[test]
fn test_deposit() {
    new_test_ext().execute_with(|| {
        ERROR_COUNT.with(|v| v.replace(0));
        let account_id_100: u64 = 100;
        let account_id_200: u64 = 200;
        let account_id_300: u64 = 300;
        let account_id_400: u64 = 400;

        #[allow(unused_must_use)]
        {
            ModuleBalances::deposit_into_existing(Usd, &account_id_100, 50);
            ModuleBalances::deposit_creating(Usd, &account_id_200, 100);
            ModuleBalances::deposit_into_existing(Usd, &account_id_100, 50);
            ModuleBalances::deposit_creating(Usd, &account_id_200, 100);
            ModuleBalances::deposit_into_existing(Usd, &account_id_300, 300);
            ModuleBalances::deposit_creating(Usd, &account_id_400, 400);
        }

        check_balance_and_debt(&account_id_100, 100, 0, Usd, line!());
        check_balance_and_debt(&account_id_200, 200, 0, Usd, line!());
        check_balance_and_debt(&account_id_300, 300, 0, Usd, line!());
        check_balance_and_debt(&account_id_400, 400, 0, Usd, line!());
        check_balances_aggregates(Usd, 60000001000, 0, line!());

        #[allow(unused_must_use)]
        {
            ModuleBalances::deposit_into_existing(Eos, &account_id_100, 100);
            ModuleBalances::deposit_creating(Eos, &account_id_200, 200);
            ModuleBalances::deposit_into_existing(Eos, &account_id_300, 300);
            ModuleBalances::deposit_creating(Eos, &account_id_400, 400);
        }

        check_balance_and_debt(&account_id_100, 100, 0, Eos, line!());
        check_balance_and_debt(&account_id_200, 200, 0, Eos, line!());
        check_balance_and_debt(&account_id_300, 300, 0, Eos, line!());
        check_balance_and_debt(&account_id_400, 400, 0, Eos, line!());
        check_balances_aggregates(Eos, 1000, 0, line!());

        ERROR_COUNT.with(|f| {
            assert_eq!(*f.borrow(), 0);
        });
    });
}

#[test]
fn test_ensure_can_withdraw_and_withdraw() {
    new_test_ext().execute_with(|| {
        ERROR_COUNT.with(|v| v.replace(0));
        let account_id_100: u64 = 100;
        let account_id_200: u64 = 200;
        let account_id_300: u64 = 300;
        let account_id_400: u64 = 400;

        assert_ok!(ModuleBalances::ensure_can_withdraw(
            Usd,
            &account_id_100,
            100,
            WithdrawReasons::all(),
            0
        ));
        assert_ok!(ModuleBalances::ensure_can_withdraw(
            Usd,
            &account_id_200,
            200,
            WithdrawReasons::all(),
            0
        ));
        assert_ok!(ModuleBalances::ensure_can_withdraw(
            Usd,
            &account_id_300,
            300,
            WithdrawReasons::all(),
            0
        ));
        assert_ok!(ModuleBalances::ensure_can_withdraw(
            Usd,
            &account_id_400,
            400,
            WithdrawReasons::all(),
            0
        ));

        #[allow(unused_must_use)]
        {
            ModuleBalances::withdraw(
                Usd,
                &account_id_100,
                100,
                WithdrawReason::TransactionPayment.into(),
                ExistenceRequirement::KeepAlive,
            );
            ModuleBalances::withdraw(
                Usd,
                &account_id_200,
                200,
                WithdrawReasons::all(),
                ExistenceRequirement::AllowDeath,
            );
            ModuleBalances::withdraw(
                Usd,
                &account_id_300,
                300,
                WithdrawReasons::all(),
                ExistenceRequirement::KeepAlive,
            );
            ModuleBalances::withdraw(
                Usd,
                &account_id_400,
                400,
                WithdrawReasons::all(),
                ExistenceRequirement::AllowDeath,
            );
        }

        check_balance_and_debt(&account_id_100, 0, 100, Usd, line!());
        check_balance_and_debt(&account_id_200, 0, 200, Usd, line!());
        check_balance_and_debt(&account_id_300, 0, 300, Usd, line!());
        check_balance_and_debt(&account_id_400, 0, 400, Usd, line!());
        check_balances_aggregates(Usd, 60000000000, 1000, line!());
        ERROR_COUNT.with(|f| {
            assert_eq!(*f.borrow(), 0);
        });
    });
}
