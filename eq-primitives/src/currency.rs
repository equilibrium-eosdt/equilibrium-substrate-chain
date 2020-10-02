#![cfg_attr(not(feature = "std"), no_std)]

use core::slice::Iter;
use frame_support::codec::{Decode, Encode};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::RuntimeDebug;

#[derive(Encode, Decode, Clone, Copy, PartialEq, RuntimeDebug, Hash)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[repr(u8)]
pub enum Currency {
    Unknown = 0,
    Usd = 1,
    Eq = 2,
    Eth = 3,
    Btc = 4,
    Eos = 5,
}

impl Currency {
    pub fn iterator() -> Iter<'static, Currency> {
        static CURRENCIES: [Currency; 4] =
            [Currency::Eq, Currency::Eth, Currency::Btc, Currency::Eos];
        CURRENCIES.iter()
    }

    pub fn iterator_with_usd() -> Iter<'static, Currency> {
        static CURRENCIES: [Currency; 5] = [
            Currency::Eq,
            Currency::Eth,
            Currency::Btc,
            Currency::Usd,
            Currency::Eos,
        ];
        CURRENCIES.iter()
    }
}

impl Eq for Currency {}

impl Default for Currency {
    fn default() -> Currency {
        Currency::Unknown
    }
}

impl Currency {
    pub fn value(&self) -> u8 {
        match *self {
            Currency::Unknown => 0x0,
            Currency::Usd => 0x1,
            Currency::Eq => 0x2,
            Currency::Eth => 0x3,
            Currency::Btc => 0x4,
            Currency::Eos => 0x5,
        }
    }
}

impl From<u8> for Currency {
    fn from(orig: u8) -> Self {
        match orig {
            0x0 => Currency::Unknown,
            0x1 => Currency::Usd,
            0x2 => Currency::Eq,
            0x3 => Currency::Eth,
            0x4 => Currency::Btc,
            0x5 => Currency::Eos,
            _ => Currency::Unknown,
        }
    }
}

pub mod test {
    use crate::currency::Currency;
    use sp_std::cmp::Ordering;

    #[derive(Copy, Clone)]
    pub struct CurrencyTag {
        pub currency: Currency,
    }
    impl CurrencyTag {
        pub fn new(currency: Currency) -> CurrencyTag {
            CurrencyTag { currency }
        }
        fn value(&self) -> i32 {
            match self.currency {
                Currency::Btc => 0,
                Currency::Eth => 1,
                Currency::Eos => 2,
                Currency::Usd => 3,
                Currency::Eq => 4,
                _ => panic!("Unexpected currency"),
            }
        }
    }
    impl PartialEq for CurrencyTag {
        fn eq(&self, _other: &Self) -> bool {
            self.value() == self.value()
        }
    }
    impl Eq for CurrencyTag {}
    impl PartialOrd for CurrencyTag {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            Some(self.value().cmp(&other.value()))
        }
    }
    impl Ord for CurrencyTag {
        fn cmp(&self, other: &Self) -> Ordering {
            self.value().cmp(&other.value())
        }
    }
}
