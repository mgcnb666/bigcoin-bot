use super::{CHAIN_ID, TOKEN, provider_ext::ProviderExt};
use alloy::{
    eips::eip1559::Eip1559Estimation,
    network::{Network, NetworkWallet, TransactionBuilder},
    primitives::{Address, U256, utils::parse_ether},
    providers::{Provider, RootProvider},
    sol,
    sol_types::SolCall,
};
use clap::Parser;
use tokio::task::JoinSet;

#[derive(Debug, Clone, Copy, Parser)]
pub struct TransferParams {
    /// Recipient address for tokens 
    #[clap(short, long)]
    pub receiver: Address,

    /// Minimum amount to transfer, e.g. "0.01"
    #[clap(short, long, value_parser = parse_ether)]
    pub min_transfer_amount: U256,
}

pub async fn multi_transfer<N: Network, W: NetworkWallet<N> + 'static>(
    provider: RootProvider<N>,
    wallets: Vec<W>,
    params: TransferParams,
    max_threads: usize,
) {
    let mut join_set = JoinSet::new();
    let mut iter = wallets.into_iter();

    for _ in 0..max_threads {
        if let Some(wallet) = iter.next() {
            let provider = provider.clone();
            join_set.spawn(transfer(provider, wallet, params));
        }
    }

    while let Some(task) = join_set.join_next().await {
        if let Err(e) = task.expect("join") {
            println!("{e:?}");
        }

        if let Some(wallet) = iter.next() {
            let provider = provider.clone();
            join_set.spawn(transfer(provider, wallet, params));
        }
    }
}

pub async fn transfer<N: Network, W: NetworkWallet<N>>(
    provider: RootProvider<N>,
    wallet: W,
    TransferParams {
        receiver,
        min_transfer_amount,
    }: TransferParams,
) -> anyhow::Result<()> {
    let addr = wallet.default_signer_address();

    let balance = provider
        .call_decode::<U256>(
            N::TransactionRequest::default()
                .with_to(TOKEN)
                .with_input(ERC20::balanceOfCall(addr).abi_encode()),
        )
        .await?;

    if min_transfer_amount > balance {
        println!("[{addr}] balance too low: {}", balance);

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
        .with_to(TOKEN)
        .with_chain_id(CHAIN_ID)
        .with_max_fee_per_gas(max_fee_per_gas)
        .with_max_priority_fee_per_gas(max_priority_fee_per_gas)
        .with_nonce(nonce)
        .with_input(
            ERC20::transferCall {
                _0: receiver,
                _1: balance,
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

sol! {
    interface ERC20 {
        function balanceOf(address) external view returns (uint256);
        function transfer(address, uint256) external returns (bool);
    }
}
