use anyhow::Result;
use gemini_keychecker::{
    BANNER,
    adapters::load_keys,
    config::{KeyCheckerConfig, client_builder},
    validation::ValidationService,
};

use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

/// Main function - orchestrates the key validation process
#[tokio::main]
async fn main() -> Result<()> {
    let config = KeyCheckerConfig::load_config().unwrap();

    // Display banner and configuration status at startup
    println!("{BANNER}");
    println!("{config}");

    let keys = load_keys(config.input_path.as_path())?;
    let client = client_builder(&config)?;

    let validation_service = ValidationService::new(config, client);
    validation_service.validate_keys(keys).await?;

    Ok(())
}
