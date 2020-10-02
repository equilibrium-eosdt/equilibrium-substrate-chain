use crate::key::{AccountKey, DevNonces, PubKeyStore};
use crate::keyring::PubKey;
use crate::requester::call_chain;
use crate::runtime::{AccountId, EqRuntime};
use crate::test::{apply_fee, delta_balance, init_nonce};
use crate::test_context::{compare_test_data, compare_test_data_keystore, TestContext};
use crate::vesting::{VestCall, VestedTransferCall, VestingStoreExt};
use crate::{assert_eq_test_data, assert_eq_test_data_keystore, IntegrationTestConfig};
use core::marker::PhantomData;
use eq_balances::currency::Currency;
use eq_balances::SignedBalance;
use futures::lock::Mutex;
use sp_runtime::{traits::AccountIdConversion, ModuleId};
use std::collections::HashMap;
use std::sync::Arc;
use substrate_subxt::Client;

pub async fn test_vesting(
    client: &Client<EqRuntime>,
    nonces: Arc<Mutex<DevNonces>>,
    pub_key_store: Arc<Mutex<PubKeyStore>>,
    integration_test_config: IntegrationTestConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Start vesting test");
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

    let min_eq = 9_000_000_000_i128;
    println!("Setting charlie's balances");
    delta_balance(
        client,
        nonces.clone(),
        charlie_acc.into(),
        Currency::Eq,
        min_eq,
        &integration_test_config.sudo,
    )
    .await?;
    delta_balance(
        client,
        nonces.clone(),
        bob_acc.into(),
        Currency::Eq,
        2_000_000_000_i128,
        &integration_test_config.sudo,
    )
    .await?;
    let block_hash = client.block_hash(None).await.unwrap();
    let block = client.block(block_hash).await.unwrap();
    let mut first_block = block.clone().unwrap().block.header.number;

    let shedule = eq_vesting::VestingInfo {
        locked: 8_000_000_000u64,
        per_block: 510_000_000u64,
        starting_block: first_block,
    };
    println!(
        "first_block: {:?}, locked: {:?}",
        first_block, shedule.locked
    );

    let mut state = TestContext::read_storage(client).await?;

    let transferred: u64 = shedule.locked;
    let call_result = call_chain(
        client,
        nonces.clone(),
        charlie_acc,
        VestedTransferCall {
            account_id: bob_acc.acc_id(),
            schedule: shedule,
        },
    )
    .await?;
    let block = client
        .block(Option::Some(call_result[0].block))
        .await
        .unwrap();
    println!(
        "VestedTransferCall block: {:?} ({:?})",
        block.clone().unwrap().block.header.number,
        block.clone().unwrap().block.header.number - first_block
    );
    apply_fee(&mut state, call_result);

    state
        .balances
        .entry(charlie_acc.acc_id())
        .and_modify(|bal| {
            bal.entry(Currency::Eq)
                .and_modify(|eqs| *eqs = eqs.sub_balance(transferred).unwrap());
        });

    let mut total_vested = 0u64;
    let max_blocks = 1 + shedule.locked / shedule.per_block;
    for n in 0..max_blocks {
        let call_result = call_chain(
            client,
            nonces.clone(),
            bob_acc,
            VestCall {
                _runtime: PhantomData,
            },
        )
        .await?;

        let block = client
            .block(Option::Some(call_result[0].block))
            .await
            .unwrap();
        println!(
            "VestCall block: {:?} ({:?})",
            block.clone().unwrap().block.header.number,
            block.clone().unwrap().block.header.number - first_block
        );
        apply_fee(&mut state, call_result);

        let block_hash = client.block_hash(None).await.unwrap();
        let block = client.block(block_hash).await.unwrap();
        let second_block = block.clone().unwrap().block.header.number;
        let vested = (second_block - first_block) * shedule.per_block;
        total_vested = shedule.locked.min(total_vested + vested);
        first_block = second_block;

        println!(
            "block{:?}: {:?}, vested{:?}: {:?}, total_vested: {:?}",
            n, second_block, n, vested, total_vested
        );

        let vesting = client.vesting(bob_acc.acc_id(), Option::None).await?;
        println!("vesting: {:?}", vesting);
        if vesting == None {
            break;
        }
    }
    let bob_sign_balance = SignedBalance::Positive(total_vested);
    let mut bob_balance = zero_balances.clone();
    bob_balance.insert(Currency::Eq, bob_sign_balance.clone());
    state
        .balances
        .entry(bob_acc.acc_id())
        .and_modify(|bal| {
            bal.entry(Currency::Eq)
                .and_modify(|eqs| {
                    println!(
                        "*eqs({:?}) = eqs({:?}) + bob_sign_balance({:?})",
                        eqs.clone() + bob_sign_balance.clone(),
                        eqs.clone(),
                        bob_sign_balance.clone()
                    );
                    *eqs = eqs.clone() + bob_sign_balance.clone()
                })
                .or_insert(bob_sign_balance.clone());
        })
        .or_insert(bob_balance);

    let vestn_acc_id: AccountId = ModuleId(*b"eq/vestn").into_account();
    let vestn_balances = state.balances.get_mut(&vestn_acc_id);
    let mut actual_state = TestContext::read_storage(client).await?;
    actual_state.vesting.remove(&bob_acc.acc_id());
    actual_state.vested.remove(&bob_acc.acc_id());
    for val in validators.clone() {
        actual_state.balances.remove(&val.acc_id());
        state.balances.remove(&val.acc_id());
    }
    {
        let store = pub_key_store.lock().await;
        assert_eq_test_data_keystore!(&state, &actual_state, &store);
    }
    println!("Stop vesting test");
    Ok(())
}
