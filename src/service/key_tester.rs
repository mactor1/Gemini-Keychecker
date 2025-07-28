use backon::{ExponentialBuilder, Retryable};
use reqwest::{Client, IntoUrl, StatusCode};
use tokio::time::Duration;
use url::Url;

use crate::config::TEST_MESSAGE_BODY;
use crate::error::ValidationError;
use crate::types::GeminiKey;

pub async fn validate_key(
    client: Client,
    api_endpoint: impl IntoUrl,
    api_key: GeminiKey,
) -> Result<GeminiKey, ValidationError> {
    let api_endpoint = api_endpoint.into_url()?;

    match send_test_request(client, &api_endpoint, api_key.clone()).await {
        Ok(response) => {
            let status = response.status();
            match status {
                StatusCode::OK => Ok(api_key),
                StatusCode::UNAUTHORIZED
                | StatusCode::FORBIDDEN
                | StatusCode::TOO_MANY_REQUESTS => Err(ValidationError::KeyInvalid),
                _ => Err(ValidationError::ReqwestError(
                    response.error_for_status().unwrap_err(),
                )),
            }
        }
        Err(e) => Err(ValidationError::ReqwestError(e)),
    }
}

async fn send_test_request(
    client: Client,
    api_endpoint: &Url,
    key: GeminiKey,
) -> Result<reqwest::Response, reqwest::Error> {
    let retry_policy = ExponentialBuilder::default()
        .with_max_times(3)
        .with_min_delay(Duration::from_secs(3))
        .with_max_delay(Duration::from_secs(5));

    (async || {
        let response = client
            .post(api_endpoint.clone())
            .header("Content-Type", "application/json")
            .header("X-goog-api-key", key.as_ref())
            .json(&*TEST_MESSAGE_BODY)
            .send()
            .await?;

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
