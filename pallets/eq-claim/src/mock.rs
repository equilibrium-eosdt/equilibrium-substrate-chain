#![cfg(test)]

use super::Call as ClaimsCall;
use super::*;
use crate::secp_utils::*;
use crate::EthereumAddress;
use codec::Encode;
use eq_balances::currency;
use frame_support::{
    assert_err, assert_noop, assert_ok,
    dispatch::DispatchError::BadOrigin,
    impl_outer_dispatch, impl_outer_origin, ord_parameter_types, parameter_types,
    traits::ExistenceRequirement,
    weights::{GetDispatchInfo, Pays},
};
use hex_literal::hex;
use sp_core::H256;
use sp_io::hashing::keccak_256;
use sp_runtime::ModuleId;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, Identity, IdentityLookup},
    Perbill,
};

impl_outer_origin! {
    pub enum Origin for Test {}
}

impl_outer_dispatch! {
    pub enum Call for Test where origin: Origin {
        claims::Claims,
    }
}
// For testing the module, we construct most of a mock runtime. This means
// first constructing a configuration type (`Test`) which `impl`s each of the
// configuration traits of modules we want to use.
#[derive(Clone, Eq, PartialEq)]
pub struct Test;
parameter_types! {
    pub const BlockHashCount: u32 = 250;
    pub const MaximumBlockWeight: u32 = 4 * 1024 * 1024;
    pub const MaximumBlockLength: u32 = 4 * 1024 * 1024;
    pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
    pub const ExistentialDeposit: u64 = 1;
}
impl frame_system::Trait for Test {
    type BaseCallFilter = ();
    type Origin = Origin;
    type Call = Call;
    type Index = u64;
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<u64>;
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
    type OnKilledAccount = Balances;
    type SystemWeightInfo = ();
}

pub type ModuleBalances = eq_balances::Module<Test>;

pub struct BalanceCheckerMock {}

impl eq_balances::BalanceChecker<u64, u64> for BalanceCheckerMock {
    fn can_change_balance(
        who: &u64,
        currency: &currency::Currency,
        change: &eq_balances::SignedBalance<u64>,
    ) -> bool {
        let res = match change {
            eq_balances::SignedBalance::Positive(_) => true,
            eq_balances::SignedBalance::Negative(change_value) => {
                let balance =
                    <Balances as eq_balances::BalanceGetter<u64, u64>>::get_balance(who, currency);
                match balance {
                    eq_balances::SignedBalance::Negative(_) => false,
                    eq_balances::SignedBalance::Positive(balance_value) => {
                        balance_value >= *change_value
                    }
                }
            }
        };
        res
    }
}

impl eq_balances::Trait for Test {
    type Balance = u64;
    type ExistentialDeposit = ExistentialDeposit;
    type BalanceChecker = BalanceCheckerMock;
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

impl eq_vesting::Trait for Test {
    type Event = ();
    type Currency = BasicCurrency;
    type BlockNumberToBalance = Identity;
    type MinVestedTransfer = MinVestedTransfer;
    type WeightInfo = ();
    type ModuleId = VestingModuleId;
}

parameter_types! {
    pub Prefix: &'static [u8] = b"Pay RUSTs to the TEST account:";
}
ord_parameter_types! {
    pub const Six: u64 = 6;
}

impl Trait for Test {
    type Event = ();
    type VestingSchedule = Vesting;
    type Prefix = Prefix;
    type MoveClaimOrigin = frame_system::EnsureSignedBy<Six, u64>;
    type VestingAccountGetter = Vesting;
    type WeightInfo = ();
}
// type System = frame_system::Module<Test>; // fix
type Balances = eq_balances::Module<Test>;
type Vesting = eq_vesting::Module<Test>;
type Claims = Module<Test>;

fn alice() -> secp256k1::SecretKey {
    secp256k1::SecretKey::parse(&keccak_256(b"Alice")).unwrap()
}
fn bob() -> secp256k1::SecretKey {
    secp256k1::SecretKey::parse(&keccak_256(b"Bob")).unwrap()
}
fn dave() -> secp256k1::SecretKey {
    secp256k1::SecretKey::parse(&keccak_256(b"Dave")).unwrap()
}
fn eve() -> secp256k1::SecretKey {
    secp256k1::SecretKey::parse(&keccak_256(b"Eve")).unwrap()
}
fn frank() -> secp256k1::SecretKey {
    secp256k1::SecretKey::parse(&keccak_256(b"Frank")).unwrap()
}

// This function basically just builds a genesis storage key/value store according to
// our desired mockup.
pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap();
    // We use default for brevity, but you can configure as desired if needed.
    eq_balances::GenesisConfig::<Test>::default()
        .assimilate_storage(&mut t)
        .unwrap();
    GenesisConfig::<Test> {
        claims: vec![
            (eth(&alice()), 100, None, false),
            (eth(&dave()), 200, None, true),
            (eth(&eve()), 300, Some(42), true),
            (eth(&frank()), 400, Some(43), false),
        ],
        vesting: vec![(eth(&alice()), (50, 10, 1))],
    }
    .assimilate_storage(&mut t)
    .unwrap();
    t.into()
}

