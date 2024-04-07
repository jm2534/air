use std::fmt::Display;
use url::Url;

use super::Usage;
use crate::{Message, Provider, ProviderError};

/// A custom provider that sends messages to a prescribed HTTP endpoint.
pub struct Custom {
    url: Url,
}

impl Custom {
    pub fn new(url: Url) -> Self {
        Self { url }
    }
}

impl Display for Custom {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Custom model at {}",
            self.url.host_str().unwrap_or("unknown location")
        )
    }
}

impl Provider for Custom {
    fn send(
        &self,
        context: &[Message],
        client: &reqwest::blocking::Client,
    ) -> Result<(Message, Usage), ProviderError> {
        let response = client.post(self.url.as_str()).json(context).send()?;
        Ok((Message::assistant(response.text()?), Usage::new()))
    }

    fn models(&self, _client: &reqwest::blocking::Client) -> Result<Vec<String>, ProviderError> {
        todo!("Implement this method")
    }
}
