use super::EqRuntime;
use crate::key::{AccountKey, DevNonces};
use crate::keyring::{KeyPair, NonceManager};
use crate::runtime::{AccountId, Balance};
use core::marker::PhantomData;
use futures::lock::Mutex;
use pallet_transaction_payment_rpc_runtime_api::RuntimeDispatchInfo;
use sp_keyring::AccountKeyring;
use std::collections::HashMap;
use std::sync::Arc;
use substrate_subxt::ExtrinsicSuccessWithFee;
use substrate_subxt::{sudo::SudoCall, Call, Client, PairSigner};

lazy_static! {
    pub static ref FEES: HashMap<String, u64> = vec![
        (String::from("transfer_and_watch"), 125_000_146),
        (String::from("register_bailsman_and_watch"), 125_000_105),
        (String::from("unregister_bailsman_and_watch"), 125_000_105),
        (String::from("register_whitelist_and_watch"), 125_000_139),
        (String::from("unregister_whitelist_and_watch"), 125_000_139),
        (String::from("set_price_and_watch"), 125_000_114),
    ]
    .into_iter()
    .collect();
}

pub struct Requester {
    pub client: Client<super::EqRuntime>,
    nonces: HashMap<super::runtime::AccountId, u32>,
    pub sudo: String,
}

impl Requester {
    pub fn new(client: Client<EqRuntime>) -> Self {
        Requester {
            client,
            nonces: HashMap::new(),
            sudo: "".to_string(),
        }
    }

    pub async fn with_sudo<C: Call<EqRuntime>>(
        &mut self,
        call: C,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let alice = AccountKeyring::Alice;
        let mut signer = PairSigner::new(alice.pair());
        let sudo_nonce = self
            .increment_nonce(AccountKeyring::Alice.to_account_id())
            .await?;

        signer.set_nonce(sudo_nonce);
        let encoded = self.client.encode(call)?;
        let sudo = SudoCall {
            call: &encoded,
            _runtime: PhantomData,
        };
        let extrinsic = self.client.watch(sudo, &signer).await?;
        println!("with_sudo extrinsic {:?}", extrinsic);
        Ok(())
    }

    pub async fn increment_nonce(
        &mut self,
        who: AccountId,
    ) -> Result<u32, Box<dyn std::error::Error>> {
        let nonce = self
            .nonces
            .entry(who.clone())
            .and_modify(|a| *a += 1)
            .or_insert(
                self.client
                    .fetch_or_default(
                        &substrate_subxt::system::AccountStore { account_id: &who },
                        None,
                    )
                    .await
                    .unwrap()
                    .nonce,
            );
        println!("new nonce is {}", nonce);
        Ok(*nonce)
    }
}

pub type ChainCallSuccess = ExtrinsicSuccessWithFee<EqRuntime, (AccountKey, Balance)>;
pub type ChainCallResult = Result<Vec<ChainCallSuccess>, Box<dyn std::error::Error>>;

pub async fn call_chain<C: Call<EqRuntime> + Send + Sync>(
    client: &Client<EqRuntime>,
    nonces: Arc<Mutex<DevNonces>>,
    account_key: AccountKey,
    call: C,
) -> Result<Vec<ChainCallSuccess>, Box<dyn std::error::Error>> {
    let nonce = { nonces.lock().await.get_nonce_and_inc(account_key) };
    println!("Current nonce for {:?} is {}", account_key, nonce);

    let mut signer = PairSigner::new(account_key.key_pair());
    signer.set_nonce(nonce);
    let extrinsic = client.create_signed(call, &signer).await?;
    let decoder = EqRuntime::create_decoder(client.metadata().clone());
    let success = client
        .submit_and_watch_extrinsic_with_fee::<Balance>(extrinsic, decoder)
        .await?;
    let fee = success
        .dispatch_info
        .as_ref()
        .map(|x| x.partial_fee)
        .unwrap_or(0);

    println!("incoming fee for {:?}: {}", account_key, fee);

    Ok(vec![ExtrinsicSuccessWithFee {
        block: success.block,
        extrinsic: success.extrinsic,
        events: success.events,
        dispatch_info: success.dispatch_info.map(|x| RuntimeDispatchInfo {
            weight: x.weight,
            class: x.class,
            partial_fee: (account_key, x.partial_fee),
        }),
    }])
}

