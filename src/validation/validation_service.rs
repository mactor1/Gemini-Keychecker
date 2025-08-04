use super::key_validator::{test_cache_content_api, test_generate_content_api};
use crate::adapters::{load_keys, write_validated_key_to_tier_files};
use crate::config::KeyCheckerConfig;
use crate::error::ValidatorError;
use crate::types::GeminiKey;
use crate::utils::client_builder;
use async_stream::stream;
use futures::{pin_mut, stream::StreamExt};
use reqwest::Client;
use std::time::Instant;
use tokio::{fs, io::AsyncWriteExt, sync::mpsc};
use tracing::error;

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

        // Create stream to validate keys concurrently (two-stage pipeline)
        let cache_api_url = self.config.cache_api_url();
        let valid_keys_stream = stream
            .map(|key| {
                test_generate_content_api(
                    self.client.clone(),
                    self.full_url.clone(),
                    key,
                    self.config.clone(),
                )
            })
            .buffer_unordered(self.config.concurrency)
            .filter_map(|result| async { result.ok() })
            .map(|validated_key| {
                test_cache_content_api(self.client.clone(), cache_api_url.clone(), validated_key)
            })
            .buffer_unordered(self.config.concurrency);
        pin_mut!(valid_keys_stream);

        // Open output files for writing keys by tier (fixed filenames)
        let free_keys_path = "freekey.txt";
        let paid_keys_path = "paidkey.txt";

        let free_file = fs::File::create(&free_keys_path).await?;
        let paid_file = fs::File::create(&paid_keys_path).await?;

        let mut free_buffer_writer = tokio::io::BufWriter::new(free_file);
        let mut paid_buffer_writer = tokio::io::BufWriter::new(paid_file);

        // Process validated keys and write to appropriate tier files
        while let Some(valid_key) = valid_keys_stream.next().await {
            if let Err(e) = write_validated_key_to_tier_files(
                &mut free_buffer_writer,
                &mut paid_buffer_writer,
                &valid_key,
            )
            .await
            {
                error!("Failed to write key to output file: {e}");
            }
        }

        // Flush both buffers to ensure all data is written to files
        free_buffer_writer.flush().await?;
        paid_buffer_writer.flush().await?;

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
