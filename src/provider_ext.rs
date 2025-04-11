use alloy::{dyn_abi::SolType, network::Network, providers::Provider, sol_types::SolValue};

#[async_trait::async_trait]
pub trait ProviderExt<N: Network>: Provider<N> {
    async fn call_decode<T: SolValue + From<<T::SolType as SolType>::RustType>>(
        &self,
        tx: N::TransactionRequest,
    ) -> anyhow::Result<T> {
        let data = self.call(tx).await?;

        T::abi_decode(&data).map_err(Into::into)
    }
}

impl<N: Network, T: Provider<N>> ProviderExt<N> for T {}
