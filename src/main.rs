use gemini_keychecker::error::ValidatorError;
use gemini_keychecker::{BANNER, config::KeyCheckerConfig, validation::start_validation};
use mimalloc::MiMalloc;
use tracing::info;
use tracing_indicatif::IndicatifLayer;
use tracing_subscriber::Layer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

/// Main function - displays banner and starts validation service
#[tokio::main]
async fn main() -> Result<(), ValidatorError> {
    let config = KeyCheckerConfig::load_config()?;

    let indicatif_layer = IndicatifLayer::new();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(false)
                .with_level(true)
                .with_writer(indicatif_layer.get_stderr_writer())
                .with_ansi(false)
                .with_filter(
                    tracing_subscriber::EnvFilter::try_from_default_env()
                        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(&config.log_level)),
                ),
        )
        .with(indicatif_layer)
        .init();

    // Display banner and configuration status at startup
    println!("{BANNER}");
    info!("Configuration loaded: {}", config);

    // Start validation service
    start_validation().await
}
