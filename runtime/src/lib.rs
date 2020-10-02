#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

use crate::sp_api_hidden_includes_construct_runtime::hidden_include::sp_runtime::traits::SaturatedConversion;
use codec::Encode;
pub use eq_balances;
pub use eq_claim::EthereumAddress;
pub use eq_distribution;
pub use eq_primitives;
pub use frame_support::{
    construct_runtime, debug, parameter_types,
    traits::{Imbalance, KeyOwnerProofSystem, Randomness, StorageMapShim},
    weights::{
        constants::{BlockExecutionWeight, ExtrinsicBaseWeight, RocksDbWeight, WEIGHT_PER_SECOND},
        IdentityFee, Weight,
    },
    StorageValue,
};
use grandpa::fg_primitives;
use grandpa::{AuthorityId as GrandpaId, AuthorityList as GrandpaAuthorityList};
use pallet_transaction_payment_rpc_runtime_api::RuntimeDispatchInfo;
use sp_api::impl_runtime_apis;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::{crypto::KeyTypeId, OpaqueMetadata};
use sp_runtime::traits::{
    self, BlakeTwo256, Block as BlockT, IdentifyAccount, Identity, IdentityLookup, NumberFor,
    OpaqueKeys, Saturating, Verify,
};
use sp_runtime::transaction_validity::{TransactionSource, TransactionValidity};
#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;
use sp_runtime::{
    create_runtime_str, generic, impl_opaque_keys, ApplyExtrinsicResult, ModuleId, MultiSignature,
    PerThing, Perquintill,
};
pub use sp_runtime::{Perbill, Permill};
use sp_std::prelude::*;
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;
use static_assertions::const_assert;
pub use timestamp::Call as TimestampCall;

/// This runtime version.
pub const VERSION: RuntimeVersion = RuntimeVersion {
    spec_name: create_runtime_str!("Equilibrium"),
    impl_name: create_runtime_str!("Equilibrium"),
    authoring_version: 10,
    spec_version: 257,
    impl_version: 0,
    apis: RUNTIME_API_VERSIONS,
    transaction_version: 1,
};

/// The version information used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
    NativeVersion {
        runtime_version: VERSION,
        can_author_with: Default::default(),
    }
}

/// An index to a block.
pub type BlockNumber = u64;

/// Alias to 512-bit hash when used in the context of a transaction signature on the chain.
pub type Signature = MultiSignature;

/// Some way of identifying an account on the chain. We intentionally make it equivalent
/// to the public key of our transaction signing scheme.
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

/// The type for looking up accounts. We don't expect more than 4 billion of them, but you
/// never know...
pub type AccountIndex = u32;

/// Balance of an account.
pub type Balance = u64;

/// Index of a transaction in the chain.
pub type Index = u32;

/// A hash of some data used by the chain.
pub type Hash = sp_core::H256;

/// Digest item type.
pub type DigestItem = generic::DigestItem<Hash>;

/// Opaque types. These are used by the CLI to instantiate machinery that don't need to know
/// the specifics of the runtime. They can then be made to be agnostic over specific formats
/// of data like extrinsics, allowing for them to continue syncing the network through upgrades
/// to even the core data structures.
pub mod opaque {
    use super::*;

    pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;

    /// Opaque block header type.
    pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
    /// Opaque block type.
    pub type Block = generic::Block<Header, UncheckedExtrinsic>;
    /// Opaque block identifier type.
    pub type BlockId = generic::BlockId<Block>;
    impl_opaque_keys! {
        pub struct SessionKeys {
            pub grandpa: Grandpa,
            pub aura: Aura,
        }
    }
}

parameter_types! {
    pub const DisabledValidatorsThreshold: Perbill = Perbill::from_percent(17);
    pub const Offset: BlockNumber = 0;
    pub const Period: BlockNumber = 10;
}

impl pallet_session::Trait for Runtime {
    type Event = Event;
    type ValidatorId = <Self as system::Trait>::AccountId;
    type ValidatorIdOf = sp_runtime::traits::ConvertInto;
    type ShouldEndSession = pallet_session::PeriodicSessions<Period, Offset>;
    type SessionManager = eq_session_manager::Module<Runtime>;
    type SessionHandler = <opaque::SessionKeys as OpaqueKeys>::KeyTypeIdProviders;
    type Keys = opaque::SessionKeys;
    type DisabledValidatorsThreshold = DisabledValidatorsThreshold;
    type NextSessionRotation = pallet_session::PeriodicSessions<Period, Offset>;
    type WeightInfo = ();
}

