use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::LazyLock;
pub mod key_validator;
pub mod validation_service;

pub use key_validator::{test_cache_content_api, test_generate_content_api};
pub use validation_service::{ValidationService, start_validation};

#[derive(Serialize, Deserialize, Debug)]
pub struct TextPart {
    pub text: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ContentPart {
    pub parts: Vec<TextPart>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ThinkingConfig {
    #[serde(rename = "thinkingBudget")]
    pub thinking_budget: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GenerationConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "thinkingConfig")]
    pub thinking_config: Option<ThinkingConfig>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GeminiRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    pub contents: Vec<ContentPart>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "generationConfig")]
    pub generation_config: Option<GenerationConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttl: Option<String>,
}

// LazyLock for the test message body used in API key validation
pub static GENERATE_CONTENT_TEST_BODY: LazyLock<Value> = LazyLock::new(|| {
    let generate_request = GeminiRequest {
        model: None,
        contents: vec![ContentPart {
            parts: vec![TextPart {
                text: "Hi".to_string(),
            }],
            role: None,
        }],
        generation_config: Some(GenerationConfig {
            thinking_config: Some(ThinkingConfig { thinking_budget: 0 }),
        }),
        ttl: None,
    };
    serde_json::to_value(generate_request).unwrap()
});

// LazyLock for the cached content test body used in cache API validation
pub static CACHE_CONTENT_TEST_BODY: LazyLock<GeminiRequest> = LazyLock::new(|| {
    // Generate random text content to meet the minimum 2048 tokens requirement for cache API
    // models/gemini-2.5-flash need 1024 tokens
    // models/gemini-2.5-flash-lite need 2048 tokens
    let long_text = "You are an expert at analyzing transcripts.".repeat(128);
    GeminiRequest {
        model: Some("models/gemini-2.5-pro".to_string()),
        contents: vec![ContentPart {
            parts: vec![TextPart { text: long_text }],
            role: Some("user".to_string()),
        }],
        generation_config: None,
        ttl: Some("30s".to_string()),
    }
});