fn total_claims() -> u64 {
    100 + 200 + 300 + 400
}

#[test]
fn basic_setup_works() {
    new_test_ext().execute_with(|| {
        assert_eq!(Claims::total(), total_claims());
        assert_eq!(Claims::claims(&eth(&alice())), Some(100));
        assert_eq!(Claims::claims(&eth(&dave())), Some(200));
        assert_eq!(Claims::claims(&eth(&eve())), Some(300));
        assert_eq!(Claims::claims(&eth(&frank())), Some(400));
        assert_eq!(Claims::claims(&EthereumAddress::default()), None);
        assert_eq!(Claims::vesting(&eth(&alice())), Some((50, 10, 1)));
    });
}

#[test]
fn serde_works() {
    let x = EthereumAddress(hex!["0123456789abcdef0123456789abcdef01234567"]);
    let y = serde_json::to_string(&x).unwrap();
    assert_eq!(y, "\"0x0123456789abcdef0123456789abcdef01234567\"");
    let z: EthereumAddress = serde_json::from_str(&y).unwrap();
    assert_eq!(x, z);
}

#[test]
fn claiming_works() {
    new_test_ext().execute_with(|| {
        assert_eq!(BasicCurrency::free_balance(&42), 0);
        assert_ok!(Claims::claim(
            Origin::none(),
            42,
            sig::<Test>(&alice(), &42u64.encode(), &[][..])
        ));
        assert_eq!(BasicCurrency::free_balance(&42), 50);
        assert_eq!(Vesting::vesting_balance(&42), Some(50));
        assert_eq!(Claims::total(), total_claims() - 100);
    });
}

#[test]
fn basic_claim_moving_works() {
    new_test_ext().execute_with(|| {
        assert_eq!(BasicCurrency::free_balance(&42), 0);
        assert_noop!(
            Claims::move_claim(Origin::signed(1), eth(&alice()), eth(&bob()), None),
            BadOrigin
        );
        assert_ok!(Claims::move_claim(
            Origin::signed(6),
            eth(&alice()),
            eth(&bob()),
            None
        ));
        assert_noop!(
            Claims::claim(
                Origin::none(),
                42,
                sig::<Test>(&alice(), &42u64.encode(), &[][..])
            ),
            Error::<Test>::SignerHasNoClaim
        );
        assert_ok!(Claims::claim(
            Origin::none(),
            42,
            sig::<Test>(&bob(), &42u64.encode(), &[][..])
        ));
        assert_eq!(BasicCurrency::free_balance(&42), 50);
        assert_eq!(BasicCurrency::free_balance(&Vesting::account_id()), 50);
        assert_eq!(Vesting::vesting_balance(&42), Some(50));
        assert_eq!(Claims::total(), total_claims() - 100);
    });
}

#[test]
fn claim_attest_moving_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(Claims::move_claim(
            Origin::signed(6),
            eth(&dave()),
            eth(&bob()),
            None
        ));
        let s = sig::<Test>(&bob(), &42u64.encode(), get_statement_text());
        assert_ok!(Claims::claim_attest(
            Origin::none(),
            42,
            s,
            get_statement_text().to_vec()
        ));
        assert_eq!(BasicCurrency::free_balance(&42), 200);
    });
}

