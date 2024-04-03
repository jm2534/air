use crate::{Message, Provider};

#[derive(Default)]
pub struct ClientConfig {
    model_name: Option<String>,
    max_tokens: Option<usize>,
    context: String,
}

pub struct Client {
    pub context: Vec<Message>,
    provider: Box<dyn Provider>,
    config: ClientConfig,
    http_client: reqwest::blocking::Client,
}

impl Client {
    /// Create a new client with the given model provider
    pub fn new<P: Provider + 'static>(provider: P) -> Self {
        Self {
            context: Vec::new(),
            provider: Box::new(provider),
            config: ClientConfig::default(),
            http_client: reqwest::blocking::Client::new(),
        }
    }

    pub fn with(self, config: ClientConfig) -> Self {
        Self { config, ..self }
    }

    pub fn clear(&mut self) {
        self.context.clear();
    }

    /// Send a message to the model alongside the existing context
    pub fn send(&mut self, content: String) -> anyhow::Result<&str> {
        let message = Message::user(content);
        self.context.push(message);
        let response = self.provider.send(&self.context, &self.http_client)?;
        self.context.push(response);
        Ok(&self.context.last().unwrap().content)
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
