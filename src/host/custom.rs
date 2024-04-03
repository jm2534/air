use std::path::PathBuf;

use crate::{Message, Provider};

pub struct Custom {
    url: PathBuf,
}

impl Custom {
    pub fn new(url: PathBuf) -> Self {
        Self { url }
    }
}

impl Provider for Custom {
    fn send(
        &self,
        context: &[Message],
        client: &reqwest::blocking::Client,
    ) -> Result<Message, reqwest::Error> {
        let response = client
            .post(self.url.to_str().unwrap())
            .json(context)
            .send()?;

        Ok(Message::assistant(response.text()?))
    }
}
