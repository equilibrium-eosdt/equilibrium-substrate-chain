#![cfg_attr(not(feature = "std"), no_std)]

use impl_trait_for_tuples::impl_for_tuples;
use sp_runtime::traits::AtLeast32Bit;
use sp_std::prelude::*;

pub mod currency;

#[derive(PartialEq, Debug, Clone)]
pub enum InterestRateError {
    ExternalError,
    MathError,
    ValueError,
}

pub trait FeeManager<AccountId, Balance> {
    fn calc_fee(owner: &AccountId, last_update: &u64) -> Result<Balance, InterestRateError>;
    fn charge_fee_inner(owner: &AccountId, debt: &Balance);
    fn charge_fee(owner: &AccountId, last_update: &u64) -> Result<Balance, InterestRateError> {
        let debt_delta = Self::calc_fee(owner, last_update)?;
        Self::charge_fee_inner(owner, &debt_delta);
        Ok(debt_delta)
    }
}

#[impl_for_tuples(5)]
impl<AccountId, Balance: AtLeast32Bit + Clone> FeeManager<AccountId, Balance> for Tuple {
    fn calc_fee(owner: &AccountId, last_update: &u64) -> Result<Balance, InterestRateError> {
        let mut debt = Balance::zero();
        for_tuples!( #( debt = debt + Tuple::calc_fee(owner, last_update)?; )* );
        Ok(debt)
    }
    fn charge_fee_inner(owner: &AccountId, debt: &Balance) {
        panic!("Don't use this in tuple!");
    }
    fn charge_fee(owner: &AccountId, last_update: &u64) -> Result<Balance, InterestRateError> {
        let mut debt = Balance::zero();
        let mut fees = Vec::<Balance>::new();
        for_tuples!( #( {
            let fee = Tuple::calc_fee(owner, last_update)?;
            debt = debt + fee.clone();
            fees.push(fee);
        } )* );

        let mut index: usize = 0;
        for_tuples!( #( {
            let debt_delta = Tuple::charge_fee_inner(owner, &fees[index]);
            index += 1;
        } )* );
        Ok(debt)
    }
}

pub trait AccountGetter<AccountId> {
    fn get_account_id() -> AccountId;
}
