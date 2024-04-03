use crate::{Message, Provider};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub struct OpenAI {
    name: String,
    key: String,
}

impl OpenAI {
    const BASE_URL: &'static str = "https://api.openai.com/v1/chat/completions";

    pub fn new(name: String, key: String) -> Self {
        Self { name, key }
    }
}

#[derive(Deserialize, Serialize)]
struct ResponseChoice {
    index: usize,
    message: HashMap<String, String>,
    finish_reason: String,
    logprobs: Option<serde_json::Value>,
}

#[derive(Deserialize, Serialize)]
struct OpenAIResponse {
    id: String,
    created: u64,
    model: String,
    choices: Vec<ResponseChoice>,
    usage: HashMap<String, u64>,
}

impl Provider for OpenAI {
    fn send(
        &self,
        context: &[Message],
        client: &reqwest::blocking::Client,
    ) -> Result<Message, reqwest::Error> {
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

        Ok(Message::assistant(text.to_string()))
    }
}
