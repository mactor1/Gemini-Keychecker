use anyhow::Result;
use gemini_keychecker::{BANNER, config::KeyCheckerConfig, service::start_validation};

use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

/// Main function - displays banner and starts validation service
#[tokio::main]
async fn main() -> Result<()> {
    // Display banner and configuration status at startup
    println!("{BANNER}");

    let config = KeyCheckerConfig::load_config()?;
    println!("{config}");

    // Start validation service
    start_validation().await
}
