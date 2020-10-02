use super::{balances, claim, rate, timestamp, vesting, whitelists};
use crate::balances::BalancesEventsDecoder;
use crate::claim::ClaimEventsDecoder;
use crate::rate::EqRateEventsDecoder;
use crate::timestamp::TimestampEventsDecoder;
use crate::whitelists::WhitelistsEventsDecoder;
use sp_runtime::traits::{BlakeTwo256, IdentifyAccount, Verify};
use sp_runtime::{generic, MultiSignature, OpaqueExtrinsic};
use substrate_subxt::{
    balances::Balances, // needed for sign extra
    extrinsic::DefaultExtra,
    sudo::Sudo,
    system::System,
    EventsDecoder,
    Metadata,
    Runtime,
};

#[derive(Clone, PartialEq, Debug)]
pub struct EqRuntime;

impl EqRuntime {
    pub fn create_decoder(metadata: Metadata) -> EventsDecoder<EqRuntime> {
        let mut decoder = EventsDecoder::<EqRuntime>::new(metadata);
        decoder.with_balances();
        decoder.with_eq_rate();
        decoder.with_timestamp();
        decoder.with_whitelists();
        decoder.with_claim();

        decoder
    }
}

impl Eq for EqRuntime {}

type Signature = MultiSignature;

impl Runtime for EqRuntime {
    type Signature = Signature;
    type Extra = DefaultExtra<Self>;
}

pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

pub type Address = AccountId;

impl System for EqRuntime {
    type Index = u32;
    type BlockNumber = u64;
    type Hash = sp_core::H256;
    type Hashing = BlakeTwo256;
    type AccountId = AccountId;
    type Address = Self::AccountId;
    type Header = generic::Header<Self::BlockNumber, BlakeTwo256>;
    type Extrinsic = OpaqueExtrinsic;
    type AccountData = ();
}

pub type Balance = u64;

impl Balances for EqRuntime {
    type Balance = Balance;
}

impl balances::Balances for EqRuntime {
    type Balance = Balance;
    type Currency = eq_balances::currency::Currency;
}

impl timestamp::Timestamp for EqRuntime {
    type Moment = u64;
}

impl Sudo for EqRuntime {}

impl whitelists::Whitelists for EqRuntime {}

impl rate::EqRate for EqRuntime {}

impl vesting::EqVesting for EqRuntime {
    type Balance = Balance;
}

impl claim::Claim for EqRuntime {
    type Balance = Balance;
    type EthereumAddress = eq_claim::EthereumAddress;
}
