use crate::network::Network;
use ethers::types::Address;
use std::{
    clone::Clone,
    collections::HashMap,
    sync::{Arc, RwLock},
};

#[derive(Debug)]
pub struct AbiRegistry<C> {
    pub url: Option<String>,
    pub network: Option<Network>,
    pub registry: Arc<RwLock<HashMap<Address, C>>>,
}

impl<C> AbiRegistry<C> {
    pub fn new(url: Option<String>, network: Option<Network>) -> Self {
        Self {
            url,
            network,
            registry: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn entry_exists(&self, address: Address) -> bool {
        let arc_clone = Arc::clone(&self.registry);
        let registry = arc_clone.read().expect("Registry RwLock poisoned!");
        let entry_exists = registry.contains_key(&address);
        drop(registry);

        entry_exists
    }

    pub fn add_entry(&self, address: Address, contract: C) {
        let arc_clone = Arc::clone(&self.registry);
        let mut registry = arc_clone.write().expect("Registry RwLock poisoned!");
        registry.insert(address, contract);
        drop(registry);
    }
}

#[macro_export]
macro_rules! abirpc {
    ($abi:ident, $abi_registry: ident) => {
        #[derive(Debug)]
        pub struct $abi_registry<M>($crate::registry::AbiRegistry<$abi<M>>)
        where
            M: ::ethers::prelude::Middleware;

        #[async_trait::async_trait]
        impl $crate::provider::AbiProviderTrait<::ethers::prelude::Provider<::ethers::prelude::Ws>>
            for $abi_registry<::ethers::prelude::Provider<::ethers::prelude::Ws>>
        {
            async fn provider(
                &self,
            ) -> Result<::ethers::prelude::Provider<::ethers::prelude::Ws>, $crate::error::Error>
            {
                let provider: ::ethers::prelude::Provider<::ethers::prelude::Ws> =
                    $crate::provider::AbiProvider::new(self.0.url.clone(), self.0.network)
                        .provider()
                        .await?;

                Ok(provider)
            }
        }

        #[async_trait::async_trait]
        impl
            $crate::provider::AbiProviderTrait<::ethers::prelude::Provider<::ethers::prelude::Http>>
            for $abi_registry<::ethers::prelude::Provider<::ethers::prelude::Http>>
        {
            async fn provider(
                &self,
            ) -> Result<::ethers::prelude::Provider<::ethers::prelude::Http>, $crate::error::Error>
            {
                let provider: ::ethers::prelude::Provider<::ethers::prelude::Http> =
                    $crate::provider::AbiProvider::new(self.0.url.clone(), self.0.network)
                        .provider()
                        .await?;

                Ok(provider)
            }
        }

        #[async_trait::async_trait]
        impl
            $crate::provider::AbiProviderTrait<
                ::ethers::prelude::Provider<::ethers::prelude::RetryClient<Http>>,
            > for $abi_registry<::ethers::prelude::Provider<::ethers::prelude::RetryClient<Http>>>
        {
            async fn provider(
                &self,
            ) -> Result<
                ::ethers::prelude::Provider<::ethers::prelude::RetryClient<Http>>,
                $crate::error::Error,
            > {
                let provider: ::ethers::prelude::Provider<::ethers::prelude::RetryClient<Http>> =
                    $crate::provider::AbiProvider::new(self.0.url.clone(), self.0.network)
                        .provider()
                        .await?;

                Ok(provider)
            }
        }

        #[async_trait::async_trait]
        impl $crate::provider::AbiProviderTrait<::ethers::prelude::Provider<::ethers::prelude::Ipc>>
            for $abi_registry<::ethers::prelude::Provider<::ethers::prelude::Ipc>>
        {
            async fn provider(
                &self,
            ) -> Result<::ethers::prelude::Provider<::ethers::prelude::Ipc>, $crate::error::Error>
            {
                let provider: ::ethers::prelude::Provider<::ethers::prelude::Ipc> =
                    $crate::provider::AbiProvider::new(self.0.url.clone(), self.0.network)
                        .provider()
                        .await?;

                Ok(provider)
            }
        }

        #[async_trait::async_trait]
        impl
            $crate::provider::AbiProviderTrait<
                ::ethers::prelude::Provider<::ethers::prelude::MockProvider>,
            > for $abi_registry<::ethers::prelude::Provider<::ethers::prelude::MockProvider>>
        {
            async fn provider(
                &self,
            ) -> Result<
                ::ethers::prelude::Provider<::ethers::prelude::MockProvider>,
                $crate::error::Error,
            > {
                let provider: ::ethers::prelude::Provider<::ethers::prelude::MockProvider> =
                    $crate::provider::AbiProvider::new(self.0.url.clone(), self.0.network)
                        .provider()
                        .await?;

                Ok(provider)
            }
        }

        impl<M> $abi_registry<M>
        where
            M: ::ethers::prelude::Middleware,
        {
            pub fn new(url: Option<String>, network: Option<$crate::network::Network>) -> Self {
                let registry = $crate::registry::AbiRegistry::<$abi<M>>::new(url, network);
                Self(registry)
            }

            pub fn register(&self, provider: M, address: ::ethers::prelude::Address) -> $abi<M> {
                if !self.0.entry_exists(address) {
                    let instance = $abi::new(address, provider.into());
                    self.0.add_entry(address, instance)
                }

                let clone_lock = std::sync::Arc::clone(&self.0.registry);
                let registry = clone_lock.read().expect("Registry RwLock poisoned!");
                let instance = registry.get(&address).unwrap().clone();
                drop(registry);

                instance
            }

            pub fn network(&self) -> Option<$crate::network::Network> {
                self.0.network
            }
        }

        impl<M> $abi<M>
        where
            M: ::ethers::prelude::Middleware,
        {
            pub async fn get_logs<E>(
                &self,
                from_block: ::ethers::prelude::BlockNumber,
                to_block: ::ethers::prelude::BlockNumber,
            ) -> Result<Vec<E>, $crate::error::Error>
            where
                E: ethers::prelude::EthEvent + std::fmt::Debug,
            {
                let res = self
                    .event::<E>()
                    .address(ethers::prelude::ValueOrArray::Value(self.address()))
                    .from_block(from_block)
                    .to_block(to_block)
                    .query()
                    .await?;

                Ok(res)
            }
        }
    };
}

#[macro_export]
macro_rules! address_from {
    ($address: expr) => {
        $address.parse::<ethers::prelude::Address>()
    };
}
