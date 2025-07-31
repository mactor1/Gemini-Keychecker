use crate::error::ValidatorError;
use async_stream::stream;
use futures::{pin_mut, stream::StreamExt};
use reqwest::Client;
use std::time::Instant;
use tokio::{fs, io::AsyncWriteExt, sync::mpsc};

use super::{http_client::client_builder, key_tester::validate_key};
use crate::adapters::{load_keys, write_keys_txt_file};
use crate::config::KeyCheckerConfig;
use crate::types::GeminiKey;

pub struct ValidationService {
    config: KeyCheckerConfig,
    client: Client,
    full_url: url::Url,
}

impl ValidationService {
    pub fn new(config: KeyCheckerConfig, client: Client) -> Self {
        let full_url = config.gemini_api_url();
        Self {
            config,
            client,
            full_url,
        }
    }

    pub async fn validate_keys(&self, keys: Vec<GeminiKey>) -> Result<(), ValidatorError> {
        let start_time = Instant::now();

        // Create channel for streaming keys from producer to consumer
        let (tx, mut rx) = mpsc::unbounded_channel::<GeminiKey>();
        let stream = stream! {
            while let Some(item) = rx.recv().await {
                yield item;
            }
        };

        // Spawn producer task to send keys through channel
        tokio::spawn(async move {
            for key in keys {
                if let Err(e) = tx.send(key) {
                    eprintln!("Failed to send key: {}", e);
                }
            }
        });

        // Create stream to validate keys concurrently
        let valid_keys_stream = stream
            .map(|key| {
                validate_key(
                    self.client.clone(),
                    self.full_url.clone(),
                    key,
                    self.config.clone(),
                )
            })
            .buffer_unordered(self.config.concurrency)
            .filter_map(|result| async { result.ok() });
        pin_mut!(valid_keys_stream);

        // Open output file for writing valid keys
        let output_file = fs::File::create(&self.config.output_path).await?;
        let mut buffer_writer = tokio::io::BufWriter::new(output_file);

        // Process validated keys and write to output file
        while let Some(valid_key) = valid_keys_stream.next().await {
            if let Err(e) = write_keys_txt_file(&mut buffer_writer, &valid_key).await {
                eprintln!("Failed to write key to output file: {}", e);
            }
        }

        // Flush the buffer to ensure all data is written to the file
        buffer_writer.flush().await?;

        println!("Total Elapsed Time: {:?}", start_time.elapsed());
        Ok(())
    }
}

pub async fn start_validation() -> Result<(), ValidatorError> {
    let config = KeyCheckerConfig::load_config()?;

    // 加载密钥
    let keys = load_keys(config.input_path.as_path())?;

    // 构建HTTP客户端
    let client = client_builder(&config)?;

    // 创建验证服务并启动
    let validation_service = ValidationService::new(config, client);
    validation_service.validate_keys(keys).await
}
