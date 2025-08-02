use gemini_keychecker::error::ValidatorError;
use gemini_keychecker::{BANNER, config::KeyCheckerConfig, validation::start_validation};
use mimalloc::MiMalloc;
use tracing::info;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

/// Main function - displays banner and starts validation service
#[tokio::main]
async fn main() -> Result<(), ValidatorError> {
    let config = KeyCheckerConfig::load_config()?;
    // Initialize tracing with professional format using configured log level
    tracing_subscriber::fmt()
        .with_target(false)
        .with_level(true)
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(&config.log_level)),
        )
        .init();
    // Display banner and configuration status at startup
    println!("{BANNER}");
    info!("Configuration loaded: {}", config);

    // Start validation service
    start_validation().await
}
