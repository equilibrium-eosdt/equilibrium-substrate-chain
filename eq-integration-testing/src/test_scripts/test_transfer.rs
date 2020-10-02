use crate::balances::TransferCall;
use crate::join_chain_calls;
use crate::key::{AccountKey, DevNonces, PubKeyStore};
use crate::keyring::PubKey;
use crate::requester::call_chain;
use crate::runtime::EqRuntime;
use crate::test::{apply_fee, init_nonce, set_balance};
use crate::test_context::{compare_test_data_keystore, TestContext};
use crate::{assert_eq_test_data_keystore, IntegrationTestConfig};
use eq_balances::{currency::Currency, SignedBalance};
use eq_integration_testing_macro::tuple_to_vec;
use eq_utils::fx64;
use futures::{lock::Mutex, try_join};
use sp_arithmetic::{FixedI64, FixedPointNumber};
use std::{collections::HashMap, sync::Arc};
use substrate_subxt::Client;

pub async fn test_transfer(
    client: &Client<EqRuntime>,
    nonces: Arc<Mutex<DevNonces>>,
    pub_key_store: Arc<Mutex<PubKeyStore>>,
    integration_test_config: IntegrationTestConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Start transfer test");
    let mut state = TestContext::read_storage(client).await?;
    let mut validators = vec![];
    for val in integration_test_config.validators {
        validators.push(AccountKey::from(&val));
    }
    let alice_acc = AccountKey::generate_random();
    init_nonce(client, nonces.clone(), alice_acc).await?;
    let bob_acc = AccountKey::generate_random();
    init_nonce(client, nonces.clone(), bob_acc).await?;
    let charlie_acc = AccountKey::generate_random();
    init_nonce(client, nonces.clone(), charlie_acc).await?;
    let dave_acc = AccountKey::generate_random();
    init_nonce(client, nonces.clone(), dave_acc).await?;
    {
        let mut store = pub_key_store.lock().await;
        store.register(alice_acc.into());
        store.register(bob_acc.into());
        store.register(charlie_acc.into());
        store.register(dave_acc.into());
    }

    let min_btc = fx64!(1, 0).into_inner() as u64;
    let min_usd = 1_000_000_000_u64;
    let min_eq = 9_000_000_000_u64;

    println!("Setting charlie's Eq balance");
    set_balance(
        client,
        nonces.clone(),
        charlie_acc.into(),
        Currency::Eq,
        min_eq,
        &integration_test_config.sudo,
    )
    .await?;

    println!("Setting Dave's balances");
    join_chain_calls!(
        set_balance(
            client,
            nonces.clone(),
            dave_acc.into(),
            Currency::Btc,
            min_btc,
            &integration_test_config.sudo,
        ),
        set_balance(
            client,
            nonces.clone(),
            dave_acc.into(),
            Currency::Usd,
            min_usd,
            &integration_test_config.sudo,
        ),
        set_balance(
            client,
            nonces.clone(),
            dave_acc.into(),
            Currency::Eq,
            min_eq,
            &integration_test_config.sudo,
        ),
    );

    let zero_balances: HashMap<Currency, SignedBalance<u64>> = [
        (Currency::Eq, SignedBalance::Positive(0)),
        (Currency::Usd, SignedBalance::Positive(0)),
        (Currency::Eos, SignedBalance::Positive(0)),
        (Currency::Btc, SignedBalance::Positive(0)),
        (Currency::Eth, SignedBalance::Positive(0)),
    ]
    .iter()
    .cloned()
    .collect();
    let mut charlie_balances = zero_balances.clone();
    charlie_balances.insert(Currency::Eq, SignedBalance::Positive(min_eq));
    state
        .balances
        .insert(charlie_acc.acc_id(), charlie_balances);
    let mut dave_balances = zero_balances.clone();
    dave_balances.insert(Currency::Btc, SignedBalance::Positive(min_btc));
    dave_balances.insert(Currency::Usd, SignedBalance::Positive(min_usd));
    dave_balances.insert(Currency::Eq, SignedBalance::Positive(min_eq));
    state.balances.insert(dave_acc.acc_id(), dave_balances);
    state
        .balances_aggregates
        .entry(Currency::Btc)
        .and_modify(|bal| bal.total_issuance = bal.total_issuance + min_btc);
    state
        .balances_aggregates
        .entry(Currency::Usd)
        .and_modify(|bal| bal.total_issuance = bal.total_issuance + min_usd);
    state
        .balances_aggregates
        .entry(Currency::Eq)
        .and_modify(|bal| bal.total_issuance = bal.total_issuance + min_eq + min_eq);

    let mut actual_state = TestContext::read_storage(client).await?;
    for val in validators.clone() {
        actual_state.balances.remove(&val.acc_id());
        state.balances.remove(&val.acc_id());
    }
    {
        let store = pub_key_store.lock().await;
        assert_eq_test_data_keystore!(&state, &actual_state, &store);
    }

    println!("Making a transfer");
    let usd_amount = fx64!(0, 5).into_inner() as u64;
    apply_fee(
        &mut state,
        call_chain(
            client,
            nonces.clone(),
            dave_acc,
            TransferCall {
                currency: Currency::Usd,
                to: alice_acc.acc_id(),
                amount: usd_amount,
            },
        )
        .await?,
    );
    let signed_balance = SignedBalance::Positive(usd_amount);
    let dave_balances = state.balances.get_mut(&dave_acc.acc_id()).unwrap();
    dave_balances.insert(Currency::Usd, signed_balance.clone());
    let mut alice_balances = zero_balances.clone();
    alice_balances.insert(Currency::Usd, signed_balance.clone());
    state
        .balances
        .entry(alice_acc.acc_id())
        .and_modify(|bal| {
            bal.entry(Currency::Usd)
                .and_modify(|bal| {
                    println!(
                        "*bal({:?}) = bal({:?}) + signed_balance({:?})",
                        bal.clone() + signed_balance.clone(),
                        bal.clone(),
                        signed_balance.clone()
                    );
                    *bal = bal.clone() + signed_balance.clone()
                })
                .or_insert(signed_balance.clone());
        })
        .or_insert(alice_balances);
    println!("Checking usd balance");
    let mut actual_state = TestContext::read_storage(client).await?;
    for val in validators.clone() {
        actual_state.balances.remove(&val.acc_id());
        state.balances.remove(&val.acc_id());
    }
    {
        let store = pub_key_store.lock().await;
        assert_eq_test_data_keystore!(&state, &actual_state, &store);
    }
    println!("Stop transfer test");
    Ok(())
}
