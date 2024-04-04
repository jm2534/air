use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt::Display};

use super::Usage;
use crate::{Message, Provider};

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

#[derive(Deserialize, Serialize)]
struct ResponseChoice {
    index: usize,
    message: HashMap<String, String>,
    finish_reason: String,
    logprobs: Option<serde_json::Value>,
}

/// Response from the OpenAI API for (de)serialization purposes.
#[derive(Deserialize, Serialize)]
struct OpenAIResponse {
    id: Option<String>,
    created: Option<u64>,
    model: Option<String>,
    choices: Vec<ResponseChoice>,
    usage: Usage,
}

impl Provider for OpenAI {
    fn send(
        &self,
        context: &[Message],
        client: &reqwest::blocking::Client,
    ) -> Result<(Message, Usage), reqwest::Error> {
        let payload = serde_json::json!({
            "model": self.name,
            "messages": context,
        });
        let response = client
            .post(Self::BASE_URL)
            .json(&payload)
            .bearer_auth(&self.key)
            .send()?
            .json::<OpenAIResponse>()?;

        let text = match response.choices.first() {
            Some(choice) => choice.message["content"].as_str(),
            None => "<Empty response from server>",
        };

        Ok((Message::assistant(text.to_string()), response.usage))
    }
}
