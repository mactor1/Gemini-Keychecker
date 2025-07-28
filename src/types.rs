use regex::Regex;
use std::str::FromStr;
use std::sync::LazyLock;

#[derive(Debug, Clone)]
pub struct GeminiKey {
    pub inner: String,
}

impl AsRef<str> for GeminiKey {
    fn as_ref(&self) -> &str {
        &self.inner
    }
}

impl FromStr for GeminiKey {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        static RE: LazyLock<Regex> =
            LazyLock::new(|| Regex::new(r"^AIzaSy[A-Za-z0-9_-]{33}$").unwrap());

        let cleaned = s.trim();

        if RE.is_match(cleaned) {
            Ok(Self {
                inner: cleaned.to_string(),
            })
        } else {
            Err("Invalid Google API key format")
        }
    }
}
