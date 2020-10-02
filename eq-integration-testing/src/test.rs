use crate::balances::{AccountStoreExt, BurnCall, DepositCall};
use crate::key::{AccountKey, DevNonces, DevPubKey};
use crate::keyring::{NonceManager, PubKey};
use crate::requester::{sudo_call_chain, ChainCallSuccess};
use crate::runtime;
use crate::runtime::{Balance, EqRuntime};
use crate::test_context::TestData;
use eq_balances::currency::Currency;
use eq_balances::SignedBalance;
use futures::lock::Mutex;
use sp_arithmetic::FixedI64;
use sp_runtime::{traits::AccountIdConversion, AccountId32, ModuleId};
use std::{collections::HashMap, str::FromStr, sync::Arc};
use substrate_subxt::{Client, ExtrinsicSuccessWithFee};

pub trait ToI128 {
    fn to_i128(&self) -> i128;
}

pub trait FromI128 {
    type To;
    fn from(&self) -> Option<Self::To>;
}

impl ToI128 for SignedBalance<u64> {
    fn to_i128(&self) -> i128 {
        match self {
            SignedBalance::<u64>::Positive(n) => *n as i128,
            SignedBalance::<u64>::Negative(n) => -(*n as i128),
        }
    }
}

impl ToI128 for u64 {
    fn to_i128(&self) -> i128 {
        *self as i128
    }
}

pub fn i128_to_signed_balance(n: i128) -> Option<SignedBalance<u64>> {
    if n < 0 {
        Some(SignedBalance::<u64>::Negative(-n as u64))
    } else {
        Some(SignedBalance::<u64>::Positive(n as u64))
    }
}

pub fn i128_to_u64(n: i128) -> Option<u64> {
    if n < 0 || n > u64::MAX as i128 {
        return None;
    }
    Some(n as u64)
}

pub fn i128_to_fixedi64(n: i128) -> Option<FixedI64> {
    if n < i64::MIN as i128 || n > i64::MAX as i128 {
        return None;
    }

    Some(FixedI64::from_inner(n as i64))
}

pub async fn init_nonce(
    client: &Client<EqRuntime>,
    nonces: Arc<Mutex<DevNonces>>,
    account_key: AccountKey,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut nonces = nonces.lock().await;

    if nonces.is_initialized(account_key) {
        Ok(())
    } else {
        let id = account_key.acc_id();
        let initial_nonce = client
            .fetch_or_default(
                &substrate_subxt::system::AccountStore { account_id: &id },
                None,
            )
            .await
            .unwrap()
            .nonce;

        nonces.init_nonce(account_key, initial_nonce);

        println!("Initial nonce for {:?} is {}", account_key, initial_nonce);

        Ok(())
    }
}

fn split(value: Balance, amount: Balance) -> (Balance, Balance) {
    let first = value.min(amount);
    let second = value - first;

    (first, second)
}

fn ration(value: Balance, first: Balance, second: Balance) -> (Balance, Balance) {
    let total = first.saturating_add(second);
    let amount1 = value.saturating_mul(first) / total;
    split(value, amount1)
}

pub fn apply_fee(state: &mut TestData, successes: Vec<ChainCallSuccess>) {
    for success in successes {
        if let ExtrinsicSuccessWithFee {
            dispatch_info: Some(dispatch_info),
            ..
        } = success
        {
            let (account_key, fee) = dispatch_info.partial_fee;
            // let imbalance = NegativeImbalance::<u64>::new(fee);
            let (treasury_ratio, alice_stash_ratio) = ration(fee, 20, 80);
            // let burn_fee = fee;
            state
                .balances
                .entry(account_key.acc_id())
                .and_modify(|bal| {
                    bal.entry(Currency::Eq)
                        .and_modify(|eqs| *eqs = eqs.sub_balance(fee).unwrap());
                });

            let mut empty_balances = HashMap::new();
            for currency in Currency::iterator_with_usd() {
                if currency != &Currency::Eq {
                    empty_balances.insert(*currency, SignedBalance::Positive(0));
                }
            }
            empty_balances.insert(Currency::Eq, SignedBalance::Positive(treasury_ratio));

            let trsry_acc_id: runtime::AccountId = ModuleId(*b"eq/trsry").into_account();
            state
                .balances
                .entry(trsry_acc_id)
                .and_modify(|bal| {
                    bal.entry(Currency::Eq)
                        .and_modify(|eqs| *eqs = eqs.add_balance(treasury_ratio).unwrap())
                        .or_insert(SignedBalance::Positive(treasury_ratio));
                })
                .or_insert(empty_balances);

            let alice_stash_acc_id =
                "0xbe5ddb1579b72e84524fc29e78609e3caf42e85aa118ebfe0b0ad404b5bdd25f";
            let alice_stash_acc_id = AccountId32::from_str(alice_stash_acc_id).unwrap();
            state.balances.entry(alice_stash_acc_id).and_modify(|bal| {
                bal.entry(Currency::Eq)
                    .and_modify(|eqs| *eqs = eqs.add_balance(alice_stash_ratio).unwrap());
            });
        }
    }
}

pub async fn delta_balance(
    client: &Client<EqRuntime>,
    nonces: Arc<Mutex<DevNonces>>,
    account_key: DevPubKey,
    currency: Currency,
    delta: i128,
    sudo: &String
) -> Result<Vec<ChainCallSuccess>, Box<dyn std::error::Error>> {
    if delta > 0 {
        println!("depositing...");
        sudo_call_chain(
            client,
            nonces.clone(),
            DepositCall {
                currency: currency,
                to: account_key.acc_id(),
                amount: i128_to_u64(delta).unwrap(),
            },
            sudo,
        )
        .await
    } else if delta < 0 {
        println!("burning...");
        sudo_call_chain(
            client,
            nonces.clone(),
            BurnCall {
                currency: currency,
                from: account_key.acc_id(),
                amount: i128_to_u64(-delta).unwrap(),
            },
            sudo,
        )
        .await
    } else {
        println!("delta is 0 so doing nothing");
        Ok(vec![])
    }
}

pub async fn set_balance(
    client: &Client<EqRuntime>,
    nonces: Arc<Mutex<DevNonces>>,
    account_key: DevPubKey,
    currency: Currency,
    balance: u64,
    sudo: &String
) -> Result<Vec<ChainCallSuccess>, Box<dyn std::error::Error>> {
    let curr = client
        .account(account_key.acc_id(), currency, Option::None)
        .await?
        .to_i128();
    let delta = balance.to_i128() - curr;
    println!(
        "currency({:?}), delta({}) = balance({}) - curr({})",
        currency, delta, balance, curr
    );
    delta_balance(client, nonces, account_key, currency, delta, sudo).await
}
