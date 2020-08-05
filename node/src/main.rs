//! Substrate Node Template CLI library.
#![type_length_limit = "1199286"]
mod chain_spec;
#[macro_use]
mod service;
mod cli;
mod command;
mod rpc;
extern crate jsonrpc_core;

fn main() -> sc_cli::Result<()> {
	command::run()
}
