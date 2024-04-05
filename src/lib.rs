use std::{collections::HashMap, fmt::Display, str::FromStr};

use enum_iterator::Sequence;
use host::Usage;
use reqwest::blocking;
use serde::{Deserialize, Serialize};

pub mod client;
pub mod host;
pub mod transcript;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Sequence)]
#[serde(rename_all = "lowercase")]
/// OpenAI-based roles for identifying message authors in a conversation
pub enum Role {
    /// A system "pre-prompt" message for guiding the model output
    System,

    /// A user prompt
    User,

    /// A response from the model
    Assistant,
}

impl FromStr for Role {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "system" => Ok(Role::System),
            "user" => Ok(Role::User),
            "assistant" => Ok(Role::Assistant),
            _ => Err(s.to_owned()),
        }
    }
}

impl Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Role::System => write!(f, "System"),
            Role::User => write!(f, "User"),
            Role::Assistant => write!(f, "Assistant"),
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Message {
    pub role: Role,
    pub content: String,
}

impl Message {
    pub fn new<S: Into<String>>(role: Role, content: S) -> Self {
        Self {
            role,
            content: content.into(),
        }
    }

    pub fn user<S: Into<String>>(content: S) -> Self {
        Self {
            role: Role::User,
            content: content.into(),
        }
    }

    pub fn system<S: Into<String>>(content: S) -> Self {
        Self {
            role: Role::System,
            content: content.into(),
        }
    }

    pub fn assistant<S: Into<String>>(content: S) -> Self {
        Self {
            role: Role::Assistant,
            content: content.into(),
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct ModelOutput {
    index: usize,
    message: HashMap<String, String>,
    finish_reason: String,
    logprobs: Option<serde_json::Value>,
}

#[derive(Deserialize, Serialize)]
pub struct ProviderResponse {
    choices: Vec<ModelOutput>,
    usage: Usage,
}

#[derive(thiserror::Error, Debug)]
pub enum ProviderError {
    #[error("Failed to submit request to the server: HTTP error {0}")]
    HttpError(reqwest::StatusCode),

    #[error("Failed to parse server response: {0}")]
    ParsingError(#[from] serde_json::Error),

    #[error("No response from the server despite successful request")]
    EmptyResponse,

    #[error("An unknown error occurred")]
    UnknownError,
}

impl From<reqwest::Error> for ProviderError {
    fn from(value: reqwest::Error) -> Self {
        match value.status() {
            Some(status) => ProviderError::HttpError(status),
            None => ProviderError::UnknownError,
        }
    }
}

pub trait Provider: Display {
    /// Helper method for `send` implementers to extract the relevant details
    /// from a provider's deserialized response object.
    fn parse(&self, mut response: ProviderResponse) -> Result<(Message, Usage), ProviderError> {
        match response.choices.first_mut() {
            None => Err(ProviderError::EmptyResponse),
            Some(choice) => {
                // avoiding a needless copy of what may be a large response
                match choice.message.remove("content") {
                    None => Err(ProviderError::EmptyResponse),
                    Some(text) => Ok((Message::assistant(text), response.usage)),
                }
            }
        }
    }

    /// Send a message and accompanying context to the model using the provided
    /// HTTP client, returning the response message and usage statistics.
    fn send(
        &self,
        context: &[Message],
        client: &blocking::Client,
    ) -> Result<(Message, Usage), ProviderError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_role_serialize_lowercase() {
        let role = Role::User;
        let serialized = serde_json::to_string(&role).unwrap();
        assert_eq!(serialized, r#""user""#);
    }

    #[test]
    fn test_message_serialize() {
        let message = Message::new(Role::User, "Hello, world!".to_string());
        let serialized = serde_json::to_string(&message).unwrap();
        assert_eq!(serialized, r#"{"role":"user","content":"Hello, world!"}"#);
    }
}
