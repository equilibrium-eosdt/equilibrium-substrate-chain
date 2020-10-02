use crate::{IntegrationTestConfig, assert_eq_test_data_keystore};
use crate::balances::TransferCall;
use crate::claim::{ClaimCall, MintClaimCall};
use crate::ethkey::EthKey;
use crate::join_chain_calls;
use crate::key::{AccountKey, DevNonces, PubKeyStore};
use crate::keyring::PubKey;
use crate::requester::{call_chain, call_chain_unsigned, sudo_call_chain};
use crate::runtime::{AccountId, EqRuntime};
use crate::test::{apply_fee, init_nonce};
use crate::test_context::{compare_test_data_keystore, TestContext, TestData};
use crate::vesting::VestCall;
use codec::Encode;
use core::marker::PhantomData;
use eq_balances::{currency::Currency, SignedBalance};
use eq_claim::to_ascii_hex;
use eq_integration_testing_macro::tuple_to_vec;
use futures::{lock::Mutex, try_join};
use sp_runtime::traits::AccountIdConversion;
use sp_runtime::ModuleId;
use std::str;
use std::time::SystemTime;
use std::{collections::HashMap, sync::Arc};
use substrate_subxt::Client;

pub async fn print_block_time(client: &Client<EqRuntime>) -> u64 {
    let block_hash = client.block_hash(None).await.unwrap();
    let block = client.block(block_hash).await.unwrap();
    let block_number = block.clone().unwrap().block.header.number;
    let sys_time = SystemTime::now();
    println!("{:?} #{}", sys_time, block_number);
    block_number
}

pub fn sign_text(acc_key: AccountKey) -> String {
    let mut sign_text =
        "\x19Ethereum Signed Message:\n96Pay TEST EQ to the TEST account:".to_owned();
    let acc_id_text = str::from_utf8(&acc_key.acc_id().using_encoded(to_ascii_hex))
        .unwrap()
        .to_owned();
    sign_text.push_str(&acc_id_text);
    sign_text
}

pub fn quote_text(text: String) -> String {
    let mut quote_text = "\"".to_owned();
    let quote = "\"".to_owned();
    quote_text.push_str(&text);
    quote_text.push_str(&quote);
    quote_text
}

pub async fn wait_block(
    client: &Client<EqRuntime>,
    wait_block: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    let block_number = print_block_time(client).await;
    if block_number >= wait_block - 1u64 {
        println!(
            "Not gonna wait {:?}, now is {:?}",
            wait_block - 1u64,
            block_number
        );
        return Ok(());
    }

    let mut subscription = client.subscribe_blocks().await?;
    #[allow(irrefutable_let_patterns)]
    while let next = subscription.next().await {
        if next.number >= wait_block - 1u64 {
            break;
        }
        println!(
            "Simon says to wait {:?} more blocks = {:?} - {:?}",
            wait_block - next.number,
            wait_block,
            next.number
        );
    }
    Ok(())
}

pub async fn set_state_balance_delta(
    mut state: TestData,
    zero_balances: HashMap<Currency, SignedBalance<u64>>,
    acc: AccountKey,
    currency: Currency,
    delta: SignedBalance<u64>,
) {
    let mut balances = zero_balances.clone();
    balances.insert(currency, delta.clone());
    state
        .balances
        .entry(acc.acc_id())
        .and_modify(|bal| {
            bal.entry(Currency::Eq)
                .and_modify(|eqs| {
                    println!(
                        "{:?}] balance({:?}) = balance({:?}) + delta({:?})",
                        acc,
                        eqs.clone() + delta.clone(),
                        eqs.clone(),
                        delta.clone()
                    );
                    *eqs = eqs.clone() + delta.clone()
                })
                .or_insert(delta.clone());
        })
        .or_insert(balances);
}

