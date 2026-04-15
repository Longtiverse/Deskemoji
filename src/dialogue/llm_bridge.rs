//! LLM API bridge — reserved interface for future integration.
//!
//! Phase 2 (future): Add `tokio` + `reqwest` (or `ureq` for blocking)
//! and stream LLM output into the dialogue queue.

use std::fmt;

#[derive(Debug)]
pub enum LlmError {
    NotConfigured,
    NetworkError(String),
    ResponseError(String),
}

impl fmt::Display for LlmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LlmError::NotConfigured => write!(f, "LLM not configured"),
            LlmError::NetworkError(msg) => write!(f, "Network error: {msg}"),
            LlmError::ResponseError(msg) => write!(f, "Response error: {msg}"),
        }
    }
}

impl std::error::Error for LlmError {}

/// Context passed to the LLM when generating a reply.
pub struct DialogueContext {
    pub current_emoji: String,
    pub user_message: Option<String>,
    pub recent_history: Vec<String>,
    pub system_prompt: String,
}

/// Generic LLM provider trait.
pub trait LlmProvider {
    /// Blocking generation (suitable for current sync architecture).
    /// Future async version can be added alongside `tokio`.
    fn generate(&self, ctx: &DialogueContext) -> Result<String, LlmError>;
}

/// Placeholder provider that always returns `NotConfigured`.
/// Replace this with a real provider (e.g., OpenAI, Claude, local Ollama)
/// once `tokio` and an HTTP client are added.
pub struct DummyLlmProvider;

impl LlmProvider for DummyLlmProvider {
    fn generate(&self, _ctx: &DialogueContext) -> Result<String, LlmError> {
        Err(LlmError::NotConfigured)
    }
}

// Future async interface (uncomment when `tokio` is available):
// #[async_trait]
// pub trait AsyncLlmProvider {
//     async fn chat(&self, ctx: &DialogueContext) -> Result<String, LlmError>;
// }