#[test]
fn attest_moving_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(Claims::move_claim(
            Origin::signed(6),
            eth(&eve()),
            eth(&bob()),
            Some(42)
        ));
        assert_ok!(Claims::attest(
            Origin::signed(42),
            get_statement_text().to_vec()
        ));
        assert_eq!(BasicCurrency::free_balance(&42), 300);
    });
}

#[test]
fn claiming_does_not_bypass_signing() {
    new_test_ext().execute_with(|| {
        assert_ok!(Claims::claim(
            Origin::none(),
            42,
            sig::<Test>(&alice(), &42u64.encode(), &[][..])
        ));
        assert_noop!(
            Claims::claim(
                Origin::none(),
                42,
                sig::<Test>(&dave(), &42u64.encode(), &[][..])
            ),
            Error::<Test>::InvalidStatement,
        );
        assert_noop!(
            Claims::claim(
                Origin::none(),
                42,
                sig::<Test>(&eve(), &42u64.encode(), &[][..])
            ),
            Error::<Test>::InvalidStatement,
        );
        assert_ok!(Claims::claim(
            Origin::none(),
            42,
            sig::<Test>(&frank(), &42u64.encode(), &[][..])
        ));
    });
}

#[test]
fn attest_claiming_works() {
    new_test_ext().execute_with(|| {
        assert_eq!(BasicCurrency::free_balance(&42), 0);
        let s = sig::<Test>(&dave(), &42u64.encode(), &get_statement_text()[1..]);
        let r = Claims::claim_attest(
            Origin::none(),
            42,
            s.clone(),
            get_statement_text()[1..].to_vec(),
        );
        assert_noop!(r, Error::<Test>::InvalidStatement);

        let r = Claims::claim_attest(Origin::none(), 42, s, get_statement_text().to_vec());
        assert_noop!(r, Error::<Test>::SignerHasNoClaim);
        // ^^^ we use ecdsa_recover, so an invalid signature just results in a random signer id
        // being recovered, which realistically will never have a claim.

        let s = sig::<Test>(&dave(), &42u64.encode(), get_statement_text());
        assert_ok!(Claims::claim_attest(
            Origin::none(),
            42,
            s,
            get_statement_text().to_vec()
        ));
        assert_eq!(BasicCurrency::free_balance(&42), 200);
        assert_eq!(Claims::total(), total_claims() - 200);

        let s = sig::<Test>(&dave(), &42u64.encode(), get_statement_text());
        let r = Claims::claim_attest(Origin::none(), 42, s, get_statement_text().to_vec());
        assert_noop!(r, Error::<Test>::SignerHasNoClaim);
    });
}

#[test]
fn attesting_works() {
    new_test_ext().execute_with(|| {
        assert_eq!(BasicCurrency::free_balance(&42), 0);
        assert_noop!(
            Claims::attest(Origin::signed(69), get_statement_text().to_vec()),
            Error::<Test>::SenderHasNoClaim
        );
        assert_noop!(
            Claims::attest(Origin::signed(42), get_statement_text()[1..].to_vec()),
            Error::<Test>::InvalidStatement
        );
        assert_ok!(Claims::attest(
            Origin::signed(42),
            get_statement_text().to_vec()
        ));
        assert_eq!(BasicCurrency::free_balance(&42), 300);
        assert_eq!(Claims::total(), total_claims() - 300);
    });
}

#[test]
fn claim_cannot_clobber_preclaim() {
    new_test_ext().execute_with(|| {
        assert_eq!(BasicCurrency::free_balance(&42), 0);
        // Alice's claim is 100
        assert_ok!(Claims::claim(
            Origin::none(),
            42,
            sig::<Test>(&alice(), &42u64.encode(), &[][..])
        ));
        assert_eq!(BasicCurrency::free_balance(&42), 50);
        assert_eq!(BasicCurrency::free_balance(&Vesting::account_id()), 50);
        assert_eq!(Vesting::vesting_balance(&42), Some(50));
        // Eve's claim is 300 through Account 42
        assert_ok!(Claims::attest(
            Origin::signed(42),
            get_statement_text().to_vec()
        ));
        assert_eq!(BasicCurrency::free_balance(&42), 50 + 300);
        assert_eq!(Claims::total(), total_claims() - 400);
    });
}

