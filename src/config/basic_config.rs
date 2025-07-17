use anyhow::{Ok, Result};
use clap::Parser;
use figment::{
    Figment,
    providers::{Env, Format, Serialized, Toml},
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::LazyLock;
use url::Url;

#[derive(Debug, Serialize, Deserialize, Parser)]
pub struct KeyCheckerConfig {
    // Input file path containing API keys to check.
    #[serde(default)]
    #[arg(short, long)]
    input_path: Option<PathBuf>,

    // Output file path for valid API keys.
    #[serde(default)]
    #[arg(short, long)]
    output_path: Option<PathBuf>,

    // Backup file path for all API keys.
    #[serde(default)]
    #[arg(short, long)]
    backup_path: Option<PathBuf>,

    // API host URL for key validation.
    #[serde(default)]
    #[arg(short, long)]
    api_host: Option<Url>,

    // Request timeout in seconds.
    #[serde(default)]
    #[arg(short, long)]
    timeout_sec: Option<u64>,

    // Maximum number of concurrent requests.
    #[serde(default)]
    #[arg(short, long)]
    concurrency: Option<usize>,

    // Optional proxy URL for HTTP requests (e.g., --proxy http://user:pass@host:port).
    #[serde(default)]
    #[arg(short, long)]
    proxy: Option<Url>,
}

impl Default for KeyCheckerConfig {
    fn default() -> Self {
        Self {
            input_path: Some(default_input_path()),
            output_path: Some(default_output_path()),
            backup_path: Some(default_backup_path()),
            api_host: Some(default_api_host()),
            timeout_sec: Some(default_timeout()),
            concurrency: Some(default_concurrency()),
            proxy: None,
        }
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

        // Load configuration from config.toml, environment variables, and defaults
        let mut figment = Figment::new()
            .merge(Serialized::defaults(Self::default()))
            .merge(Toml::file(CONFIG_PATH.as_path()))
            .merge(Env::prefixed("KEYCHECKER_"));

        // Only merge non-None command line arguments
        let cli_args = Self::parse();
        if let Some(input_path) = cli_args.input_path {
            figment = figment.merge(("input_path", input_path));
        }
        if let Some(output_path) = cli_args.output_path {
            figment = figment.merge(("output_path", output_path));
        }
        if let Some(backup_path) = cli_args.backup_path {
            figment = figment.merge(("backup_path", backup_path));
        }
        if let Some(api_host) = cli_args.api_host {
            figment = figment.merge(("api_host", api_host));
        }
        if let Some(timeout_sec) = cli_args.timeout_sec {
            figment = figment.merge(("timeout_sec", timeout_sec));
        }
        if let Some(concurrency) = cli_args.concurrency {
            figment = figment.merge(("concurrency", concurrency));
        }
        if let Some(proxy) = cli_args.proxy {
            figment = figment.merge(("proxy", proxy));
        }

        let config = figment.extract()?;

        println!("Final loaded config: {:?}", config);

        Ok(config)
    }
    pub fn input_path(&self) -> PathBuf {
        self.input_path.clone().unwrap_or_else(default_input_path)
    }
    pub fn output_path(&self) -> PathBuf {
        self.output_path.clone().unwrap_or_else(default_output_path)
    }
    pub fn backup_path(&self) -> PathBuf {
        self.backup_path.clone().unwrap_or_else(default_backup_path)
    }
    pub fn api_host(&self) -> Url {
        self.api_host.clone().unwrap_or_else(default_api_host)
    }
    pub fn timeout_sec(&self) -> u64 {
        self.timeout_sec.unwrap_or_else(default_timeout)
    }
    pub fn concurrency(&self) -> usize {
        self.concurrency.unwrap_or_else(default_concurrency)
    }
    pub fn proxy(&self) -> Option<Url> {
        self.proxy.clone()
    }
}

fn default_input_path() -> PathBuf {
    "keys.txt".into()
}

fn default_output_path() -> PathBuf {
    "output_keys.txt".into()
}
fn default_backup_path() -> PathBuf {
    "backup_keys.txt".into()
}
fn default_api_host() -> Url {
    Url::parse("https://generativelanguage.googleapis.com/").unwrap()
}
fn default_timeout() -> u64 {
    20
}
fn default_concurrency() -> usize {
    30
}
