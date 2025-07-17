use crate::adapters::output::write_keys_to_file;
use crate::types::GeminiKey;
use anyhow::Result;
use std::{collections::HashSet, fs, path::Path, str::FromStr};

/// Load and validate API keys from a file
/// Returns a vector of unique, valid API keys
pub fn load_keys(path: &Path) -> Result<Vec<GeminiKey>> {
    let keys_txt = fs::read_to_string(path)?;
    // Use HashSet to automatically deduplicate keys
    let unique_keys_set: HashSet<&str> = keys_txt
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect();

    let mut keys = Vec::new();
    let mut valid_keys_for_backup = Vec::new();

    for key_str in unique_keys_set {
        match GeminiKey::from_str(key_str) {
            Ok(api_key) => {
                keys.push(api_key.clone());
                valid_keys_for_backup.push(api_key.inner.clone());
            }
            Err(e) => eprintln!("Skipping invalid key: {}", e),
        }
    }

    // Write validated keys to backup.txt
    if let Err(e) = write_keys_to_file(&valid_keys_for_backup, "backup.txt") {
        eprintln!("Failed to write backup file: {}", e);
    }

    Ok(keys)
}
