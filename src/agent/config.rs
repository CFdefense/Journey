/*
 * src/agent/config.rs
 *
 * File for Agent Configuration
 *
 * Purpose:
 *   Store Agent Configuration
 */

pub struct AgentConfig {
	pub model: String,
	pub temperature: f32,
	pub max_tokens: u32,
}