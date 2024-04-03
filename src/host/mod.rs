mod custom;
mod openai;
pub use custom::Custom;
pub use openai::OpenAI;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default)]
pub struct Usage {
    prompt_tokens: Option<u64>,
    completion_tokens: Option<u64>,
    pub total_tokens: Option<u64>,
}

impl Usage {
    /// Empty usage object
    pub fn new() -> Self {
        Self::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_usage_defaults() {
        let usage = Usage::new();
        assert_eq!(usage.prompt_tokens, None);
        assert_eq!(usage.completion_tokens, None);
        assert_eq!(usage.total_tokens, None);
    }
}
