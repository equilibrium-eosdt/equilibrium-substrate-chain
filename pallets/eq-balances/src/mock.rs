#![cfg(test)]

use super::*;

use frame_support::{impl_outer_origin, parameter_types, weights::Weight};
use sp_core::H256;
use sp_runtime::{testing::Header, traits::IdentityLookup, Perbill};

impl_outer_origin! {
    pub enum Origin for Test {}
}

// For testing the module, we construct most of a mock runtime. This means
// first constructing a configuration type (`Test`) which `impl`s each of the
// configuration traits of modules we want to use.
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Test;

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const MaximumBlockWeight: Weight = 1024;
    pub const MaximumBlockLength: u32 = 2 * 1024;
    pub const AvailableBlockRatio: Perbill = Perbill::one();
    pub const MinimumPeriod: u64 = 1;
    pub const EpochDuration: u64 = 3;
    pub const ExpectedBlockTime: u64 = 1;
    pub const DisabledValidatorsThreshold: Perbill = Perbill::from_percent(16);
    pub const TotalIssuence:u64 = 1_000_000_000;
    pub const ExistentialDeposit: u64 = 1;
}

type DummyValidatorId = u64;

impl system::Trait for Test {
    type Origin = Origin;
    type Index = u64;
    type BlockNumber = u64;
    type Call = ();
    type Hash = H256;
    type Version = ();
    type Hashing = sp_runtime::traits::BlakeTwo256;
    type AccountId = DummyValidatorId;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = ();
    type BlockHashCount = BlockHashCount;
    type MaximumBlockWeight = MaximumBlockWeight;
    type MaximumExtrinsicWeight = MaximumBlockWeight;
    type DbWeight = ();
    type BlockExecutionWeight = ();
    type ExtrinsicBaseWeight = ();
    type AvailableBlockRatio = AvailableBlockRatio;
    type MaximumBlockLength = MaximumBlockLength;
    type ModuleToIndex = ();
    type AccountData = ();
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type BaseCallFilter = ();
    type SystemWeightInfo = ();
}

impl Trait for Test {
    type Balance = u64;
    type TotalIssuance = TotalIssuence;
    type ExistentialDeposit = ExistentialDeposit;
    type BalanceChecker = ();
    type Event = ();
    type BalanceGetter = ModuleBalances;
    type WeightInfo = ();
}

pub type ModuleBalances = Module<Test>;

pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap();

    GenesisConfig::<Test> {
        balances: vec![
            (1, 1000_000_000_000 as u64, currency::Currency::Btc.value()),
            (2, 2000_000_000_000 as u64, currency::Currency::Btc.value()),
            (10, 10_000_000_000 as u64, currency::Currency::Usd.value()),
            (20, 20_000_000_000 as u64, currency::Currency::Usd.value()),
            (30, 30_000_000_000 as u64, currency::Currency::Usd.value()),
        ],
    }
    .assimilate_storage(&mut t)
    .unwrap();

    t.into()
}
