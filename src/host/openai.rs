use super::Usage;
use crate::{Message, Provider, ProviderError, ProviderResponse};
use std::fmt::Display;

/// A provider that sends messages to the OpenAI API.
pub struct OpenAI {
    name: String,
    key: String,
}

impl OpenAI {
    const BASE_URL: &'static str = "https://api.openai.com/v1/chat/completions";

    pub fn new<S: Into<String>>(name: S, key: S) -> Self {
        Self {
            name: name.into(),
            key: key.into(),
        }
    }
}

impl Display for OpenAI {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "OpenAI {}", self.name)
    }
}

impl Provider for OpenAI {
    fn send(
        &self,
        context: &[Message],
        client: &reqwest::blocking::Client,
    ) -> Result<(Message, Usage), ProviderError> {
        let payload = serde_json::json!({
            "model": self.name,
            "messages": context,
        });

        let response = client
            .post(Self::BASE_URL)
            .json(&payload)
            .bearer_auth(&self.key)
            .send()?
            .error_for_status()?
            .json::<ProviderResponse>()?;

        self.parse(response)
    }
}
