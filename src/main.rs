use anyhow::Result;
use clap::Parser;
use regex::Regex;
use reqwest::{Client, StatusCode};
use std::{
    collections::HashSet,
    fs,
    path::PathBuf,
    sync::{Arc, LazyLock},
    time::Instant,
};
use tokio::{
    sync::Semaphore,
    task::JoinSet,
    time::{Duration, sleep},
};
use url::Url;
static API_KEY_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^AIzaSy.{33}$").unwrap());
#[derive(Parser, Debug)]
#[command(version, about = "A tool to check and backup API keys", long_about = None)]
struct KeyCheckerConfig {
    #[arg(long, short = 'i', default_value = "keys.txt")]
    input_path: PathBuf,

    #[arg(long, short = 'o', default_value = "output_keys.txt")]
    output_path: PathBuf,

    #[arg(long, short = 'u', default_value = "https://generativelanguage.googleapis.com/")]
    api_host: Url,

    #[arg(long, short = 't', default_value_t = 5000)]
    timeout_ms: u64,

    #[arg(long, short = 'c', default_value_t = 30)]
    concurrency: usize,
}
#[derive(Debug)]
enum KeyStatus {
    Valid,
    Invalid,
    Retryable(String),
}
fn load_keys(path: &PathBuf) -> Result<Vec<String>> {
    let keys_txt = fs::read_to_string(path)?;
    let unique_keys_set: HashSet<&str> = keys_txt
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .filter(|line| API_KEY_REGEX.is_match(line))
        .collect();
    let keys: Vec<String> = unique_keys_set.into_iter().map(String::from).collect();
    Ok(keys)
}

fn output_file_txt(keys: &[String], output_path: &PathBuf) -> Result<()> {
    let content = keys.join("\n");
    fs::write(output_path, content)?;
    println!(
        "Successfully wrote {} keys to {:?}",
        keys.len(),
        output_path
    );
    Ok(())
}

async fn keytest(client: &Client, api_host: &Url, keys: &str) -> Result<KeyStatus> {
    const API_PATH: &str = "v1beta/models/gemini-2.0-flash-exp:generateContent";
    let full_url = api_host.join(API_PATH)?;
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
        // 200 OK
        StatusCode::OK => KeyStatus::Valid,

        // 403 & 401
        StatusCode::FORBIDDEN | StatusCode::UNAUTHORIZED => KeyStatus::Invalid,

        // Other Status Code
        other => KeyStatus::Retryable(format!("Received status {}, will retry.", other)),
    };
    Ok(key_status)
}

#[tokio::main]
async fn main() -> Result<()> {
    let start_time = Instant::now();
    let config = KeyCheckerConfig::parse();
    let keys = load_keys(&config.input_path)?;
    let client = Client::builder()
        .timeout(Duration::from_millis(config.timeout_ms))
        .build()?;

    let semaphore = Arc::new(Semaphore::new(config.concurrency));
    let mut set = JoinSet::new();

    for key in keys {
        let client_clone = client.clone();
        let api_host_clone = config.api_host.clone();
        let semaphore_clone = Arc::clone(&semaphore);

        set.spawn(async move {
            const MAX_RETRIES: u32 = 3; 
            let _permit = semaphore_clone.acquire().await.unwrap();
            for attempt in 0..MAX_RETRIES {
                match keytest(&client_clone, &api_host_clone, &key).await {
                    Ok(KeyStatus::Valid) => {
                        println!("Key: {}... -> SUCCESS", &key[..10]);
                        return Some(key);
                    }
                    Ok(KeyStatus::Invalid) => {
                        println!("Key: {}... -> INVALID (Forbidden)", &key[..10]);
                        return None;
                    }
                    Ok(KeyStatus::Retryable(reason)) => {
                        eprintln!(
                            "Key: {}... -> RETRYABLE (Attempt {}/{}, Reason: {})",
                            &key[..10],
                            attempt + 1,
                            MAX_RETRIES,
                            reason
                        );
                        if attempt < MAX_RETRIES - 1 {
                            sleep(Duration::from_secs(2_u64.pow(attempt))).await;
                        }
                    }
                    Err(e) => {
                        eprintln!(
                            "Key: {}... -> NETWORK ERROR (Attempt {}/{}, Reason: {})",
                            &key[..10],
                            attempt + 1,
                            MAX_RETRIES,
                            e
                        );
                        if attempt < MAX_RETRIES - 1 {
                            sleep(Duration::from_secs(2_u64.pow(attempt))).await;
                        }
                    }
                }
            }

            eprintln!("Key: {}... -> FAILED after all retries.", &key[..10]);
            None
        });
    }
    let mut valid_keys = Vec::new();
    while let Some(res) = set.join_next().await {
        if let Ok(Some(key)) = res {
            valid_keys.push(key);
        }
    }
    output_file_txt(&valid_keys, &config.output_path)?;
    println!("Total cost time:{:?}", start_time.elapsed());
    Ok(())
}
