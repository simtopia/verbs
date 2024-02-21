use super::runtime_client::{RuntimeClient, RuntimeClientBuilder};
use ethers_core::types::Chain;
use ethers_providers::{is_local_endpoint, Provider, DEFAULT_LOCAL_POLL_INTERVAL};
use eyre::{Result, WrapErr};
use reqwest::Url;
use std::{
    path::{Path, PathBuf},
    time::Duration,
};
use url::ParseError;

const ALCHEMY_FREE_TIER_CUPS: u64 = 330;
const REQUEST_TIMEOUT: Duration = Duration::from_secs(45);

pub type RetryProvider = Provider<RuntimeClient>;

#[derive(Debug)]
pub struct ProviderBuilder {
    url: Result<Url>,
    chain: Chain,
    max_retry: u32,
    timeout_retry: u32,
    initial_backoff: u64,
    timeout: Duration,
    compute_units_per_second: u64,
    jwt: Option<String>,
    headers: Vec<String>,
}

impl ProviderBuilder {
    pub fn new(url_str: &str) -> Self {
        let mut url_str = url_str;
        let storage;
        if url_str.starts_with("localhost:") {
            storage = format!("http://{url_str}");
            url_str = storage.as_str();
        }

        let url = Url::parse(url_str)
            .or_else(|err| match err {
                ParseError::RelativeUrlWithoutBase => {
                    let path = Path::new(url_str);

                    if let Ok(path) = resolve_path(path) {
                        Url::parse(&format!("file://{}", path.display()))
                    } else {
                        Err(err)
                    }
                }
                _ => Err(err),
            })
            .wrap_err_with(|| format!("invalid provider URL: {url_str:?}"));

        Self {
            url,
            chain: Chain::Mainnet,
            max_retry: 8,
            timeout_retry: 8,
            initial_backoff: 800,
            timeout: REQUEST_TIMEOUT,
            compute_units_per_second: ALCHEMY_FREE_TIER_CUPS,
            jwt: None,
            headers: vec![],
        }
    }

    pub fn build(self) -> Result<RetryProvider> {
        let ProviderBuilder {
            url,
            chain,
            max_retry,
            timeout_retry,
            initial_backoff,
            timeout,
            compute_units_per_second,
            jwt,
            headers,
        } = self;
        let url = url?;

        let client_builder = RuntimeClientBuilder::new(
            url.clone(),
            max_retry,
            timeout_retry,
            initial_backoff,
            timeout,
            compute_units_per_second,
        )
        .with_headers(headers)
        .with_jwt(jwt);

        let mut provider = Provider::new(client_builder.build());

        let is_local = is_local_endpoint(url.as_str());

        if is_local {
            provider = provider.interval(DEFAULT_LOCAL_POLL_INTERVAL);
        } else if let Some(blocktime) = chain.average_blocktime_hint() {
            provider = provider.interval(blocktime / 2);
        }

        Ok(provider)
    }
}

#[cfg(not(windows))]
fn resolve_path(path: &Path) -> Result<PathBuf, ()> {
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        std::env::current_dir().map(|d| d.join(path)).map_err(drop)
    }
}

#[cfg(windows)]
fn resolve_path(path: &Path) -> Result<PathBuf, ()> {
    if let Some(s) = path.to_str() {
        if s.starts_with(r"\\.\pipe\") {
            return Ok(path.to_path_buf());
        }
    }
    Err(())
}