#[test]
fn valid_attest_transactions_are_free() {
    new_test_ext().execute_with(|| {
        let p = PrevalidateAttests::<Test>::new();
        let c = Call::Claims(ClaimsCall::attest(get_statement_text().to_vec()));
        let di = c.get_dispatch_info();
        assert_eq!(di.pays_fee, Pays::No);
        let r = p.validate(&42, &c, &di, 20);
        assert_eq!(r, TransactionValidity::Ok(ValidTransaction::default()));
    });
}

#[test]
fn invalid_attest_transactions_are_recognised() {
    new_test_ext().execute_with(|| {
        let p = PrevalidateAttests::<Test>::new();
        let c = Call::Claims(ClaimsCall::attest(get_statement_text()[1..].to_vec()));
        let di = c.get_dispatch_info();
        let r = p.validate(&42, &c, &di, 20);
        assert!(r.is_err());
        let c = Call::Claims(ClaimsCall::attest(get_statement_text()[1..].to_vec()));
        let di = c.get_dispatch_info();
        let r = p.validate(&69, &c, &di, 20);
        assert!(r.is_err());
    });
}

#[test]
fn cannot_bypass_attest_claiming() {
    new_test_ext().execute_with(|| {
        assert_eq!(BasicCurrency::free_balance(&42), 0);
        let s = sig::<Test>(&dave(), &42u64.encode(), &[]);
        let r = Claims::claim(Origin::none(), 42, s.clone());
        assert_noop!(r, Error::<Test>::InvalidStatement);
    });
}

#[test]
fn add_claim_works() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Claims::mint_claim(Origin::signed(42), eth(&bob()), 200, None, false),
            sp_runtime::traits::BadOrigin,
        );
        assert_eq!(BasicCurrency::free_balance(&42), 0);
        assert_noop!(
            Claims::claim(
                Origin::none(),
                69,
                sig::<Test>(&bob(), &69u64.encode(), &[][..])
            ),
            Error::<Test>::SignerHasNoClaim,
        );
        assert_ok!(Claims::mint_claim(
            Origin::root(),
            eth(&bob()),
            200,
            None,
            false
        ));
        assert_eq!(Claims::total(), total_claims() + 200);
        assert_ok!(Claims::claim(
            Origin::none(),
            69,
            sig::<Test>(&bob(), &69u64.encode(), &[][..])
        ));
        assert_eq!(BasicCurrency::free_balance(&69), 200);
        assert_eq!(Vesting::vesting_balance(&69), None);
        assert_eq!(Claims::total(), total_claims());
    });
}

#[test]
fn add_claim_with_vesting_works() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Claims::mint_claim(
                Origin::signed(42),
                eth(&bob()),
                200,
                Some((50, 10, 1)),
                false
            ),
            sp_runtime::traits::BadOrigin,
        );
        assert_eq!(BasicCurrency::free_balance(&42), 0);
        assert_noop!(
            Claims::claim(
                Origin::none(),
                69,
                sig::<Test>(&bob(), &69u64.encode(), &[][..])
            ),
            Error::<Test>::SignerHasNoClaim,
        );
        assert_ok!(Claims::mint_claim(
            Origin::root(),
            eth(&bob()),
            200,
            Some((50, 10, 1)),
            false
        ));
        assert_ok!(Claims::claim(
            Origin::none(),
            69,
            sig::<Test>(&bob(), &69u64.encode(), &[][..])
        ));
        assert_eq!(BasicCurrency::free_balance(&69), 150);
        assert_eq!(BasicCurrency::free_balance(&Vesting::account_id()), 50);
        assert_eq!(Vesting::vesting_balance(&69), Some(50));

        // Make sure we can not transfer the vested balance.
        assert_err!(
            BasicCurrency::transfer(&69, &80, 180, ExistenceRequirement::AllowDeath),
            eq_balances::Error::<Test>::NotAllowedToChangeBalance
        );
    });
}