pub const MILLISECS_PER_BLOCK: u64 = 6000;

pub const SLOT_DURATION: u64 = MILLISECS_PER_BLOCK;

// These time units are defined in number of blocks.
pub const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
pub const HOURS: BlockNumber = MINUTES * 60;
pub const DAYS: BlockNumber = HOURS * 24;

const AVERAGE_ON_INITIALIZE_WEIGHT: Perbill = Perbill::from_percent(10);
parameter_types! {
    pub const BlockHashCount: BlockNumber = 2400;
    /// We allow for 2 seconds of compute with a 6 second average block time.
    pub const MaximumBlockWeight: Weight = 2 * WEIGHT_PER_SECOND;
    pub const MaximumBlockLength: u32 = 5 * 1024 * 1024;
    pub const Version: RuntimeVersion = VERSION;
    pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
    pub MaximumExtrinsicWeight: Weight =
        AvailableBlockRatio::get().saturating_sub(AVERAGE_ON_INITIALIZE_WEIGHT)
        * MaximumBlockWeight::get();
}

#[allow(unused_parens)]
impl system::Trait for Runtime {
    type BaseCallFilter = ();
    type Origin = Origin;
    type Call = Call;
    type Index = Index;
    type BlockNumber = BlockNumber;
    type Hash = Hash;
    type Hashing = BlakeTwo256;
    type AccountId = AccountId;
    type Lookup = IdentityLookup<AccountId>;
    type Header = generic::Header<BlockNumber, BlakeTwo256>;
    type Event = Event;
    type BlockHashCount = BlockHashCount;
    type MaximumBlockWeight = MaximumBlockWeight;
    type MaximumExtrinsicWeight = MaximumExtrinsicWeight;
    type DbWeight = RocksDbWeight;
    type BlockExecutionWeight = BlockExecutionWeight;
    type ExtrinsicBaseWeight = ExtrinsicBaseWeight;
    type MaximumBlockLength = MaximumBlockLength;
    type AvailableBlockRatio = AvailableBlockRatio;
    type Version = Version;
    type ModuleToIndex = ModuleToIndex;
    type OnNewAccount = ();
    type OnKilledAccount = (eq_balances::Module<Runtime>);
    type AccountData = ();
    type SystemWeightInfo = ();
}

impl aura::Trait for Runtime {
    type AuthorityId = AuraId;
}

impl grandpa::Trait for Runtime {
    type Event = Event;
    type Call = Call;

    type KeyOwnerProofSystem = ();

    type KeyOwnerProof =
        <Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(KeyTypeId, GrandpaId)>>::Proof;

    type KeyOwnerIdentification = <Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(
        KeyTypeId,
        GrandpaId,
    )>>::IdentificationTuple;

    type HandleEquivocation = ();
}

parameter_types! {
    pub const TreasuryModuleId: ModuleId = ModuleId(*b"eq/trsry");
    pub const ParachainOfferingModuleId: ModuleId = ModuleId(*b"eq/prcho");
    pub const MinimumPeriod: u64 = SLOT_DURATION / 2;
}

impl timestamp::Trait for Runtime {
    /// A timestamp: milliseconds since the unix epoch.
    type Moment = u64;
    type OnTimestampSet = Aura;
    type MinimumPeriod = MinimumPeriod;
    type WeightInfo = ();
}

parameter_types! {
    pub const ExistentialDeposit: Balance = 0;
    pub const TotalIssuence: Balance = 0;
    pub const BasicCurrencyGet: eq_primitives::currency::Currency = eq_primitives::currency::Currency::Eq;
}

pub struct BalanceChecker;
type SignedBalance = eq_balances::SignedBalance<Balance>;

impl eq_balances::BalanceChecker<Balance, AccountId> for BalanceChecker {
    fn can_change_balance(
        who: &AccountId,
        currency: &eq_primitives::currency::Currency,
        change: &SignedBalance,
    ) -> bool {
        let res = match change {
            SignedBalance::Positive(_) => true,
            SignedBalance::Negative(change_value) => {
                let balance =
                    <Balances as eq_balances::BalanceGetter<AccountId, Balance>>::get_balance(
                        who, currency,
                    );
                match balance {
                    SignedBalance::Negative(_) => false,
                    SignedBalance::Positive(balance_value) => balance_value >= *change_value,
                }
            }
        };
        res
    }
}

impl eq_balances::Trait for Runtime {
    /// The type for recording an account's balance.
    type Balance = Balance;
    /// The ubiquitous event type.
    type Event = Event;

