use anyhow::Result;
use clap::Parser;
use figment::{
    Figment,
    providers::{Env, Format, Serialized, Toml},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use std::sync::LazyLock;
use url::Url;

/// Cli arguments
#[derive(Parser, Debug, Serialize, Deserialize)]
struct Cli {
    #[arg(short = 'i', long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    input_path: Option<PathBuf>,

    #[arg(short = 'o', long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    output_path: Option<PathBuf>,

    #[arg(short = 'b', long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    backup_path: Option<PathBuf>,

    #[arg(short = 'u', long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    api_host: Option<Url>,

    #[arg(short = 't', long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    timeout_sec: Option<u64>,

    #[arg(short = 'c', long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    concurrency: Option<usize>,

    #[arg(short = 'x', long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    proxy: Option<Url>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyCheckerConfig {
    // Input file path containing API keys to check.
    #[serde(default)]
    pub input_path: PathBuf,

    // Output file path for valid API keys.
    #[serde(default)]
    pub output_path: PathBuf,

    // Backup file path for all API keys.
    #[serde(default)]
    pub backup_path: PathBuf,

    // API host URL for key validation.
    #[serde(default = "default_api_host")]
    pub api_host: Url,

    // Request timeout in seconds.
    #[serde(default)]
    pub timeout_sec: u64,

    // Maximum number of concurrent requests.
    #[serde(default)]
    pub concurrency: usize,

    // Optional proxy URL for HTTP requests (e.g., --proxy http://user:pass@host:port).
    #[serde(default)]
    pub proxy: Option<Url>,
}

impl Default for KeyCheckerConfig {
    fn default() -> Self {
        (*DEFAULT_CONFIG).clone()
    }
}
impl KeyCheckerConfig {
    pub fn load_config() -> Result<Self> {
        // Define the path to the configuration file
        static CONFIG_PATH: LazyLock<PathBuf> = LazyLock::new(|| "Config.toml".into());

        // Check if config.toml exists, if not create it with default values
        if !CONFIG_PATH.exists() {
            let default_config = Self::default();
            let toml_content = toml::to_string_pretty(&default_config)?;
            fs::write(CONFIG_PATH.as_path(), toml_content)?;
        }

        // Load configuration from config.toml, environment variables, and CLI arguments
        let config: Self = Figment::new()
            .merge(Serialized::defaults(Self::default()))
            .merge(Toml::file(CONFIG_PATH.as_path()))
            .merge(Env::prefixed("KEYCHECKER_"))
            .merge(Serialized::defaults(Cli::parse()))
            .extract()?;

        Ok(config)
    }

    /// Returns the complete Gemini API URL for generateContent endpoint
    pub fn gemini_api_url(&self) -> Url {
        self.api_host
            .join("v1beta/models/gemini-2.0-flash-exp:generateContent")
            .expect("Failed to join API URL")
    }
}

// Single LazyLock for entire default configuration
static DEFAULT_CONFIG: LazyLock<KeyCheckerConfig> = LazyLock::new(|| KeyCheckerConfig {
    input_path: "keys.txt".into(),
    output_path: "output_keys.txt".into(),
    backup_path: "backup_keys.txt".into(),
    api_host: Url::parse("https://generativelanguage.googleapis.com/").unwrap(),
    timeout_sec: 15,
    concurrency: 50,
    proxy: None,
});

// LazyLock for the test message body used in API key validation
pub static TEST_MESSAGE_BODY: LazyLock<Value> = LazyLock::new(|| {
    serde_json::json!({
        "contents": [
            {
                "parts": [
                    {
                        "text": "Hi"
                    }
                ]
            }
        ]
    })
});

fn default_api_host() -> Url {
    DEFAULT_CONFIG.api_host.clone()
}
