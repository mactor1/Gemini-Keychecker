use crate::types::GeminiKey;
use anyhow::Result;
use std::{fs, io::Write};
use tokio::io::{AsyncWriteExt, BufWriter};
use toml::Value;

// Write valid key to output file
pub async fn write_keys_txt_file(
    file: &mut BufWriter<tokio::fs::File>,
    key: &GeminiKey,
) -> Result<()> {
    file.write_all(format!("{}\n", key.as_ref()).as_bytes()).await?;
    Ok(())
}

// Write valid key to output file in Clewdr format
pub fn write_keys_clewdr_format(file: &mut fs::File, key: &GeminiKey) -> Result<()> {
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
pub fn write_keys_to_file(keys: &[String], filename: &str) -> Result<()> {
    let content = keys.join("\n");
    fs::write(filename, content)?;
    println!("File '{}' created with {} keys", filename, keys.len());
    Ok(())
}