    type BalanceGetter = eq_balances::Module<Runtime>;
    type BalanceChecker = BalanceChecker;

    type ExistentialDeposit = ExistentialDeposit;
    type TotalIssuance = TotalIssuence;
    type WeightInfo = ();
}

pub type BasicCurrency = eq_balances::balance_adapter::BalanceAdapter<
    Runtime,
    eq_balances::Module<Runtime>,
    BasicCurrencyGet,
>;

parameter_types! {
    pub const TransactionBaseFee: Balance = 1;
    pub const TransactionByteFee: Balance = 1;
    pub const TargetBlockFullness: Perquintill = Perquintill::from_percent(25);
}

const_assert!(
    TargetBlockFullness::get().deconstruct()
        < (AvailableBlockRatio::get().deconstruct() as <Perquintill as PerThing>::Inner)
            * (<Perquintill as PerThing>::ACCURACY
                / <Perbill as PerThing>::ACCURACY as <Perquintill as PerThing>::Inner)
);

type EqImbalance = eq_balances::NegativeImbalance<Balance>;

pub struct DealWithFees;
impl frame_support::traits::OnUnbalanced<EqImbalance> for DealWithFees {
    fn on_unbalanceds<B>(mut fees_then_tips: impl Iterator<Item = EqImbalance>) {
        if let Some(fees) = fees_then_tips.next() {
            // for fees, 20% to treasury, 80% to author
            let mut split = fees.ration(20, 80);
            if let Some(tips) = fees_then_tips.next() {
                // for tips, if any, 20% to treasury, 80% to author (though this can be anything)
                tips.ration_merge_into(20, 80, &mut split);
            }

            <Balances as eq_balances::EqCurrency<AccountId, Balance>>::resolve_creating(
                eq_balances::currency::Currency::Eq,
                &Authorship::author(),
                split.1,
            );

            <Balances as eq_balances::EqCurrency<AccountId, Balance>>::resolve_creating(
                eq_balances::currency::Currency::Eq,
                &EqTreasury::account_id(),
                split.0,
            );
        }
    }
}

impl transaction_payment::Trait for Runtime {
    type Currency = BasicCurrency;
    type OnTransactionPayment = DealWithFees;
    type TransactionByteFee = TransactionByteFee;
    type WeightToFee = IdentityFee<Balance>;
    type FeeMultiplierUpdate = ();
}

parameter_types! {
    pub const UncleGenerations: BlockNumber = 5;
}

impl authorship::Trait for Runtime {
    type FindAuthor = pallet_session::FindAccountFromAuthorIndex<Self, Aura>;
    type UncleGenerations = UncleGenerations;
    type FilterUncle = ();
    type EventHandler = ();
}

impl sudo::Trait for Runtime {
    type Event = Event;
    type Call = Call;
}

parameter_types! {
    pub const MinVestedTransfer: Balance = 1_000_000_000; // 1 eq
    pub const VestingModuleId: ModuleId = ModuleId(*b"eq/vestn");
    pub Prefix: &'static [u8] = b"Pay TEST EQ to the TEST account:";
}

impl eq_vesting::Trait for Runtime {
    type Event = Event;
    type Currency = BasicCurrency;
    type BlockNumberToBalance = Identity;
    type MinVestedTransfer = MinVestedTransfer;
    type WeightInfo = ();
    type ModuleId = VestingModuleId;
}

impl eq_claim::Trait for Runtime {
    type Event = Event;
    type VestingSchedule = eq_vesting::Module<Runtime>;
    type Prefix = Prefix;
    type MoveClaimOrigin = system::EnsureNever<Self::AccountId>;
    type VestingAccountGetter = eq_vesting::Module<Runtime>;
    type WeightInfo = ();
}

type Treasury = eq_distribution::Instance1;
impl eq_distribution::Trait<Treasury> for Runtime {
    type ModuleId = TreasuryModuleId;
}

type ParachainOffering = eq_distribution::Instance2;
impl eq_distribution::Trait<ParachainOffering> for Runtime {
    type ModuleId = ParachainOfferingModuleId;
}

impl eq_session_manager::Trait for Runtime {
    type Event = Event;
    type ValidatorId = <Self as system::Trait>::AccountId;
    type RegistrationChecker = pallet_session::Module<Runtime>;
}

impl system::offchain::SigningTypes for Runtime {
    type Public = <Signature as traits::Verify>::Signer;
    type Signature = Signature;
}

impl<C> system::offchain::SendTransactionTypes<C> for Runtime
where
    Call: From<C>,
{
    type OverarchingCall = Call;
    type Extrinsic = UncheckedExtrinsic;
}

