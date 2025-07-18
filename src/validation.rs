use anyhow::Result;
use async_stream::stream;
use futures::{pin_mut, stream::StreamExt};
use reqwest::Client;
use std::time::Instant;
use tokio::{fs, io::AsyncWriteExt, sync::mpsc};

use crate::adapters::write_keys_txt_file;
use crate::config::KeyCheckerConfig;
use crate::key_validator::validate_key_with_retry;
use crate::types::GeminiKey;

pub struct ValidationService {
    config: KeyCheckerConfig,
    client: Client,
}

impl ValidationService {
    pub fn new(config: KeyCheckerConfig, client: Client) -> Self {
        Self { config, client }
    }

    pub async fn validate_keys(&self, keys: Vec<GeminiKey>) -> Result<()> {
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
            .map(|key| validate_key_with_retry(self.client.to_owned(), self.config.api_host.clone(), key))
            .buffer_unordered(self.config.concurrency)
            .filter_map(|r| async { r });
        pin_mut!(valid_keys_stream);

        // Open output file for writing valid keys
        let output_file = fs::File::create(&self.config.output_path).await?;
        let mut buffer_writer = tokio::io::BufWriter::new(output_file);

        // Process validated keys and write to output file
        while let Some(valid_key) = valid_keys_stream.next().await {
            println!("Valid key found: {}", valid_key.as_ref());
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
