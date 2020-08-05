#![cfg_attr(not(feature = "std"), no_std)]

use super::*;
use sp_std::mem;
/// Opaque, move-only struct with private fields that serves as a token denoting that
/// funds have been created without any equal and opposite accounting.
#[must_use]
pub struct PositiveImbalance<Balance>(Balance)
where
    Balance: Member
        + AtLeast32BitUnsigned
        + FullCodec
        + Copy
        + MaybeSerializeDeserialize
        + Debug
        + Default;

impl<Balance> PositiveImbalance<Balance>
where
    Balance: Member
        + AtLeast32BitUnsigned
        + FullCodec
        + Copy
        + MaybeSerializeDeserialize
        + Debug
        + Default,
{
    /// Create a new positive imbalance from a balance.
    pub fn new(amount: Balance) -> Self {
        PositiveImbalance(amount)
    }
}

/// Opaque, move-only struct with private fields that serves as a token denoting that
/// funds have been destroyed without any equal and opposite accounting.
#[must_use]
pub struct NegativeImbalance<Balance>(Balance)
where
    Balance: Member
        + AtLeast32BitUnsigned
        + FullCodec
        + Copy
        + MaybeSerializeDeserialize
        + Debug
        + Default;

impl<Balance> NegativeImbalance<Balance>
where
    Balance: Member
        + AtLeast32BitUnsigned
        + FullCodec
        + Copy
        + MaybeSerializeDeserialize
        + Debug
        + Default,
{
    /// Create a new negative imbalance from a balance.
    pub fn new(amount: Balance) -> Self {
        NegativeImbalance(amount)
    }
}

impl<Balance> TryDrop for PositiveImbalance<Balance>
where
    Balance: Member
        + AtLeast32BitUnsigned
        + FullCodec
        + Copy
        + MaybeSerializeDeserialize
        + Debug
        + Default,
{
    fn try_drop(self) -> result::Result<(), Self> {
        self.drop_zero()
    }
}

impl<Balance> Imbalance<Balance> for PositiveImbalance<Balance>
where
    Balance: Member
        + AtLeast32BitUnsigned
        + FullCodec
        + Copy
        + MaybeSerializeDeserialize
        + Debug
        + Default,
{
    type Opposite = NegativeImbalance<Balance>;

    fn zero() -> Self {
        Self(Zero::zero())
    }
    fn drop_zero(self) -> result::Result<(), Self> {
        if self.0.is_zero() {
            Ok(())
        } else {
            Err(self)
        }
    }
    fn split(self, amount: Balance) -> (Self, Self) {
        let first = self.0.min(amount);
        let second = self.0 - first;

        mem::forget(self);
        (Self(first), Self(second))
    }
    fn merge(mut self, other: Self) -> Self {
        self.0 = self.0.saturating_add(other.0);
        mem::forget(other);

        self
    }
    fn subsume(&mut self, other: Self) {
        self.0 = self.0.saturating_add(other.0);
        mem::forget(other);
    }
    fn offset(self, other: Self::Opposite) -> result::Result<Self, Self::Opposite> {
        let (a, b) = (self.0, other.0);
        mem::forget((self, other));

        if a >= b {
            Ok(Self(a - b))
        } else {
            Err(NegativeImbalance::new(b - a))
        }
    }
    fn peek(&self) -> Balance {
        self.0.clone()
    }
}

impl<Balance> TryDrop for NegativeImbalance<Balance>
where
    Balance: Member
        + AtLeast32BitUnsigned
        + FullCodec
        + Copy
        + MaybeSerializeDeserialize
        + Debug
        + Default,
{
    fn try_drop(self) -> result::Result<(), Self> {
        self.drop_zero()
    }
}

impl<Balance> Imbalance<Balance> for NegativeImbalance<Balance>
where
    Balance: Member
        + AtLeast32BitUnsigned
        + FullCodec
        + Copy
        + MaybeSerializeDeserialize
        + Debug
        + Default,
{
    type Opposite = PositiveImbalance<Balance>;

    fn zero() -> Self {
        Self(Zero::zero())
    }
    fn drop_zero(self) -> result::Result<(), Self> {
        if self.0.is_zero() {
            Ok(())
        } else {
            Err(self)
        }
    }
    fn split(self, amount: Balance) -> (Self, Self) {
        let first = self.0.min(amount);
        let second = self.0 - first;

        mem::forget(self);
        (Self(first), Self(second))
    }
    fn merge(mut self, other: Self) -> Self {
        self.0 = self.0.saturating_add(other.0);
        mem::forget(other);

        self
    }
    fn subsume(&mut self, other: Self) {
        self.0 = self.0.saturating_add(other.0);
        mem::forget(other);
    }
    fn offset(self, other: Self::Opposite) -> result::Result<Self, Self::Opposite> {
        let (a, b) = (self.0, other.0);
        mem::forget((self, other));

        if a >= b {
            Ok(Self(a - b))
        } else {
            Err(PositiveImbalance::new(b - a))
        }
    }
    fn peek(&self) -> Balance {
        self.0.clone()
    }
}

impl<Balance> Drop for PositiveImbalance<Balance>
where
    Balance: Member
        + AtLeast32BitUnsigned
        + FullCodec
        + Copy
        + MaybeSerializeDeserialize
        + Debug
        + Default,
{
    /// Basic drop handler will just square up the total issuance.
    fn drop(&mut self) {}
}

impl<Balance> Drop for NegativeImbalance<Balance>
where
    Balance: Member
        + AtLeast32BitUnsigned
        + FullCodec
        + Copy
        + MaybeSerializeDeserialize
        + Debug
        + Default,
{
    /// Basic drop handler will just square up the total issuance.
    fn drop(&mut self) {}
}
