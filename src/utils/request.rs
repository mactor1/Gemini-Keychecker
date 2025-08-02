use backon::{ExponentialBuilder, Retryable};
use reqwest::{Client, StatusCode};
use serde_json::Value;
use tokio::time::Duration;
use tracing::debug;
use url::Url;

use crate::types::GeminiKey;

pub async fn send_request(
    client: Client,
    api_endpoint: &Url,
    key: GeminiKey,
    payload: &Value,
    max_retries: usize,
) -> Result<reqwest::Response, reqwest::Error> {
    let retry_policy = ExponentialBuilder::default()
        .with_max_times(max_retries)
        .with_min_delay(Duration::from_secs(1))
        .with_max_delay(Duration::from_secs(2));

    (async || {
        let response = client
            .post(api_endpoint.clone())
            .header("Content-Type", "application/json")
            .header("X-goog-api-key", key.as_ref())
            .json(payload)
            .send()
            .await?;
        debug!("Response for key {}: {:?}", key.as_ref(), response.status());
        response.error_for_status()
    })
    .retry(&retry_policy)
    .when(|e: &reqwest::Error| {
        !matches!(
            e.status(),
            Some(StatusCode::FORBIDDEN | StatusCode::UNAUTHORIZED)
        )
    })
    .await
}
