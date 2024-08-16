use {
    crate::{
        chain::{Chain, RetryClientConfig},
        error::Error,
    },
    async_trait::async_trait,
    ethers::{
        providers::{
            Http, HttpRateLimitRetryPolicy, Ipc, Middleware, MockProvider, Provider, RetryClient,
            RetryClientBuilder, Ws,
        },
        types::U256,
    },
    std::{path::Path, time::Duration},
    url::Url,
};

#[async_trait]
pub trait AbiProviderTrait<M>
where
    M: Middleware,
{
    async fn provider(&self) -> Result<M, Error>;
}

pub struct AbiProvider {
    pub url: Option<String>,
    pub chain: Option<Chain>,
}

impl AbiProvider {
    pub fn _new(url: Option<String>, chain: Option<Chain>) -> Self {
        Self { url, chain }
    }

    pub fn new(url: String, chain: Chain) -> Self {
        Self {
            url: Some(url),
            chain: Some(chain),
        }
    }

    pub fn mock() -> Self {
        Self {
            url: None,
            chain: None,
        }
    }
}

macro_rules! assert_chain_id {
    ($chain: expr, $provider: expr) => {
        if let Some(chain) = $chain {
            if chain.assert_chain_id() {
                let provider_chain_id = $provider.get_chainid().await?;
                if U256::from(chain.id()) != provider_chain_id {
                    let e = format!(
                        "Configured chain_id ({}) does not match chain ({})",
                        U256::from(chain.id()),
                        provider_chain_id
                    );
                    return Err(Error::ChainIdError(e));
                }
            }
        }
    };
}

#[async_trait]
impl AbiProviderTrait<Provider<Ws>> for AbiProvider {
    async fn provider(&self) -> Result<Provider<Ws>, Error> {
        match &self.url {
            Some(url) => {
                let url = Url::parse(url)?;
                let provider = Provider::<Ws>::connect(url).await?;
                assert_chain_id!(self.chain, provider);
                Ok(provider)
            }
            None => Err(Error::Error(String::from("Provider url is None"))),
        }
    }
}

#[async_trait]
impl AbiProviderTrait<Provider<Ipc>> for AbiProvider {
    async fn provider(&self) -> Result<Provider<Ipc>, Error> {
        match &self.url {
            Some(url) => {
                let provider = Provider::<Ipc>::connect_ipc(Path::new(&url)).await?;
                assert_chain_id!(self.chain, provider);
                Ok(provider)
            }
            None => Err(Error::Error(String::from("Provider url is None"))),
        }
    }
}

#[async_trait]
impl AbiProviderTrait<Provider<Http>> for AbiProvider {
    async fn provider(&self) -> Result<Provider<Http>, Error> {
        match &self.url {
            Some(url) => {
                let url = Url::parse(url)?;
                let provider = Provider::<Http>::new(Http::new(url));
                assert_chain_id!(self.chain, provider);
                Ok(provider)
            }
            None => Err(Error::Error(String::from("Provider url is None"))),
        }
    }
}

#[async_trait]
impl AbiProviderTrait<Provider<RetryClient<Http>>> for AbiProvider {
    async fn provider(&self) -> Result<Provider<RetryClient<Http>>, Error> {
        match &self.url {
            Some(url) => {
                let url = Url::parse(url)?;
                let retry_config = match self.chain {
                    Some(chain) => chain.retry_client_config(),
                    None => RetryClientConfig::default(),
                };

                let provider = Provider::new(
                    RetryClientBuilder::default()
                        .rate_limit_retries(retry_config.rate_limit_retries)
                        .timeout_retries(retry_config.timeout_retries)
                        .initial_backoff(Duration::from_millis(retry_config.initial_backoff_ms))
                        .build(Http::new(url), Box::new(HttpRateLimitRetryPolicy)),
                );
                assert_chain_id!(self.chain, provider);
                Ok(provider)
            }
            None => Err(Error::Error(String::from("Provider url is None"))),
        }
    }
}

#[async_trait]
impl AbiProviderTrait<Provider<MockProvider>> for AbiProvider {
    async fn provider(&self) -> Result<Provider<MockProvider>, Error> {
        match &self.url {
            Some(_) => Err(Error::Error(String::from("MockProvider url is not None"))),
            None => {
                let (provider, _mock) = Provider::mocked();
                Ok(provider)
            }
        }
    }
}