#[test]
fn add_claim_with_statement_works() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Claims::mint_claim(Origin::signed(42), eth(&bob()), 200, None, true),
            sp_runtime::traits::BadOrigin,
        );
        assert_eq!(BasicCurrency::free_balance(&42), 0);
        let signature = sig::<Test>(&bob(), &69u64.encode(), get_statement_text());
        assert_noop!(
            Claims::claim_attest(
                Origin::none(),
                69,
                signature.clone(),
                get_statement_text().to_vec()
            ),
            Error::<Test>::SignerHasNoClaim
        );
        assert_ok!(Claims::mint_claim(
            Origin::root(),
            eth(&bob()),
            200,
            None,
            true
        ));
        assert_noop!(
            Claims::claim_attest(Origin::none(), 69, signature.clone(), vec![],),
            Error::<Test>::SignerHasNoClaim
        );
        assert_ok!(Claims::claim_attest(
            Origin::none(),
            69,
            signature.clone(),
            get_statement_text().to_vec()
        ));
        assert_eq!(BasicCurrency::free_balance(&69), 200);
    });
}

#[test]
fn origin_signed_claiming_fail() {
    new_test_ext().execute_with(|| {
        assert_eq!(BasicCurrency::free_balance(&42), 0);
        assert_err!(
            Claims::claim(
                Origin::signed(42),
                42,
                sig::<Test>(&alice(), &42u64.encode(), &[][..])
            ),
            sp_runtime::traits::BadOrigin,
        );
    });
}

#[test]
fn double_claiming_doesnt_work() {
    new_test_ext().execute_with(|| {
        assert_eq!(BasicCurrency::free_balance(&42), 0);
        assert_ok!(Claims::claim(
            Origin::none(),
            42,
            sig::<Test>(&alice(), &42u64.encode(), &[][..])
        ));
        assert_noop!(
            Claims::claim(
                Origin::none(),
                42,
                sig::<Test>(&alice(), &42u64.encode(), &[][..])
            ),
            Error::<Test>::SignerHasNoClaim
        );
    });
}

#[test]
fn claiming_while_vested_doesnt_work() {
    new_test_ext().execute_with(|| {
        // A user is already vested
        assert_ok!(<Test as Trait>::VestingSchedule::add_vesting_schedule(
            &69,
            total_claims(),
            100,
            10
        ));
        CurrencyOf::<Test>::make_free_balance_be(&69, total_claims());
        assert_eq!(BasicCurrency::free_balance(&69), total_claims());
        assert_ok!(Claims::mint_claim(
            Origin::root(),
            eth(&bob()),
            200,
            Some((50, 10, 1)),
            false
        ));
        // New total
        assert_eq!(Claims::total(), total_claims() + 200);

        // They should not be able to claim
        assert_noop!(
            Claims::claim(
                Origin::none(),
                69,
                sig::<Test>(&bob(), &69u64.encode(), &[][..])
            ),
            Error::<Test>::VestedBalanceExists,
        );
    });
}

#[test]
fn non_sender_sig_doesnt_work() {
    new_test_ext().execute_with(|| {
        assert_eq!(BasicCurrency::free_balance(&42), 0);
        assert_noop!(
            Claims::claim(
                Origin::none(),
                42,
                sig::<Test>(&alice(), &69u64.encode(), &[][..])
            ),
            Error::<Test>::SignerHasNoClaim
        );
    });
}

#[test]
fn non_claimant_doesnt_work() {
    new_test_ext().execute_with(|| {
        assert_eq!(BasicCurrency::free_balance(&42), 0);
        assert_noop!(
            Claims::claim(
                Origin::none(),
                42,
                sig::<Test>(&bob(), &69u64.encode(), &[][..])
            ),
            Error::<Test>::SignerHasNoClaim
        );
    });
}

#[test]
fn real_eth_sig_works() {
    new_test_ext().execute_with(|| {
			// "Pay RUSTs to the TEST account:2a00000000000000"
			let sig = hex!["444023e89b67e67c0562ed0305d252a5dd12b2af5ac51d6d3cb69a0b486bc4b3191401802dc29d26d586221f7256cd3329fe82174bdf659baea149a40e1c495d1c"];
			let sig = EcdsaSignature(sig);
			let who = 42u64.using_encoded(to_ascii_hex);
			let signer = Claims::eth_recover(&sig, &who, &[][..]).unwrap();
			assert_eq!(signer.0, hex!["6d31165d5d932d571f3b44695653b46dcc327e84"]);
		});
}

