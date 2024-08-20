use {
    crate::chain::Chain,
    ethers::types::Address,
    std::{
        clone::Clone,
        collections::HashMap,
        sync::{Arc, RwLock},
    },
};

#[derive(Debug)]
pub struct AbiRegistry<C> {
    pub url: Option<String>,
    pub chain: Option<Chain>,
    pub registry: Arc<RwLock<HashMap<Address, C>>>,
}

impl<C> AbiRegistry<C> {
    pub fn new(url: String, chain: Chain) -> Self {
        Self {
            url: Some(url),
            chain: Some(chain),
            registry: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn mock() -> Self {
        Self {
            url: None,
            chain: None,
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
        impl $crate::provider::AbiProviderTrait<$crate::provider::WsProvider>
            for $abi_registry<$crate::provider::WsProvider>
        {
            async fn provider(&self) -> Result<$crate::provider::WsProvider, $crate::error::Error> {
                let provider: $crate::provider::WsProvider =
                    $crate::provider::AbiProvider::_new(self.0.url.clone(), self.0.chain)
                        .provider()
                        .await?;

                Ok(provider)
            }
        }

        #[async_trait::async_trait]
        impl $crate::provider::AbiProviderTrait<$crate::provider::IpcProvider>
            for $abi_registry<$crate::provider::IpcProvider>
        {
            async fn provider(
                &self,
            ) -> Result<$crate::provider::IpcProvider, $crate::error::Error> {
                let provider: $crate::provider::IpcProvider =
                    $crate::provider::AbiProvider::_new(self.0.url.clone(), self.0.chain)
                        .provider()
                        .await?;

                Ok(provider)
            }
        }

        #[async_trait::async_trait]
        impl $crate::provider::AbiProviderTrait<$crate::provider::HttpProvider>
            for $abi_registry<$crate::provider::HttpProvider>
        {
            async fn provider(
                &self,
            ) -> Result<$crate::provider::HttpProvider, $crate::error::Error> {
                let provider: $crate::provider::HttpProvider =
                    $crate::provider::AbiProvider::_new(self.0.url.clone(), self.0.chain)
                        .provider()
                        .await?;

                Ok(provider)
            }
        }

        #[async_trait::async_trait]
        impl $crate::provider::AbiProviderTrait<$crate::provider::RetryProvider>
            for $abi_registry<$crate::provider::RetryProvider>
        {
            async fn provider(
                &self,
            ) -> Result<$crate::provider::RetryProvider, $crate::error::Error> {
                let provider: $crate::provider::RetryProvider =
                    $crate::provider::AbiProvider::_new(self.0.url.clone(), self.0.chain)
                        .provider()
                        .await?;

                Ok(provider)
            }
        }

        #[async_trait::async_trait]
        impl $crate::provider::AbiProviderTrait<$crate::provider::MockProvider>
            for $abi_registry<$crate::provider::MockProvider>
        {
            async fn provider(
                &self,
            ) -> Result<$crate::provider::MockProvider, $crate::error::Error> {
                let provider: $crate::provider::MockProvider =
                    $crate::provider::AbiProvider::mock().provider().await?;

                Ok(provider)
            }
        }

        impl<M> $abi_registry<M>
        where
            M: ::ethers::prelude::Middleware,
        {
            pub fn new(url: String, chain: $crate::chain::Chain) -> Self {
                let registry = $crate::registry::AbiRegistry::<$abi<M>>::new(url, chain);
                Self(registry)
            }

            pub fn mock() -> Self {
                let registry = $crate::registry::AbiRegistry::<$abi<M>>::mock();
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

            pub fn chain(&self) -> Option<$crate::chain::Chain> {
                self.0.chain
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