impl<LocalCall> system::offchain::CreateSignedTransaction<LocalCall> for Runtime
where
    Call: From<LocalCall>,
{
    fn create_transaction<C: system::offchain::AppCrypto<Self::Public, Self::Signature>>(
        call: Call,
        public: <Signature as traits::Verify>::Signer,
        account: AccountId,
        nonce: Index,
    ) -> Option<(
        Call,
        <UncheckedExtrinsic as traits::Extrinsic>::SignaturePayload,
    )> {
        let period = BlockHashCount::get()
            .checked_next_power_of_two()
            .map(|c| c / 2)
            .unwrap_or(2) as u64;

        let current_block = System::block_number()
            .saturated_into::<u64>()
            // The `System::block_number` is initialized with `n+1`,
            // so the actual block number is `n`.
            .saturating_sub(1);

        let extra: SignedExtra = (
            system::CheckSpecVersion::<Runtime>::new(),
            system::CheckTxVersion::<Runtime>::new(),
            system::CheckGenesis::<Runtime>::new(),
            system::CheckEra::<Runtime>::from(generic::Era::mortal(period, current_block)),
            system::CheckNonce::<Runtime>::from(nonce),
            system::CheckWeight::<Runtime>::new(),
            transaction_payment::ChargeTransactionPayment::<Runtime>::from(0),
            eq_claim::PrevalidateAttests::<Runtime>::new(),
        );

        let raw_payload = SignedPayload::new(call, extra)
            .map_err(|e| {
                debug::native::warn!("SignedPayload error: {:?}", e);
            })
            .ok()?;

        let signature = raw_payload.using_encoded(|payload| C::sign(payload, public))?;
        let (call, extra, _) = raw_payload.deconstruct();
        let address = account;
        Some((call, (address, signature, extra)))
    }
}

use sp_std::prelude::Vec;

construct_runtime!(
    pub enum Runtime where
        Block = Block,
        NodeBlock = opaque::Block,
        UncheckedExtrinsic = UncheckedExtrinsic
    {
        System: system::{Module, Call, Config, Storage, Event<T>},
        RandomnessCollectiveFlip: randomness_collective_flip::{Module, Call, Storage},
        Timestamp: timestamp::{Module, Call, Storage, Inherent},
        Aura: aura::{Module, Config<T>, Inherent},
        Grandpa: grandpa::{Module, Call, Storage, Config, Event},
        Session: pallet_session::{Module, Call, Storage, Event, Config<T>},
        Authorship: authorship::{Module, Call, Storage, Inherent},

        Balances: eq_balances::{Module, Call, Storage, Config<T>, Event<T>},

        TransactionPayment: transaction_payment::{Module, Storage},
        Sudo: sudo::{Module, Call, Config<T>, Storage, Event<T>},
        EqSessionManager: eq_session_manager::{Module, Call, Storage, Event<T>, Config<T>,},
        EqTreasury: eq_distribution::<Instance1>::{Module, Storage},
        EqParachainOffering: eq_distribution::<Instance2>::{Module, Storage},
        EqVesting: eq_vesting::{Module, Call, Storage, Event<T>, Config<T>},
        Claim: eq_claim::{Module, Call, Storage, Event<T>, Config<T>, ValidateUnsigned}
    }
);

/// The address format for describing accounts.
pub type Address = AccountId;
/// Block header type as expected by this runtime.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;
/// A Block signed with a Justification
pub type SignedBlock = generic::SignedBlock<Block>;
/// BlockId type as expected by this runtime.
pub type BlockId = generic::BlockId<Block>;
/// The SignedExtension to the basic transaction logic.
pub type SignedExtra = (
    system::CheckSpecVersion<Runtime>,
    system::CheckTxVersion<Runtime>,
    system::CheckGenesis<Runtime>,
    system::CheckEra<Runtime>,
    system::CheckNonce<Runtime>,
    system::CheckWeight<Runtime>,
    transaction_payment::ChargeTransactionPayment<Runtime>,
    eq_claim::PrevalidateAttests<Runtime>,
);

pub type SignedPayload = generic::SignedPayload<Call, SignedExtra>;
/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic = generic::UncheckedExtrinsic<Address, Call, Signature, SignedExtra>;
/// Extrinsic type that has already been checked.
pub type CheckedExtrinsic = generic::CheckedExtrinsic<AccountId, Call, SignedExtra>;
/// Executive: handles dispatch to the various modules.
pub type Executive =
    frame_executive::Executive<Runtime, Block, system::ChainContext<Runtime>, Runtime, AllModules>;