pub async fn test_claim(
    client: &Client<EqRuntime>,
    nonces: Arc<Mutex<DevNonces>>,
    pub_key_store: Arc<Mutex<PubKeyStore>>,
    integration_test_config: IntegrationTestConfig
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Start claim test");
    let mut state = TestContext::read_storage(client).await?;
    let mut validators = vec![];
    for val in integration_test_config.validators {
        validators.push(AccountKey::from(&val));
    }

    let bob_acc = AccountKey::generate_random();
    init_nonce(client, nonces.clone(), bob_acc).await?;
    let charlie_acc = AccountKey::generate_random();
    init_nonce(client, nonces.clone(), charlie_acc).await?;
    let dave_acc = AccountKey::generate_random();
    init_nonce(client, nonces.clone(), dave_acc).await?;
    let eve_acc = AccountKey::generate_random();
    init_nonce(client, nonces.clone(), eve_acc).await?;
    let ferdie_acc = AccountKey::generate_random();
    init_nonce(client, nonces.clone(), ferdie_acc).await?;

    {
        let mut store = pub_key_store.lock().await;
        store.register(bob_acc.into());
        store.register(charlie_acc.into());
        store.register(dave_acc.into());
        store.register(eve_acc.into());
        store.register(ferdie_acc.into());
    }

    let bob_eth_key = EthKey::generate_random();
    let bob_text = sign_text(bob_acc);
    let bob_signature = bob_eth_key.sign(&bob_text);
    let bob_eth: eq_claim::EthereumAddress =
        serde_json::from_str(&quote_text(bob_eth_key.address())).unwrap();
    let charlie_eth_key = EthKey::generate_random();
    let charlie_eth: eq_claim::EthereumAddress =
        serde_json::from_str(&quote_text(charlie_eth_key.address())).unwrap();
    let ferdie_eth_key = EthKey::generate_random();
    let ferdie_text = sign_text(ferdie_acc);
    let ferdie_signature = ferdie_eth_key.sign(&ferdie_text);
    let ferdie_eth: eq_claim::EthereumAddress =
        serde_json::from_str(&quote_text(ferdie_eth_key.address())).unwrap();
    let dave_eth_key = EthKey::generate_random();
    let dave_signature = dave_eth_key.sign(&sign_text(dave_acc));
    let dave_eth: eq_claim::EthereumAddress =
        serde_json::from_str(&quote_text(dave_eth_key.address())).unwrap();
    let eve_eth_key = EthKey::generate_random();
    let eve_eth: eq_claim::EthereumAddress =
        serde_json::from_str(&quote_text(eve_eth_key.address())).unwrap();

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

    let block_hash = client.block_hash(None).await.unwrap();
    let block = client.block(block_hash).await.unwrap();
    let first_block = block.clone().unwrap().block.header.number;
    //STEP: 1
    let bob_value = 100_000_000_000_000_000u64;
    let bob_initial_transfer = 10_000_000_000_000_000u64;
    let bob_schedule = (
        90_000_000_000_000_000u64,
        6_245_722_108_145u64,
        first_block + 1u64,
    );
    let charlie_value = 100_000_000_000_000_000u64;
    let charlie_schedule = (
        90_000_000_000_000_000u64,
        6_245_722_108_145u64,
        first_block + 1u64,
    );
    let dave_value = 100_000_000_000_000_000u64;
    let dave_initial_transfer = 0u64;
    let dave_schedule = (
        100_000_000_000_000_000u64,
        3_469_845_615_636u64,
        first_block + 1u64,
    );
    let ferdie_value = 50_000_000_000_000_000u64;
    let ferdie_initial_transfer = 5_000_000_000_000_000u64;
    let ferdie_schedule = (
        45_000_000_000_000_000u64,
        1_561_430_527_036u64,
        first_block + 2u64,
    );
    let eve_value = 50_000_000_000_000_000u64;
    let eve_schedule = (
        40_000_000_000_000_000u64,
        1_387_938_246_254u64,
        first_block + 13u64,
    );
    println!("STEP:1 from {:?} start mint claims", bob_schedule.2);

    join_chain_calls!(
        sudo_call_chain(
            client,
            nonces.clone(),
            MintClaimCall {
                who: bob_eth,
                value: bob_value,
                vesting_schedule: Option::Some(bob_schedule),
                statement: false,
            },
            &integration_test_config.sudo
        ),
        sudo_call_chain(
            client,
            nonces.clone(),
            MintClaimCall {
                who: charlie_eth,
                value: charlie_value,
                vesting_schedule: Option::Some(charlie_schedule),
                statement: false,
            },
            &integration_test_config.sudo
        ),
        sudo_call_chain(
            client,
            nonces.clone(),
            MintClaimCall {
                who: dave_eth,
                value: dave_value,
                vesting_schedule: Option::Some(dave_schedule),
                statement: false,
            },
            &integration_test_config.sudo
        ),
        sudo_call_chain(
            client,
            nonces.clone(),
            MintClaimCall {
                who: eve_eth,
                value: eve_value,
                vesting_schedule: Option::Some(eve_schedule),
                statement: false,
            },
            &integration_test_config.sudo
        ),
        sudo_call_chain(
            client,
            nonces.clone(),
            MintClaimCall {
                who: ferdie_eth,
                value: ferdie_value,
                vesting_schedule: Option::Some(ferdie_schedule),
                statement: false,
            },
            &integration_test_config.sudo
        )
    );
    println!("mint claims done");

    state.total = state.total + bob_value + charlie_value + dave_value + eve_value + ferdie_value;
    state.claims.insert(bob_eth, bob_value);
    state.claims.insert(charlie_eth, charlie_value);
    state.claims.insert(dave_eth, dave_value);
    state.claims.insert(eve_eth, eve_value);
    state.claims.insert(ferdie_eth, ferdie_value);
    state.claim_vesting.insert(bob_eth, bob_schedule);
    state.claim_vesting.insert(charlie_eth, charlie_schedule);
    state.claim_vesting.insert(dave_eth, dave_schedule);
    state.claim_vesting.insert(eve_eth, eve_schedule);
    state.claim_vesting.insert(ferdie_eth, ferdie_schedule);

    let mut actual_state = TestContext::read_storage(client).await?;
    for val in validators.clone() {
        actual_state.balances.remove(&val.acc_id());
        state.balances.remove(&val.acc_id());
    }
    {
        let store = pub_key_store.lock().await;
        assert_eq_test_data_keystore!(&state, &actual_state, &store);
    }

    //STEP:2
    println!("STEP:2 waiting");
    wait_block(client, bob_schedule.2 + 2u64).await?;
    println!("STEP:2 from {:?} start first claim", bob_schedule.2 + 2u64);

    let call_result = call_chain_unsigned(
        client,
        ClaimCall {
            dest: bob_acc.acc_id(),
            ethereum_signature: bob_signature.clone(),
        },
    )
    .await?;
    let block = client
        .block(Option::Some(call_result[0].block))
        .await
        .unwrap();
    println!(
        "block step 2: {:?} ({:?})",
        block.clone().unwrap().block.header.number,
        block.clone().unwrap().block.header.number - bob_schedule.2
    );
    apply_fee(&mut state, call_result);
    let d9 = block.clone().unwrap().block.header.number - bob_schedule.2;
    let d15 = bob_initial_transfer + bob_schedule.1 * d9;
    let d17 = bob_initial_transfer + bob_schedule.0 - d15;
    println!("d9: {:?}, d15: {:?}, d17: {:?}.", d9, d15, d17);

    let vestn_acc_id: AccountId = ModuleId(*b"eq/vestn").into_account();
    let mut vst_balance = zero_balances.clone();
    let vst_sign_balance = SignedBalance::Positive(d17);
    vst_balance.insert(Currency::Eq, vst_sign_balance.clone());
    state
        .balances
        .entry(vestn_acc_id.clone())
        .and_modify(|bal| {
            bal.entry(Currency::Eq)
                .and_modify(|eqs| *eqs = eqs.clone() + vst_sign_balance.clone())
                .or_insert(vst_sign_balance.clone());
        })
        .or_insert(vst_balance);
    let bob_sign_balance = SignedBalance::Positive(d15);
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
    state
        .balances_aggregates
        .entry(Currency::Eq)
        .and_modify(|eqs| eqs.total_issuance = eqs.total_issuance + bob_value);
    state.claims.remove(&bob_eth);
    state.total = state.total - bob_value;
    state.claim_vesting.remove(&bob_eth);

    let mut actual_state = TestContext::read_storage(client).await?;
    for val in validators.clone() {
        actual_state.balances.remove(&val.acc_id());
        state.balances.remove(&val.acc_id());
    }
    {
        let store = pub_key_store.lock().await;
        assert_eq_test_data_keystore!(&state, &actual_state, &store);
    }

    //STEP:3
    println!("STEP:3 waiting");
    wait_block(client, bob_schedule.2 + 5u64).await?;
    println!("STEP:3 ferdie");

    let call_result = call_chain_unsigned(
        client,
        ClaimCall {
            dest: ferdie_acc.acc_id(),
            ethereum_signature: ferdie_signature.clone(),
        },
    )
    .await?;
    let block = client
        .block(Option::Some(call_result[0].block))
        .await
        .unwrap();
    println!(
        "block step 3: {:?} ({:?})",
        block.clone().unwrap().block.header.number,
        block.clone().unwrap().block.header.number - bob_schedule.2
    );
    apply_fee(&mut state, call_result);
    let d24 = block.clone().unwrap().block.header.number - ferdie_schedule.2;
    let d33 = ferdie_initial_transfer + ferdie_schedule.1 * (d24);
    let d35 = d15;
    let d37 = d17 + ferdie_initial_transfer + ferdie_schedule.0 - d33;
    let d38 = d37 - d17;
    println!(
        "d24: {:?},d33: {:?}, d37: {:?}, d38: {:?}.",
        d24, d33, d37, d38
    );

    let vst_sign_balance = SignedBalance::Positive(d38);
    state
        .balances
        .entry(vestn_acc_id.clone())
        .and_modify(|bal| {
            bal.entry(Currency::Eq)
                .and_modify(|eqs| *eqs = eqs.clone() + vst_sign_balance.clone());
        });
    let ferdie_sign_balance = SignedBalance::Positive(d33);
    let mut ferdie_balance = zero_balances.clone();
    ferdie_balance.insert(Currency::Eq, ferdie_sign_balance.clone());
    state
        .balances
        .entry(ferdie_acc.acc_id())
        .and_modify(|bal| {
            bal.entry(Currency::Eq)
                .and_modify(|eqs| *eqs = eqs.clone() + ferdie_sign_balance.clone())
                .or_insert(ferdie_sign_balance.clone());
        })
        .or_insert(ferdie_balance);
    state
        .balances_aggregates
        .entry(Currency::Eq)
        .and_modify(|eqs| eqs.total_issuance = eqs.total_issuance + ferdie_value);
    state.claims.remove(&ferdie_eth);
    state.total = state.total - ferdie_value;
    state.claim_vesting.remove(&ferdie_eth);

    let mut actual_state = TestContext::read_storage(client).await?;
    for val in validators.clone() {
        actual_state.balances.remove(&val.acc_id());
        state.balances.remove(&val.acc_id());
    }
    {
        let store = pub_key_store.lock().await;
        assert_eq_test_data_keystore!(&state, &actual_state, &store);
    }

    //STEP 3.2
    wait_block(client, bob_schedule.2 + 7u64).await?;
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
        "block step 3.2: {:?} ({:?})",
        block.clone().unwrap().block.header.number,
        block.clone().unwrap().block.header.number - bob_schedule.2
    );
    apply_fee(&mut state, call_result);
    let passed_blocks = block.clone().unwrap().block.header.number - bob_schedule.2;
    let d44 = d15 + bob_schedule.1 * (passed_blocks - d9);
    let d45 = d44 - d15;
    let d46 = d37 + d35 - d44;
    println!(
        "passed_blocks: {:?},d15: {:?},d44: {:?}, d45: {:?}, d46: {:?}.",
        passed_blocks, d15, d44, d45, d46
    );

    let vst_sign_balance = SignedBalance::Negative(d45);
    state
        .balances
        .entry(vestn_acc_id.clone())
        .and_modify(|bal| {
            bal.entry(Currency::Eq)
                .and_modify(|eqs| *eqs = eqs.clone() + vst_sign_balance.clone());
        });

    let bob_sign_balance = SignedBalance::Positive(d45);
    state.balances.entry(bob_acc.acc_id()).and_modify(|bal| {
        bal.entry(Currency::Eq).and_modify(|eqs| {
            println!(
                "*eqs({:?}) = eqs({:?}) + bob_sign_balance({:?})",
                eqs.clone() + bob_sign_balance.clone(),
                eqs.clone(),
                bob_sign_balance.clone()
            );
            *eqs = eqs.clone() + bob_sign_balance.clone()
        });
    });

    let mut actual_state = TestContext::read_storage(client).await?;
    for val in validators.clone() {
        actual_state.balances.remove(&val.acc_id());
        state.balances.remove(&val.acc_id());
    }
    {
        let store = pub_key_store.lock().await;
        assert_eq_test_data_keystore!(&state, &actual_state, &store);
    }

    //STEP:4
    wait_block(client, bob_schedule.2 + 10u64).await?;

    let call_result = call_chain_unsigned(
        client,
        ClaimCall {
            dest: dave_acc.acc_id(),
            ethereum_signature: dave_signature.clone(),
        },
    )
    .await?;

    let block = client
        .block(Option::Some(call_result[0].block))
        .await
        .unwrap();
    println!(
        "block step 4: {:?} ({:?})",
        block.clone().unwrap().block.header.number,
        block.clone().unwrap().block.header.number - bob_schedule.2
    );
    apply_fee(&mut state, call_result);
    let passed_blocks = block.clone().unwrap().block.header.number - bob_schedule.2;
    let d57 = dave_initial_transfer + (dave_schedule.1 * passed_blocks + 1);
    let d58 = d57;
    let d59 = d46 + dave_schedule.0 - d57;
    let d60 = d59 - d46;
    println!(
        "passed_blocks: {:?},d58: {:?},d59: {:?}, d60: {:?}.",
        passed_blocks, d58, d59, d60
    );

    let vst_sign_balance = SignedBalance::Positive(d60);
    state
        .balances
        .entry(vestn_acc_id.clone())
        .and_modify(|bal| {
            bal.entry(Currency::Eq)
                .and_modify(|eqs| *eqs = eqs.clone() + vst_sign_balance.clone());
        });
    let dave_sign_balance = SignedBalance::Positive(d57);
    let mut dave_balance = zero_balances.clone();
    dave_balance.insert(Currency::Eq, dave_sign_balance.clone());
    state
        .balances
        .entry(dave_acc.acc_id())
        .and_modify(|bal| {
            bal.entry(Currency::Eq)
                .and_modify(|eqs| *eqs = eqs.clone() + dave_sign_balance.clone())
                .or_insert(dave_sign_balance.clone());
        })
        .or_insert(dave_balance);
    state
        .balances_aggregates
        .entry(Currency::Eq)
        .and_modify(|eqs| eqs.total_issuance = eqs.total_issuance + dave_value);
    state.claims.remove(&dave_eth);
    state.total = state.total - dave_value;
    state.claim_vesting.remove(&dave_eth);

    let mut actual_state = TestContext::read_storage(client).await?;
    for val in validators.clone() {
        actual_state.balances.remove(&val.acc_id());
        state.balances.remove(&val.acc_id());
    }
    {
        let store = pub_key_store.lock().await;
        assert_eq_test_data_keystore!(&state, &actual_state, &store);
    }

    //STEP:4.2
    //revert trx
    /*let call_result_err = call_chain(
        client,
        nonces.clone(),
        bob_acc,
        VestedTransferCall {
            account_id: eve_acc.acc_id(),
            schedule: eq_vesting::VestingInfo{ locked:eve_schedule.0, per_block:eve_schedule.1, starting_block: eve_schedule.2},
        },
    )
    .await;
    assert!(
        call_result_err.is_err(),
        "{}",
        call_result_err.unwrap_err()
    );*/
    // revert trx

    //STEP:5
    wait_block(client, bob_schedule.2 + 12u64).await?;
    let transfer_amount = 2_000_000_000_000u64;
    let call_result = call_chain(
        client,
        nonces.clone(),
        bob_acc,
        TransferCall {
            currency: Currency::Eq,
            to: dave_acc.acc_id(),
            amount: transfer_amount,
        },
    )
    .await?;

    let block = client
        .block(Option::Some(call_result[0].block))
        .await
        .unwrap();
    println!(
        "block step 4: {:?}",
        block.clone().unwrap().block.header.number
    );
    apply_fee(&mut state, call_result);

    let bob_sign_balance = SignedBalance::Negative(transfer_amount);
    state.balances.entry(bob_acc.acc_id()).and_modify(|bal| {
        bal.entry(Currency::Eq).and_modify(|eqs| {
            println!(
                "bob balance *eqs({:?}) = eqs({:?}) - bob_sign_balance({:?})",
                eqs.clone() + bob_sign_balance.clone(),
                eqs.clone(),
                bob_sign_balance.clone()
            );
            *eqs = eqs.clone() + bob_sign_balance.clone()
        });
    });
    let dave_sign_balance = SignedBalance::Positive(transfer_amount);
    state.balances.entry(dave_acc.acc_id()).and_modify(|bal| {
        bal.entry(Currency::Eq)
            .and_modify(|eqs| *eqs = eqs.clone() + dave_sign_balance.clone());
    });

    let mut actual_state = TestContext::read_storage(client).await?;
    for val in validators.clone() {
        actual_state.balances.remove(&val.acc_id());
        state.balances.remove(&val.acc_id());
    }
    {
        let store = pub_key_store.lock().await;
        assert_eq_test_data_keystore!(&state, &actual_state, &store);
    }

    println!("Stop claim test");
    Ok(())
}
