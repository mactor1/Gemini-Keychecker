use backon::{ExponentialBuilder, Retryable};
use reqwest::{Client, IntoUrl, StatusCode};
use tokio::time::Duration;
use tracing::{debug, error, info, warn};
use url::Url;

use crate::config::TEST_MESSAGE_BODY;
use crate::error::ValidatorError;
use crate::types::GeminiKey;

pub async fn validate_key(
    client: Client,
    api_endpoint: impl IntoUrl,
    api_key: GeminiKey,
) -> Result<GeminiKey, ValidatorError> {
    let api_endpoint = api_endpoint.into_url()?;

    match send_test_request(client, &api_endpoint, api_key.clone()).await {
        Ok(_) => {
            info!("SUCCESS - {}... - Valid key found", &api_key.as_ref()[..10]);
            Ok(api_key)
        }
        Err(e) => {
            match e.status() {
                Some(StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN | StatusCode::TOO_MANY_REQUESTS) => {
                    warn!("INVALID - {}... - {}", &api_key.as_ref()[..10], ValidatorError::KeyInvalid);
                    Err(ValidatorError::KeyInvalid)
                }
                _ => {
                    let req_error = ValidatorError::ReqwestError(e);
                    error!("ERROR-  {}... - {}", &api_key.as_ref()[..10], req_error);
                    Err(req_error)
                }
            }
        }
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