#[test]
fn real_eth_sig_works_ci() {
    new_test_ext().execute_with(|| {
			// "Pay RUSTs to the TEST account:2a00000000000000"
			let sig = hex!["ab4e9ab31a149aa456765c26decf553ddb3aba15f2cae7e3c002040c97e261777f861c89a0d340ffa6185645e2bbdf4396b77b18da9d226cb9232c5f59403e431b"];
			let sig = EcdsaSignature(sig);
			let who = [100, 52, 51, 53, 57, 51, 99, 55, 49, 53, 102, 100, 100, 51, 49, 99, 54, 49, 49, 52, 49, 97, 98, 100, 48, 52, 97, 57, 57, 102, 100, 54, 56, 50, 50, 99, 56, 53, 53, 56, 56, 53, 52, 99, 99, 100, 101, 51, 57, 97, 53, 54, 56, 52, 101, 55, 97, 53, 54, 100, 97, 50, 55, 100];
			let signer = Claims::eth_recover(&sig, &who, &[][..]).unwrap();
			assert_eq!(signer.0, hex!["5A4447BB16Ae41B00051feda82990F88da7EC2A9"]);
		});
}

#[test]
fn validate_unsigned_works() {
    use sp_runtime::traits::ValidateUnsigned;
    let source = sp_runtime::transaction_validity::TransactionSource::External;

    new_test_ext().execute_with(|| {
        assert_eq!(
            <Module<Test>>::validate_unsigned(
                source,
                &ClaimsCall::claim(1, sig::<Test>(&alice(), &1u64.encode(), &[][..]))
            ),
            Ok(ValidTransaction {
                priority: 100,
                requires: vec![],
                provides: vec![("claims", eth(&alice())).encode()],
                longevity: TransactionLongevity::max_value(),
                propagate: true,
            })
        );
        assert_eq!(
            <Module<Test>>::validate_unsigned(
                source,
                &ClaimsCall::claim(0, EcdsaSignature([0; 65]))
            ),
            InvalidTransaction::Custom(ValidityError::InvalidEthereumSignature.into()).into(),
        );
        assert_eq!(
            <Module<Test>>::validate_unsigned(
                source,
                &ClaimsCall::claim(1, sig::<Test>(&bob(), &1u64.encode(), &[][..]))
            ),
            InvalidTransaction::Custom(ValidityError::SignerHasNoClaim.into()).into(),
        );
        let s = sig::<Test>(&dave(), &1u64.encode(), get_statement_text());
        let call = ClaimsCall::claim_attest(1, s, get_statement_text().to_vec());
        assert_eq!(
            <Module<Test>>::validate_unsigned(source, &call),
            Ok(ValidTransaction {
                priority: 100,
                requires: vec![],
                provides: vec![("claims", eth(&dave())).encode()],
                longevity: TransactionLongevity::max_value(),
                propagate: true,
            })
        );
        assert_eq!(
            <Module<Test>>::validate_unsigned(
                source,
                &ClaimsCall::claim_attest(
                    1,
                    EcdsaSignature([0; 65]),
                    get_statement_text().to_vec()
                )
            ),
            InvalidTransaction::Custom(ValidityError::InvalidEthereumSignature.into()).into(),
        );

        let s = sig::<Test>(&bob(), &1u64.encode(), get_statement_text());
        let call = ClaimsCall::claim_attest(1, s, get_statement_text().to_vec());
        assert_eq!(
            <Module<Test>>::validate_unsigned(source, &call),
            InvalidTransaction::Custom(ValidityError::SignerHasNoClaim.into()).into(),
        );

        let s = sig::<Test>(&dave(), &1u64.encode(), get_statement_text());
        let call = ClaimsCall::claim_attest(1, s, get_statement_text()[1..].to_vec());
        assert_eq!(
            <Module<Test>>::validate_unsigned(source, &call),
            InvalidTransaction::Custom(ValidityError::SignerHasNoClaim.into()).into(),
        );

        let s = sig::<Test>(&dave(), &1u64.encode(), &get_statement_text()[1..]);
        let call = ClaimsCall::claim_attest(1, s, get_statement_text()[1..].to_vec());
        assert_eq!(
            <Module<Test>>::validate_unsigned(source, &call),
            InvalidTransaction::Custom(ValidityError::InvalidStatement.into()).into(),
        );
    });
}
