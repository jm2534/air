use std::fmt::Display;

use crate::{Message, Provider};

#[derive(Default)]
pub struct ClientConfig {
    model_name: Option<String>,
    max_tokens: Option<usize>,
}

/// A client for interacting with a model provider. `Client`s maintain a context
/// for each conversation, sending messages to the provider for processing. See
/// the `Provider` trait for more information on implementing a model provider.
///
/// # Examples
///
/// ```
/// use air::client::Client;
/// use air::Message;
/// use air::host::OpenAI;
///
/// let model = OpenAI::new("gpt-3.5-turbo", "my-api-key");
/// let mut client = Client::new(model);
/// let message = Message::user("What is the meaning of life?");
/// let answer = client.send(message);
/// ```
pub struct Client {
    pub context: Vec<Message>,
    pub tokens_sent: Option<u64>,
    provider: Box<dyn Provider>,
    config: ClientConfig,
    http_client: reqwest::blocking::Client,
}

impl Display for Client {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.provider)
    }
}

impl Client {
    /// Create a new client with the given model provider
    pub fn new<P: Provider + 'static>(provider: P) -> Self {
        Self {
            tokens_sent: Some(0),
            context: Vec::new(),
            provider: Box::new(provider),
            config: ClientConfig::default(),
            http_client: reqwest::blocking::Client::new(),
        }
    }

    pub fn with_context(mut self, context: Vec<Message>) -> Self {
        self.context = context;
        self
    }

    pub fn with(mut self, config: ClientConfig) -> Self {
        self.config = config;
        self
    }

    pub fn clear(&mut self) {
        self.context.clear();
    }

    /// Send a message to the model alongside the existing context
    pub fn send(&mut self, content: Message) -> anyhow::Result<&Message> {
        self.context.push(content);
        let (message, usage) = self.provider.send(&self.context, &self.http_client)?;

        self.context.push(message);
        self.tokens_sent = match (usage.total_tokens, self.tokens_sent) {
            (_, None) | (None, Some(_)) => None,
            (Some(x), Some(y)) => Some(x + y),
        };

        let model_response = self.context.last().unwrap();
        Ok(model_response)
    }
}

#[cfg(test)]
mod tests {
    use crate::host::OpenAI;

    use super::*;

    #[test]
    fn test_client_context_init() {
        let name = String::from("gpt-3.5-turbo");
        let key = String::from("api-key");
        let client = Client::new(OpenAI::new(name, key));
        assert!(client.context.is_empty());
    }
}
