#![recursion_limit = "256"]

pub mod balances;
pub mod claim;
pub mod ethkey;
pub mod key;
pub mod keyring;
pub mod rate;
pub mod requester;
pub mod runtime;
pub mod test;
pub mod test_context;
pub mod test_scripts;
pub mod timestamp;
pub mod vesting;
pub mod whitelists;

use crate::key::{AccountKey, DevNonces, PubKeyStore};
use crate::keyring::PubKey;
use crate::test::init_nonce;
use crate::test_context::register_common_pub_keys;
use futures::lock::Mutex;
use log::{Level, Metadata, Record};
use log::{LevelFilter, SetLoggerError};
use runtime::EqRuntime;
use serde::{Deserialize, Serialize};
use sp_keyring::AccountKeyring;
use std::fs::File;
use std::io::BufReader;
use std::sync::Arc;
use substrate_subxt::ClientBuilder;
use test_scripts::{
    test_claim::test_claim, test_transfer::test_transfer, test_vesting::test_vesting,
};

#[macro_use]
extern crate lazy_static;

pub const ONE: u64 = 1_000_000_000;

struct SimpleLogger;
impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            println!("{} - {}", record.level(), record.args());
        }
    }

    fn flush(&self) {}
}

static LOGGER: SimpleLogger = SimpleLogger;
pub fn init() -> Result<(), SetLoggerError> {
    log::set_logger(&LOGGER).map(|()| log::set_max_level(LevelFilter::Info))
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IntegrationTestConfig {
    local: bool,
    sudo: String,
    endpoint: String,
    tests_to_launch: Vec<String>,
    validators: Vec<String>,
}

pub fn read_integration_test_config() -> IntegrationTestConfig {
    let file = File::open("integration_test.json").unwrap();
    let reader = BufReader::new(file);
    let conf: IntegrationTestConfig =
        serde_json::from_reader(reader).expect("JSON was not well-formatted");
    println!("{:?}", conf);
    conf
}

#[async_std::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = read_integration_test_config();
    let client = ClientBuilder::<EqRuntime>::new()
        .set_url(config.clone().endpoint)
        .set_page_size(1)
        .build()
        .await?;

    let nonces = Arc::new(Mutex::new(DevNonces::new()));
    let pub_key_store = Arc::new(Mutex::new(PubKeyStore::new()));
    {
        let mut store = pub_key_store.lock().await;
        register_common_pub_keys(&mut store);
    }

    let alice_acc = AccountKey::from(AccountKeyring::Alice);
    println!("alice {:?}", alice_acc.acc_id());
    init_nonce(&client, nonces.clone(), alice_acc).await?;

    let sudo_acc = AccountKey::from(&config.sudo);
    println!("sudo {:?}", sudo_acc.acc_id());
    init_nonce(&client, nonces.clone(), sudo_acc).await?;

    if config
        .tests_to_launch
        .contains(&"test_transfer".to_string())
    {
        println!("\n\nRunning transfer test \n\n");
        test_transfer(
            &client,
            nonces.clone(),
            pub_key_store.clone(),
            config.clone(),
        )
        .await?;
    }

    if config.tests_to_launch.contains(&"test_claim".to_string()) {
        println!("\n\nRunning claim test \n\n");
        test_claim(
            &client,
            nonces.clone(),
            pub_key_store.clone(),
            config.clone(),
        )
        .await?;
    }

    if config.tests_to_launch.contains(&"test_vesting".to_string()) {
        println!("\n\nRunning vesting test \n\n");
        test_vesting(
            &client,
            nonces.clone(),
            pub_key_store.clone(),
            config.clone(),
        )
        .await?;
    }

    Ok(())
}
