use reqwest::{Client, IntoUrl, StatusCode};
use tracing::{error, info, warn};

use super::{CACHE_CONTENT_TEST_BODY, GENERATE_CONTENT_TEST_BODY};
use crate::config::KeyCheckerConfig;
use crate::error::ValidatorError;
use crate::types::GeminiKey;
use crate::utils::send_request;

pub async fn test_generate_content_api(
    client: Client,
    api_endpoint: impl IntoUrl,
    api_key: GeminiKey,
    config: KeyCheckerConfig,
) -> Result<GeminiKey, ValidatorError> {
    let api_endpoint = api_endpoint.into_url()?;

    match send_request(
        client,
        &api_endpoint,
        api_key.clone(),
        &GENERATE_CONTENT_TEST_BODY,
        config.max_retries,
    )
    .await
    {
        Ok(_) => {
            info!("SUCCESS - {}... - Valid key found", &api_key.as_ref()[..10]);
            Ok(api_key)
        }
        Err(e) => match e.status() {
            Some(
                StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN | StatusCode::TOO_MANY_REQUESTS,
            ) => {
                warn!(
                    "INVALID - {}... - {}",
                    &api_key.as_ref()[..10],
                    ValidatorError::KeyInvalid
                );
                Err(ValidatorError::KeyInvalid)
            }
            _ => {
                let req_error = ValidatorError::from(e);
                error!("ERROR-  {}... - {}", &api_key.as_ref()[..10], req_error);
                Err(req_error)
            }
        },
    }
}

pub async fn test_cache_content_api(
    client: Client,
    api_endpoint: impl IntoUrl,
    api_key: GeminiKey,
    config: KeyCheckerConfig,
) -> Result<GeminiKey, ValidatorError> {
    let api_endpoint = api_endpoint.into_url()?;

    match send_request(
        client,
        &api_endpoint,
        api_key.clone(),
        &CACHE_CONTENT_TEST_BODY,
        config.max_retries,
    )
    .await
    {
        Ok(_) => {
            info!(
                "CACHE SUCCESS - {}... - Valid key for cache API",
                &api_key.as_ref()[..10]
            );
            Ok(api_key)
        }
        Err(e) => match e.status() {
            Some(
                StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN | StatusCode::TOO_MANY_REQUESTS,
            ) => {
                warn!(
                    "CACHE INVALID - {}... - {}",
                    &api_key.as_ref()[..10],
                    ValidatorError::KeyInvalid
                );
                Err(ValidatorError::KeyInvalid)
            }
            _ => {
                let req_error = ValidatorError::from(e);
                error!(
                    "CACHE ERROR - {}... - {}",
                    &api_key.as_ref()[..10],
                    req_error
                );
                Err(req_error)
            }
        },
    }
}
