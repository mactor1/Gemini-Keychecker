use crate::config::KeyCheckerConfig;
use reqwest::Client;
use std::time::Duration;

pub fn client_builder(config: &KeyCheckerConfig) -> Result<Client, reqwest::Error> {
    // Set the maximum number of connections per host based on concurrency.
    let pool_size = config.concurrency / 2;

    let mut builder = Client::builder()
        .timeout(Duration::from_secs(config.timeout_sec))
        .pool_max_idle_per_host(pool_size);

    if let Some(ref proxy_url) = config.proxy {
        builder = builder.proxy(reqwest::Proxy::all(proxy_url.clone())?);
    }

    if !config.enable_multiplexing {
        builder = builder.http1_only();
    }

    builder.build()
}
