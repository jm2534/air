use std::{fmt::Display, str::FromStr};

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
    pub fn new(role: Role, content: String) -> Self {
        Self { role, content }
    }

    pub fn user(content: String) -> Self {
        Self {
            role: Role::User,
            content,
        }
    }

    pub fn system(content: String) -> Self {
        Self {
            role: Role::System,
            content,
        }
    }

    pub fn assistant(content: String) -> Self {
        Self {
            role: Role::Assistant,
            content,
        }
    }
}

pub trait Provider: Display {
    /// Send a message and accompanying context to the model using the provided
    /// HTTP client, returning the response message and usage statistics.
    fn send(
        &self,
        context: &[Message],
        client: &blocking::Client,
    ) -> Result<(Message, Usage), reqwest::Error>;
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
