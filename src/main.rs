use anyhow::Result;
use async_stream::stream;
use backon::{ExponentialBuilder, Retryable};
use clap::Parser;
use futures::{pin_mut, stream::StreamExt};
use regex::Regex;
use reqwest::{Client, StatusCode};
use std::{
    collections::HashSet,
    fs,
    io::Write,
    path::{Path, PathBuf},
    sync::LazyLock,
    thread::spawn,
    time::Instant,
};
use tokio::time::Duration;
use url::Url;

// Regex pattern for validating Google API keys (AIzaSy followed by 33 characters)
static API_KEY_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^AIzaSy.{33}$").unwrap());
/// Configuration structure for the key checker tool
#[derive(Parser, Debug)]
#[command(version, about = "A tool to check and backup API keys", long_about = None)]
struct KeyCheckerConfig {
    /// Input file path containing API keys to check
    #[arg(long, short = 'i', default_value = "keys.txt")]
    input_path: PathBuf,

    /// Output file path for valid API keys
    #[arg(long, short = 'o', default_value = "output_keys.txt")]
    output_path: PathBuf,

    /// API host URL for key validation
    #[arg(long, short = 'u', default_value = "https://generativelanguage.googleapis.com/")]
    api_host: Url,

    /// Request timeout in seconds
    #[arg(long, short = 't', default_value_t = 60)]
    timeout_sec: u64,

    /// Maximum number of concurrent requests
    #[arg(long, short = 'c', default_value_t = 30)]
    concurrency: usize,

    /// Optional proxy URL for HTTP requests (supports http://user:pass@host:port)
    #[arg(long, short = 'x')]
    proxy: Option<Url>,
}
/// Status of API key validation
#[derive(Debug)]
enum KeyStatus {
    /// Key is valid and working
    Valid,
    /// Key is invalid or unauthorized
    Invalid,
    /// Temporary error, key validation should be retried
    Retryable(String),
}
/// Load and validate API keys from a file
/// Returns a vector of unique, valid API keys
fn load_keys(path: &Path) -> Result<Vec<String>> {
    let keys_txt = fs::read_to_string(path)?;
    // Use HashSet to automatically deduplicate keys
    let unique_keys_set: HashSet<&str> = keys_txt
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .filter(|line| API_KEY_REGEX.is_match(line))
        .collect();
    let keys: Vec<String> = unique_keys_set.into_iter().map(String::from).collect();
    Ok(keys)
}

/// Validate an API key with exponential backoff retry logic
/// Returns Some(key) if valid, None if invalid or failed after all retries
async fn validate_key_with_retry(client: &Client, api_host: &Url, key: String) -> Option<String> {
    // Configure exponential backoff retry policy
    let retry_policy = ExponentialBuilder::default()
        .with_max_times(3)
        .with_min_delay(Duration::from_secs(5))
        .with_max_delay(Duration::from_secs(10));

    let result = (|| async {
        match keytest(&client, &api_host, &key).await {
            Ok(KeyStatus::Valid) => {
                println!("Key: {}... -> SUCCESS", &key[..10]);
                Ok(Some(key.clone()))
            }
            Ok(KeyStatus::Invalid) => {
                println!("Key: {}... -> INVALID (Forbidden)", &key[..10]);
                Ok(None)
            }
            Ok(KeyStatus::Retryable(reason)) => {
                eprintln!("Key: {}... -> RETRYABLE (Reason: {})", &key[..10], reason);
                Err(anyhow::anyhow!("Retryable error: {}", reason))
            }
            Err(e) => {
                eprintln!("Key: {}... -> NETWORK ERROR (Reason: {})", &key[..10], e);
                Err(e)
            }
        }
    })
    .retry(retry_policy)
    .await;

    match result {
        Ok(key_result) => key_result,
        Err(_) => {
            eprintln!("Key: {}... -> FAILED after all retries.", &key[..10]);
            None
        }
    }
}

/// Test a single API key by making a request to the Gemini API
/// Returns the validation status based on the HTTP response
async fn keytest(client: &Client, api_host: &Url, keys: &str) -> Result<KeyStatus> {
    const API_PATH: &str = "v1beta/models/gemini-2.0-flash-exp:generateContent";
    let full_url = api_host.join(API_PATH)?;
    
    // Simple test request body
    let request_body = serde_json::json!({
        "contents": [
            {
                "parts": [
                    {
                        "text": "Hi"
                    }
                ]
            }
        ]
    });

    let response = client
        .post(full_url)
        .header("Content-Type", "application/json")
        .header("X-goog-api-key", keys)
        .json(&request_body)
        .send()
        .await?;

    let status = response.status();

    let key_status = match status {
        // 200 OK - Key is valid
        StatusCode::OK => KeyStatus::Valid,

        // 403 & 401 - Key is invalid or unauthorized
        StatusCode::FORBIDDEN | StatusCode::UNAUTHORIZED => KeyStatus::Invalid,

        // Other status codes - Temporary error, retry
        other => KeyStatus::Retryable(format!("Received status {}, will retry.", other)),
    };
    Ok(key_status)
}

/// Build HTTP client with optional proxy configuration
/// Returns a configured reqwest Client
fn build_client(config: &KeyCheckerConfig) -> Result<Client> {
    let mut client_builder = Client::builder()
        .timeout(Duration::from_secs(config.timeout_sec));
    
    // Add proxy configuration if specified
    if let Some(proxy_url) = &config.proxy {
        let mut proxy = reqwest::Proxy::all(proxy_url.clone())?;
        
        // Extract username and password from URL if present
        if !proxy_url.username().is_empty() {
            let username = proxy_url.username();
            let password = proxy_url.password().unwrap_or("");
            proxy = proxy.basic_auth(username, password);
        }
        
        client_builder = client_builder.proxy(proxy);
    }
    
    client_builder.build().map_err(Into::into)
}

/// Main function - orchestrates the key validation process
#[tokio::main]
async fn main() -> Result<()> {
    let start_time = Instant::now();
    let config = KeyCheckerConfig::parse();
    let keys = load_keys(&config.input_path)?;
    let client = build_client(&config)?;

    // Create channel for streaming keys from producer to consumer
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<String>();
    let stream = stream! {
        while let Some(item) = rx.recv().await {
            yield item;
        }
    };

    // Spawn producer thread to send keys through channel
    spawn(move || {
        for key in keys {
            if API_KEY_REGEX.is_match(&key) {
                if let Err(e) = tx.send(key) {
                    eprintln!("Failed to send key: {}", e);
                }
            } else {
                eprintln!("Invalid key format: {}", key);
            }
        }
    });

    // Create stream to validate keys concurrently
    let valid_keys_stream = stream
        .map(|key| validate_key_with_retry(&client, &config.api_host, key))
        .buffer_unordered(config.concurrency)
        .filter_map(|r| async { r });
    pin_mut!(valid_keys_stream);
    
    // Open output file for writing valid keys
    let mut output_file = fs::File::create(&config.output_path)?;
    
    // Process validated keys and write to output file
    while let Some(valid_key) = valid_keys_stream.next().await {
        println!("Valid key found: {}", valid_key);
        if let Err(e) = writeln!(output_file, "{}", valid_key) {
            eprintln!("Failed to write key to output file: {}", e);
        }
    }
    
    println!("Total Elapsed Time: {:?}", start_time.elapsed());
    Ok(())
}