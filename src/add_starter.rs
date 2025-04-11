use super::{BigcoinAbi, CHAIN_ID, CONTRACT, provider_ext::ProviderExt};
use alloy::{
    eips::eip1559::Eip1559Estimation,
    network::{Network, NetworkWallet, TransactionBuilder},
    primitives::U256,
    providers::{Provider, RootProvider},
    sol_types::SolCall,
};
use clap::Parser;
use tokio::task::JoinSet;

#[derive(Debug, Clone, Copy, Parser)]
pub struct AddStarterParams {
    #[arg(long, default_value_t = 0)]
    pub x: u8,
    #[arg(long, default_value_t = 0)]
    pub y: u8,
}

pub async fn multi_add_starter<N: Network, W: NetworkWallet<N> + 'static>(
    provider: RootProvider<N>,
    wallets: Vec<W>,
    params: AddStarterParams,
    max_threads: usize,
) {
    let mut join_set = JoinSet::new();
    let mut iter = wallets.into_iter();

    for _ in 0..max_threads {
        if let Some(wallet) = iter.next() {
            let provider = provider.clone();
            join_set.spawn(add_starter(provider, wallet, params));
        }
    }

    while let Some(task) = join_set.join_next().await {
        if let Err(e) = task.expect("join") {
            println!("{e:?}");
        }

        if let Some(wallet) = iter.next() {
            let provider = provider.clone();
            join_set.spawn(add_starter(provider, wallet, params));
        }
    }
}

pub async fn add_starter<N: Network, W: NetworkWallet<N>>(
    provider: RootProvider<N>,
    wallet: W,
    AddStarterParams { x, y }: AddStarterParams,
) -> anyhow::Result<()> {
    let addr = wallet.default_signer_address();

    let initialized = provider
        .call_decode::<bool>(
            N::TransactionRequest::default()
                .with_to(CONTRACT)
                .with_input(BigcoinAbi::initializedStarterFacilityCall(addr).abi_encode()),
        )
        .await?;

    if !initialized {
        println!("[{addr}] not initialized");

        return Ok(());
    }

    let with_starter_miner = provider
        .call_decode::<bool>(
            N::TransactionRequest::default()
                .with_to(CONTRACT)
                .with_input(BigcoinAbi::acquiredStarterMinerCall(addr).abi_encode()),
        )
        .await?;

    if with_starter_miner {
        println!("[{addr}] already with starter miner");

        return Ok(());
    }

    println!("[{addr}] processing...");
    let nonce = provider.get_transaction_count(addr).await?;
    let Eip1559Estimation {
        max_fee_per_gas,
        max_priority_fee_per_gas,
    } = provider.estimate_eip1559_fees().await?;

    let mut tx = N::TransactionRequest::default()
        .with_from(addr)
        .with_to(CONTRACT)
        .with_chain_id(CHAIN_ID)
        .with_max_fee_per_gas(max_fee_per_gas)
        .with_max_priority_fee_per_gas(max_priority_fee_per_gas)
        .with_nonce(nonce)
        .with_input(
            BigcoinAbi::getFreeStarterMinerCall {
                x: U256::from(x),
                y: U256::from(y),
            }
            .abi_encode(),
        );

    let gas_limit = provider.estimate_gas(tx.clone()).await?;
    tx.set_gas_limit(gas_limit);

    let fee = U256::from(max_fee_per_gas + max_priority_fee_per_gas) * U256::from(gas_limit);
    let balance = provider.get_balance(addr).await?;

    if fee > balance {
        println!("[{addr}] no enough balance to pay fee: {}", fee);

        return Ok(());
    }

    let raw = tx.build(&wallet).await?;
    let tx_hash = *provider.send_tx_envelope(raw).await?.tx_hash();
    println!("[{addr}] transaction sent: {tx_hash}");

    Ok(())
}
