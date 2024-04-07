use super::Usage;
use crate::{Message, Provider, ProviderError, ProviderResponse};

use serde::{Deserialize, Serialize};
use std::fmt::Display;

/// A provider that sends messages to the OpenAI API.
pub struct OpenAI {
    name: String,
    key: String,
}

impl OpenAI {
    const BASE_URL: &'static str = "https://api.openai.com/v1";

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

#[derive(Deserialize, Serialize)]
struct ModelEndpointResponse {
    object: String,
    data: Vec<ModelEndpointEntity>,
}

#[derive(Deserialize, Serialize)]
struct ModelEndpointEntity {
    // Entity name
    id: String,

    // Entity description; should be `model` for models
    object: String,

    // Entity owner; should be `openai` for OpenAI models
    owned_by: String,
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
            .post(format!("{}/chat/completions", Self::BASE_URL))
            .json(&payload)
            .bearer_auth(&self.key)
            .send()?
            .error_for_status()?
            .json::<ProviderResponse>()?;

        self.parse(response)
    }

    fn models(&self, client: &reqwest::blocking::Client) -> Result<Vec<String>, ProviderError> {
        let response = client
            .post(format!("{}/models", Self::BASE_URL))
            .bearer_auth(&self.key)
            .send()?
            .error_for_status()?
            .json::<ModelEndpointResponse>()?;

        let data = response
            .data
            .into_iter()
            .filter_map(|data| {
                if data.owned_by == "openai" && data.object == "model" {
                    Some(data.id)
                } else {
                    None
                }
            })
            .collect();

        Ok(data)
    }
}