pub async fn sudo_call_chain<C: Call<EqRuntime> + Send + Sync>(
    client: &Client<EqRuntime>,
    nonces: Arc<Mutex<DevNonces>>,
    call: C,
    sudo: &String,
) -> Result<Vec<ChainCallSuccess>, Box<dyn std::error::Error>> {
    let encoded = client.encode(call)?;
    let sudo_call = SudoCall {
        call: &encoded,
        _runtime: PhantomData,
    };
    let sudo_key = AccountKey::from(sudo);

    call_chain(client, nonces, sudo_key.into(), sudo_call).await
}

pub async fn call_chain_unsigned<C: Call<EqRuntime> + Send + Sync>(
    client: &Client<EqRuntime>,
    call: C,
) -> Result<Vec<ChainCallSuccess>, Box<dyn std::error::Error>> {
    let extrinsic = client.create_unsigned(call)?;
    let decoder = EqRuntime::create_decoder(client.metadata().clone());
    let success = client
        .submit_and_watch_extrinsic(extrinsic, decoder)
        .await?;

    Ok(vec![ExtrinsicSuccessWithFee {
        block: success.block,
        extrinsic: success.extrinsic,
        events: success.events,
        dispatch_info: Option::None,
    }])
}

#[macro_export]
macro_rules! join_chain_calls {
    ( $($tokens:tt)* ) => {{
        let join_result = try_join!($($tokens)*)?;

        let vec_vec = tuple_to_vec!(join_result, $($tokens)*);

        let flatten: Vec<_> = vec_vec.into_iter().flatten().collect();

        flatten
    }};
}

#[macro_export]
macro_rules! call_chain_old {
    ($requester:ident.$call:ident($who:expr, $test_storage:expr, $($args:expr),*) ) => {async {
        let account_id = $who.to_account_id();
        let nonce = {
            let mut locked_requester = $requester.lock().await;
            locked_requester.increment_nonce(account_id.clone()).await.unwrap()
        };
        let mut pair = PairSigner::new($who.pair());
        pair.set_nonce(nonce);
        let fee = FEES.get(stringify!($call));
        if let Some(fee_value) = fee {
            {
                let mut locked_storage = $test_storage.lock().await;
                locked_storage
                    .Balances
                    .entry(account_id)
                    .and_modify(|bal| {
                        bal
                            .entry(Currency::Eq)
                            .and_modify(|eqs| *eqs = eqs.sub_balance(*fee_value).unwrap());
                    } );
                let old_total_inssuance = locked_storage
                    .BalancesAggregates
                    .get(&Currency::Eq)
                    .unwrap()
                    .total_issuance;
                locked_storage
                    .BalancesAggregates
                    .entry(Currency::Eq)
                    .and_modify(|eq| {eq.total_issuance = old_total_inssuance - fee_value;});
            }
        } else {println!("No fee found for action {}", stringify!($call));}
        $requester.lock().await.client.$call(&pair, $($args),*).await.unwrap()
    }};
    ($requester:ident.$call:ident($who:expr,  $test_storage:expr )) => { async {
        let account_id = $who.to_account_id();
        let nonce = {
            let mut locked_requester = $requester.lock().await;
            locked_requester.increment_nonce(account_id.clone()).await.unwrap()
        };
        let mut pair = PairSigner::new($who.pair());
        pair.set_nonce(nonce);
        let fee = FEES.get(stringify!($call));
        if let Some(fee_value) = fee {
            {
                let mut locked_storage = $test_storage.lock().await;
                locked_storage
                    .Balances
                    .entry(account_id)
                    .and_modify(|bal| {
                        bal
                            .entry(Currency::Eq)
                            .and_modify(|eqs| *eqs = eqs.sub_balance(*fee_value).unwrap());
                    } );
                let old_total_inssuance = locked_storage
                    .BalancesAggregates
                    .get(&Currency::Eq)
                    .unwrap()
                    .total_issuance;
                locked_storage
                    .BalancesAggregates
                    .entry(Currency::Eq)
                    .and_modify(|eq| {eq.total_issuance = old_total_inssuance - fee_value;});
            }
        } else {println!("No fee found for action {}", stringify!($call));}
        let extrinsic = $requester.lock().await.client.$call(&pair).await.unwrap();
        println!("{:?}", extrinsic);
    } };
}
