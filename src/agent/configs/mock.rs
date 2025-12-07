/*
 * src/agent/configs/mock.rs
 *
 * Mock LLM implementation for testing
 */

use async_trait::async_trait;
use futures::stream::{self, Stream};
use langchain_rust::language_models::GenerateResult;
use langchain_rust::language_models::LLMError;
use langchain_rust::language_models::llm::LLM;
use langchain_rust::schemas::{Message, StreamData};
use serde_json::Value;
use std::pin::Pin;

/// Mock LLM implementation for testing that returns dummy responses
/// without making actual API calls
#[derive(Clone)]
pub struct MockLLM;

#[async_trait]
impl LLM for MockLLM {
	async fn generate(&self, _messages: &[Message]) -> Result<GenerateResult, LLMError> {
		Ok(GenerateResult {
			generation: "This is a mock response for testing.".to_string(),
			tokens: None,
		})
	}

	async fn stream(
		&self,
		_messages: &[Message],
	) -> Result<Pin<Box<dyn Stream<Item = Result<StreamData, LLMError>> + Send>>, LLMError> {
		let response = StreamData::new(
			Value::String("This is a mock response for testing.".to_string()),
			None,
			"This is a mock response for testing.",
		);
		let stream = stream::once(async move { Ok(response) });
		Ok(Box::pin(stream))
	}
}
