use backon::{ExponentialBuilder, Retryable};
use reqwest::Client;
use serde_json::Value;
use tokio::time::Duration;
use tracing::debug;
use url::Url;

use crate::error::ValidatorError;
use crate::types::GeminiKey;

pub async fn send_request(
    client: Client,
    api_endpoint: &Url,
    key: GeminiKey,
    payload: &Value,
    max_retries: usize,
) -> Result<(), ValidatorError> {
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
