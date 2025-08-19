use crate::error::ValidatorError;
use crate::types::{GeminiKey, KeyTier, ValidatedKey};
use std::{fs, io::Write};
use tokio::io::{AsyncWrite, AsyncWriteExt};
use toml::Value;
use tracing::info;

// Write valid key to appropriate tier file
pub async fn write_validated_key_to_tier_writers<W>(
    free_writer: &mut W,
    paid_writer: &mut W,
    validated_key: &ValidatedKey,
) -> Result<(), ValidatorError>
where
    W: AsyncWrite + Unpin,
{
    match validated_key.tier {
        KeyTier::Free => {
            free_writer
                .write_all(format!("{}\n", validated_key.key.as_ref()).as_bytes())
                .await?;
        }
        KeyTier::Paid => {
            paid_writer
                .write_all(format!("{}\n", validated_key.key.as_ref()).as_bytes())
                .await?;
        }
    }
    Ok(())
}

// Write valid key to output file in Clewdr format
pub fn write_keys_clewdr_format(
    file: &mut fs::File,
    key: &GeminiKey,
) -> Result<(), ValidatorError> {
    let mut table = toml::value::Table::new();
    table.insert("key".to_string(), Value::String(key.as_ref().to_string()));

    let gemini_keys = Value::Array(vec![Value::Table(table)]);
    let mut root = toml::value::Table::new();
    root.insert("gemini_keys".to_string(), gemini_keys);

    let toml_string = toml::to_string(&Value::Table(root))?;
    write!(file, "{}", toml_string)?;
    Ok(())
}

// Write keys to a text file with custom filename
pub fn write_keys_to_file(keys: &[String], filename: &str) -> Result<(), ValidatorError> {
    let content = keys.join("\n");
    fs::write(filename, content)?;
    info!("File '{}' created with {} keys", filename, keys.len());
    Ok(())
}
