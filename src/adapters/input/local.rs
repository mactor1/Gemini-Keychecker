use tracing::{error, warn};

use crate::adapters::output::write_keys_to_file;
use crate::error::ValidatorError;
use crate::types::GeminiKey;
use std::{collections::HashSet, fs, path::Path, str::FromStr};

pub fn load_keys_from_txt(path: &Path) -> Result<Vec<GeminiKey>, ValidatorError> {
    let keys_txt = fs::read_to_string(path)?;

    let keys: Vec<GeminiKey> = keys_txt
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            (!trimmed.is_empty()).then_some(trimmed)
        })
        .collect::<HashSet<_>>()
        .into_iter()
        .filter_map(|key_str| match GeminiKey::from_str(key_str) {
            Ok(api_key) => Some(api_key),
            Err(e) => {
                warn!("Skipping invalid key : {e}");
                None
            }
        })
        .collect();

    if !keys.is_empty() {
        let valid_keys_for_backup: Vec<String> = keys.iter().map(|k| k.inner.clone()).collect();
        if let Err(e) = write_keys_to_file(&valid_keys_for_backup, "backup.txt") {
            error!("Failed to write backup file: {e}");
        }
    }

    Ok(keys)
}
