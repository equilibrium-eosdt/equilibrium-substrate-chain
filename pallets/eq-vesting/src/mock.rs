#![cfg(test)]

use super::*;

use frame_support::{impl_outer_origin, parameter_types, weights::Weight};
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, Identity, IdentityLookup},
    Perbill,
};

impl_outer_origin! {
    pub enum Origin for Test where system = frame_system {}
}

#[derive(Clone, Eq, PartialEq)]
pub struct Test;
parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const MaximumBlockWeight: Weight = 1024;
    pub const MaximumBlockLength: u32 = 2 * 1024;
    pub const AvailableBlockRatio: Perbill = Perbill::one();
    pub const ExistentialDeposit: u64 = 1;
}
impl frame_system::Trait for Test {
    type BaseCallFilter = ();
    type Origin = Origin;
    type Index = u64;
    type BlockNumber = u64;
    type Hash = H256;
    type Call = ();
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = ();
    type BlockHashCount = BlockHashCount;
    type MaximumBlockWeight = MaximumBlockWeight;
    type DbWeight = ();
    type BlockExecutionWeight = ();
    type ExtrinsicBaseWeight = ();
    type MaximumExtrinsicWeight = MaximumBlockWeight;
    type MaximumBlockLength = MaximumBlockLength;
    type AvailableBlockRatio = AvailableBlockRatio;
    type Version = ();
    type ModuleToIndex = ();
    type AccountData = ();
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
}

thread_local! {}

impl eq_balances::Trait for Test {
    type Balance = u64;
    type ExistentialDeposit = ExistentialDeposit;
    type BalanceChecker = ();
    type Event = ();
    type TotalIssuance = ();
    type BalanceGetter = ModuleBalances;
    type WeightInfo = ();
}
parameter_types! {
    pub const MinVestedTransfer: u64 = 1_000_000_000;
    pub const BasicCurrencyGet: eq_balances::currency::Currency = eq_balances::currency::Currency::Eq;
    pub const VestingModuleId: ModuleId = ModuleId(*b"eq/vestn");
}
pub type BasicCurrency =
    eq_balances::balance_adapter::BalanceAdapter<Test, eq_balances::Module<Test>, BasicCurrencyGet>;
impl Trait for Test {
    type Event = ();
    type Currency = BasicCurrency;
    type BlockNumberToBalance = Identity;
    type MinVestedTransfer = MinVestedTransfer;
    type WeightInfo = ();
    type ModuleId = VestingModuleId;
}
pub type System = frame_system::Module<Test>;
pub type ModuleVesting = Module<Test>;
pub type ModuleBalances = eq_balances::Module<Test>;

pub fn new_test_ext() -> sp_io::TestExternalities {
    let r = frame_system::GenesisConfig::default().build_storage::<Test>();

    r.unwrap().into()
}
