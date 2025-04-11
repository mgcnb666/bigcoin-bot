use std::str::FromStr;

use alloy::{
    network::{Ethereum, EthereumWallet},
    providers::{Provider, ProviderBuilder, RootProvider},
    signers::local::PrivateKeySigner,
    transports::http::reqwest::Url,
};
use anyhow::{Context, bail};
use bigcoin_cli::{
    CHAIN_ID, add_starter::multi_add_starter, claim::multi_claim, initialize::multi_initialize,
    print::print, transfer::multi_transfer,
};
use clap::Parser;

mod commands;
use commands::{Action, Args};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let Args {
        max_threads,
        action,
        path,
        rpc,
    } = commands::Args::parse();

    let url: Url = rpc.parse().with_context(|| "parse url")?;
    let provider: RootProvider<Ethereum> = ProviderBuilder::default().on_http(url);

    let chain = provider.get_chain_id().await.expect("failed get chain_id");
    if chain != CHAIN_ID {
        bail!(
            "Chain ID mismatch: expected {}, but got {}",
            CHAIN_ID,
            chain
        );
    };

    let file = std::fs::read_to_string(&path).with_context(|| "read keys")?;
    let mut wallets = vec![];
    for (i, line) in file.lines().enumerate() {
        let private_key =
            PrivateKeySigner::from_str(line).with_context(|| format!("failed {} key", i))?;

        wallets.push(EthereumWallet::new(private_key));
    }

    match action {
        Action::Initialize(params) => multi_initialize(provider, wallets, max_threads, params.referrer).await,
        Action::AddStarter(params) => {
            multi_add_starter(provider, wallets, params, max_threads).await
        }
        Action::Claim(params) => multi_claim(provider, wallets, params, max_threads).await,
        Action::Transfer(params) => multi_transfer(provider, wallets, params, max_threads).await,
        Action::Print => print(provider, wallets, max_threads).await,
    };

    Ok(())
}
