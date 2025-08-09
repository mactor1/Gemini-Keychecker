use crate::types::GeminiKey;
use crate::{config::KeyCheckerConfig, error::ValidatorError};
use backon::{ExponentialBuilder, Retryable};
use reqwest::Client;
use serde::Serialize;
use std::time::Duration;
use tokio::time::Duration as TokioDuration;
use tracing::debug;
use url::Url;

pub fn client_builder(config: &KeyCheckerConfig) -> Result<Client, ValidatorError> {
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

    Ok(builder.build()?)
}

pub async fn send_request<T>(
    client: Client,
    api_endpoint: &Url,
    key: GeminiKey,
    payload: &T,
    max_retries: usize,
) -> Result<(), ValidatorError>
where
    T: Serialize,
{
    let retry_policy = ExponentialBuilder::default()
        .with_max_times(max_retries)
        .with_min_delay(TokioDuration::from_secs(1))
        .with_max_delay(TokioDuration::from_secs(2));

    (async || {
        let response = client
            .post(api_endpoint.clone())
            .header("Content-Type", "application/json")
            .header("X-goog-api-key", key.as_ref())
            .json(payload)
            .send()
            .await?;

        let status = response.status();

        if status.is_success() {
            Ok(())
        } else {
            let body = response.text().await.map_err(ValidatorError::from)?;
            debug!(
                "Response for key {}: status={:?}, body={}",
                key.as_ref(),
                status,
                body
            );

            let status_code = status.as_u16();
            match status_code {
                400 => Err(ValidatorError::HttpBadRequest { body }),
                401 => Err(ValidatorError::HttpUnauthorized { body }),
                403 => Err(ValidatorError::HttpForbidden { body }),
                429 => Err(ValidatorError::HttpTooManyRequests { body }),
                400..=499 => Err(ValidatorError::HttpClientError {
                    status: status_code,
                    body,
                }),
                500..=599 => Err(ValidatorError::HttpServerError {
                    status: status_code,
                    body,
                }),
                _ => {
                    // For other status codes, treat as client error
                    Err(ValidatorError::HttpClientError {
                        status: status_code,
                        body,
                    })
                }
            }
        }
    })
    .retry(&retry_policy)
    .when(|error: &ValidatorError| {
        !matches!(
            error,
            ValidatorError::HttpUnauthorized { .. } | ValidatorError::HttpForbidden { .. }
        )
    })
    .await
}