impl_runtime_apis! {
    impl sp_api::Core<Block> for Runtime {
        fn version() -> RuntimeVersion {
            VERSION
        }

        fn execute_block(block: Block) {
            Executive::execute_block(block)
        }

        fn initialize_block(header: &<Block as BlockT>::Header) {
            Executive::initialize_block(header)
        }
    }

    impl sp_api::Metadata<Block> for Runtime {
        fn metadata() -> OpaqueMetadata {
            Runtime::metadata().into()
        }
    }

    impl sp_block_builder::BlockBuilder<Block> for Runtime {
        fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
            Executive::apply_extrinsic(extrinsic)
        }

        fn finalize_block() -> <Block as BlockT>::Header {
            Executive::finalize_block()
        }

        fn inherent_extrinsics(data: sp_inherents::InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
            data.create_extrinsics()
        }

        fn check_inherents(
            block: Block,
            data: sp_inherents::InherentData,
        ) -> sp_inherents::CheckInherentsResult {
            data.check_extrinsics(&block)
        }

        fn random_seed() -> <Block as BlockT>::Hash {
            RandomnessCollectiveFlip::random_seed()
        }
    }

    impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Index> for Runtime {
        fn account_nonce(account: AccountId) -> Index {
            System::account_nonce(account)
        }
    }

    impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
        fn validate_transaction(_source: TransactionSource, tx: <Block as BlockT>::Extrinsic) -> TransactionValidity {
            Executive::validate_transaction(_source,  tx)
        }
    }

    impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
        fn offchain_worker(header: &<Block as BlockT>::Header) {
            Executive::offchain_worker(header)
        }
    }

    impl sp_consensus_aura::AuraApi<Block, AuraId> for Runtime {
        fn slot_duration() -> u64 {
            Aura::slot_duration()
        }

        fn authorities() -> Vec<AuraId> {
            Aura::authorities()
        }
    }

    impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<
        Block,
        Balance,
    > for Runtime {
        fn query_info(uxt: <Block as BlockT>::Extrinsic, len: u32) -> RuntimeDispatchInfo<Balance> {
            TransactionPayment::query_info(uxt, len)
        }
    }

    impl sp_session::SessionKeys<Block> for Runtime {
        fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
            opaque::SessionKeys::generate(seed)
        }

        fn decode_session_keys(
            encoded: Vec<u8>,
        ) -> Option<Vec<(Vec<u8>, KeyTypeId)>> {
            opaque::SessionKeys::decode_into_raw_public_keys(&encoded)
        }
    }

    impl fg_primitives::GrandpaApi<Block> for Runtime {
        fn grandpa_authorities() -> GrandpaAuthorityList {
            Grandpa::grandpa_authorities()
        }

        fn submit_report_equivocation_unsigned_extrinsic(
            _equivocation_proof: fg_primitives::EquivocationProof<
                <Block as BlockT>::Hash,
                NumberFor<Block>,
            >,
            _key_owner_proof: fg_primitives::OpaqueKeyOwnershipProof,
        ) -> Option<()> {
            None
        }

        fn generate_key_ownership_proof(
            _set_id: fg_primitives::SetId,
            _authority_id: GrandpaId,
        ) -> Option<fg_primitives::OpaqueKeyOwnershipProof> {
            // NOTE: this is the only implementation possible since we've
            // defined our key owner proof type as a bottom type (i.e. a type
            // with no values).
            None
        }
    }

    #[cfg(feature = "runtime-benchmarks")]
    impl frame_benchmarking::Benchmark<Block> for Runtime {
        fn dispatch_benchmark(
            pallet: Vec<u8>,
            benchmark: Vec<u8>,
            lowest_range_values: Vec<u32>,
            highest_range_values: Vec<u32>,
            steps: Vec<u32>,
            repeat: u32,
        ) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, sp_runtime::RuntimeString> {
            use frame_benchmarking::{add_benchmark, BenchmarkBatch, Benchmarking};

            let mut batches = Vec::<BenchmarkBatch>::new();
            let params = (
                &pallet,
                &benchmark,
                &lowest_range_values,
                &highest_range_values,
                &steps,
                repeat,
                &vec![],
            );

            add_benchmark!(params, batches, eq_balances, Balances);
            add_benchmark!(params, batches, eq_claim, Claim);
            add_benchmark!(params, batches, eq_vesting, EqVesting);

            if batches.is_empty() {
                return Err("Benchmark not found for this pallet.".into());
            }
            Ok(batches)
        }
    }
}
