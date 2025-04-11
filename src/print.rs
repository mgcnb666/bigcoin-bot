use super::{BigcoinAbi, CONTRACT, provider_ext::ProviderExt};
use alloy::{
    network::{Network, NetworkWallet, TransactionBuilder},
    primitives::U256,
    providers::RootProvider,
    sol_types::SolCall,
};
use tokio::task::JoinSet;

pub async fn print<N: Network, W: NetworkWallet<N> + 'static>(
    provider: RootProvider<N>,
    wallets: Vec<W>,
    max_threads: usize,
) {
    let mut join_set = JoinSet::new();
    let mut iter = wallets.into_iter();

    for _ in 0..max_threads {
        if let Some(wallet) = iter.next() {
            let provider = provider.clone();
            join_set.spawn(reward(provider, wallet));
        }
    }

    let mut total = U256::ZERO;
    while let Some(task) = join_set.join_next().await {
        match task.expect("join") {
            Ok(reward) => total += reward,
            Err(e) => println!("{e:?}"),
        }

        if let Some(wallet) = iter.next() {
            let provider = provider.clone();
            join_set.spawn(reward(provider, wallet));
        }
    }

    println!(
        "Total pending rewards: {}",
        total.to::<u128>() as f64 / 1e18
    );
}

pub async fn reward<N: Network, W: NetworkWallet<N>>(
    provider: RootProvider<N>,
    wallet: W,
) -> anyhow::Result<U256> {
    let addr = wallet.default_signer_address();

    let pending_reward = provider
        .call_decode::<U256>(
            N::TransactionRequest::default()
                .with_to(CONTRACT)
                .with_input(BigcoinAbi::pendingRewardsCall { player: addr }.abi_encode()),
        )
        .await?;

    Ok(pending_reward)
}
