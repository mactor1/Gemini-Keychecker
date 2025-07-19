use anyhow::Result;
use backon::{ExponentialBuilder, Retryable};
use reqwest::{Client, StatusCode};
use tokio::time::Duration;
use url::Url;

use crate::config::TEST_MESSAGE_BODY;
use crate::types::{GeminiKey, KeyStatus};

pub async fn validate_key_with_retry(
    client: Client,
    full_url: Url,
    key: GeminiKey,
) -> Option<GeminiKey> {
    let retry_policy = ExponentialBuilder::default()
        .with_max_times(3)
        .with_min_delay(Duration::from_secs(3))
        .with_max_delay(Duration::from_secs(5));

    let result = (async || match keytest(client.to_owned(), &full_url, &key).await {
        Ok(KeyStatus::Valid) => {
            println!("Key: {}... -> SUCCESS", &key.as_ref()[..10]);
            Ok(Some(key.clone()))
        }
        Ok(KeyStatus::Invalid) => {
            println!("Key: {}... -> INVALID (Forbidden)", &key.as_ref()[..10]);
            Ok(None)
        }
        Ok(KeyStatus::Retryable(reason)) => {
            eprintln!(
                "Key: {}... -> RETRYABLE (Reason: {})",
                &key.as_ref()[..10],
                reason
            );
            Err(anyhow::anyhow!("Retryable error: {}", reason))
        }
        Err(e) => {
            eprintln!(
                "Key: {}... -> NETWORK ERROR (Reason: {})",
                &key.as_ref()[..10],
                e
            );
            Err(e)
        }
    })
    .retry(retry_policy)
    .await;

    match result {
        Ok(key_result) => key_result,
        Err(_) => {
            eprintln!(
                "Key: {}... -> FAILED after all retries.",
                &key.as_ref()[..10]
            );
            None
        }
    }
}

async fn keytest(client: Client, full_url: &Url, key: &GeminiKey) -> Result<KeyStatus> {
    let response = client
        .post(full_url.clone())
        .header("Content-Type", "application/json")
        .header("X-goog-api-key", key.as_ref())
        .json(&*TEST_MESSAGE_BODY)
        .send()
        .await?;

    let status = response.status();

    let key_status = match status {
        StatusCode::OK => KeyStatus::Valid,
        StatusCode::FORBIDDEN | StatusCode::UNAUTHORIZED => KeyStatus::Invalid,
        other => KeyStatus::Retryable(format!("Received status {}, will retry.", other)),
    };
    Ok(key_status)
}
