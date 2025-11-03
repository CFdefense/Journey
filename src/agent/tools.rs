/*
 * src/agent/tools.rs
 *
 * File for Agent Tools
 *
 * Purpose:
 *   Store Agent Tools
 */

use async_trait::async_trait;
use langchain_rust::tools::Tool;
use serde_json::{json, Value};
use std::error::Error;

/// Example tool that returns a greeting message
///
/// This tool accepts a name as input and generates a friendly greeting.
/// It implements the Tool trait from langchain-rust to be used by AI agents.
#[derive(Clone)]
pub struct GreetingTool;

#[async_trait]
impl Tool for GreetingTool {
    fn name(&self) -> String {
        "greeting_tool".to_string()
    }

    fn description(&self) -> String {
        "A tool that generates a friendly greeting message. Use this when you need to greet the user or create a welcoming message."
            .to_string()
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "description": "The name of the person to greet"
                }
            },
            "required": ["name"]
        })
    }

    async fn run(&self, input: Value) -> Result<String, Box<dyn Error>> {
        let name = input["name"]
            .as_str()
            .ok_or("Name should be a string")?;
        
        Ok(format!("Hello, {}! Welcome to our AI assistant.", name))
    }
}
