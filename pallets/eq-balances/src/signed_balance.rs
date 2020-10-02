#![cfg_attr(not(feature = "std"), no_std)]

use sp_runtime::{
    traits::{AtLeast32Bit, MaybeSerializeDeserialize, Member, Zero},
    RuntimeDebug,
};

use codec::{Decode, Encode, FullCodec};
use core::ops::Add;
use frame_support::traits::{Imbalance, SignedImbalance};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_std::fmt::Debug;

/// Balance that supports negative values
#[derive(Encode, Decode, Clone, PartialEq, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum SignedBalance<Balance>
where
    Balance: Member,
{
    Positive(Balance),
    Negative(Balance),
}

impl<Balance> Zero for SignedBalance<Balance>
where
    Balance: Member + AtLeast32Bit,
{
    fn zero() -> Self {
        SignedBalance::Positive(Balance::zero())
    }

    fn is_zero(&self) -> bool {
        match self {
            Self::Positive(value) => value.is_zero(),
            Self::Negative(value) => value.is_zero(),
        }
    }
}

impl<Balance> Add for SignedBalance<Balance>
where
    Balance: Member + AtLeast32Bit,
{
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        match rhs {
            SignedBalance::Positive(value) => self.add_balance(value),
            SignedBalance::Negative(value) => self.sub_balance(value),
        }
        .unwrap()
    }
}

impl<Balance> SignedBalance<Balance>
where
    Balance: Member + AtLeast32Bit,
{
    pub fn sub_balance(&self, other: Balance) -> Option<Self> {
        match self {
            SignedBalance::Positive(value) => {
                let min_to_remove = value.min(&other);
                let new_value = value.checked_sub(&min_to_remove)?;
                let new_other = other.checked_sub(&min_to_remove)?;
                if new_other.is_zero() {
                    Some(SignedBalance::Positive(new_value))
                } else {
                    Some(SignedBalance::Negative(new_other))
                }
            }
            SignedBalance::Negative(value) => {
                let new_value = other.checked_add(value)?;
                Some(SignedBalance::Negative(new_value))
            }
        }
    }

    pub fn add_balance(&self, other: Balance) -> Option<Self> {
        match self {
            SignedBalance::Negative(value) => {
                let min_to_remove = value.min(&other);
                let new_value = value.checked_sub(&min_to_remove)?;
                let new_other = other.checked_sub(&min_to_remove)?;
                if new_other.is_zero() {
                    Some(SignedBalance::Negative(new_value))
                } else {
                    Some(SignedBalance::Positive(new_other))
                }
            }
            SignedBalance::Positive(value) => {
                let new_value = other.checked_add(value)?;
                Some(SignedBalance::Positive(new_value))
            }
        }
    }
}

impl<Balance> Default for SignedBalance<Balance>
where
    Balance: Member + Default,
{
    fn default() -> SignedBalance<Balance> {
        SignedBalance::Positive(Default::default())
    }
}

impl<
        B: AtLeast32Bit + FullCodec + Copy + MaybeSerializeDeserialize + Debug + Default + Member,
        P: Imbalance<B, Opposite = N>,
        N: Imbalance<B, Opposite = P>,
    > From<&SignedImbalance<B, P>> for SignedBalance<B>
{
    fn from(imbalance: &SignedImbalance<B, P>) -> SignedBalance<B> {
        match imbalance {
            SignedImbalance::Positive(x) => SignedBalance::Positive(x.peek()),
            SignedImbalance::Negative(x) => SignedBalance::Negative(x.peek()),
        }
    }
}
