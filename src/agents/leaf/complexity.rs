//! Complexity estimation agent.
//!
//! Analyzes a task description and estimates:
//! - Complexity score (0-1)
//! - Whether to split into subtasks
//! - Estimated token count

use async_trait::async_trait;
use serde_json::json;

use crate::agents::{
    Agent, AgentContext, AgentId, AgentResult, AgentType, Complexity, LeafAgent, LeafCapability,
};
use crate::llm::{ChatMessage, Role};
use crate::task::Task;

/// Agent that estimates task complexity.
/// 
/// # Purpose
/// Given a task description, estimate how complex it is and whether
/// it should be split into subtasks.
/// 
/// # Algorithm
/// 1. Send task description to LLM with complexity evaluation prompt
/// 2. Parse LLM response for complexity score and reasoning
/// 3. Return structured Complexity object
pub struct ComplexityEstimator {
    id: AgentId,
}

impl ComplexityEstimator {
    /// Create a new complexity estimator.
    pub fn new() -> Self {
        Self { id: AgentId::new() }
    }

    /// Prompt template for complexity estimation.
    /// 
    /// # Response Format
    /// LLM should respond with JSON containing:
    /// - score: float 0-1
    /// - reasoning: string explanation
    /// - estimated_tokens: int
    /// - subtasks: optional array if should split
    fn build_prompt(&self, task: &Task) -> String {
        format!(
            r#"You are a task complexity analyzer. Analyze the following task and estimate its complexity.

Task: {}

Respond with a JSON object containing:
- "score": A float from 0.0 to 1.0 where:
  - 0.0-0.2: Trivial (single command, simple file operation)
  - 0.2-0.4: Simple (few steps, straightforward implementation)
  - 0.4-0.6: Moderate (multiple files, some decision making)
  - 0.6-0.8: Complex (many files, architectural decisions, testing)
  - 0.8-1.0: Very Complex (large refactoring, many dependencies)
  
- "reasoning": Brief explanation of why this complexity level

- "estimated_tokens": Estimated total tokens needed (input + output) to complete this task

- "should_split": Boolean, true if task should be broken into subtasks

- "subtasks": If should_split is true, array of suggested subtask descriptions

Respond with ONLY the JSON object, no other text."#,
            task.description()
        )
    }

    /// Parse LLM response into Complexity struct.
    /// 
    /// # Postconditions
    /// - Returns valid Complexity with score in [0, 1]
    /// - Falls back to moderate complexity on parse error
    fn parse_response(&self, response: &str) -> Complexity {
        // Try to parse as JSON
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(response) {
            let score = json["score"].as_f64().unwrap_or(0.5);
            let reasoning = json["reasoning"].as_str().unwrap_or("No reasoning provided");
            let estimated_tokens = json["estimated_tokens"].as_u64().unwrap_or(2000);
            
            return Complexity::new(score, reasoning, estimated_tokens);
        }

        // Try to extract score from text
        if let Some(score) = self.extract_score_from_text(response) {
            return Complexity::new(score, response, 2000);
        }

        // Default to moderate complexity
        Complexity::moderate("Could not parse complexity response")
    }

    /// Try to extract a score from free-form text.
    fn extract_score_from_text(&self, text: &str) -> Option<f64> {
        // Look for patterns like "0.5" or "score: 0.5" or "50%"
        let text_lower = text.to_lowercase();
        
        // Check for keywords
        if text_lower.contains("trivial") || text_lower.contains("very simple") {
            return Some(0.1);
        }
        if text_lower.contains("very complex") || text_lower.contains("extremely") {
            return Some(0.9);
        }
        if text_lower.contains("complex") {
            return Some(0.7);
        }
        if text_lower.contains("moderate") || text_lower.contains("medium") {
            return Some(0.5);
        }
        if text_lower.contains("simple") || text_lower.contains("easy") {
            return Some(0.3);
        }

        None
    }
}

impl Default for ComplexityEstimator {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Agent for ComplexityEstimator {
    fn id(&self) -> &AgentId {
        &self.id
    }

    fn agent_type(&self) -> AgentType {
        AgentType::ComplexityEstimator
    }

    fn description(&self) -> &str {
        "Estimates task complexity and recommends splitting strategy"
    }

    /// Estimate complexity of a task.
    /// 
    /// # Returns
    /// AgentResult with Complexity data in the `data` field.
    async fn execute(&self, task: &mut Task, ctx: &AgentContext) -> AgentResult {
        let prompt = self.build_prompt(task);
        
        let messages = vec![
            ChatMessage {
                role: Role::System,
                content: Some("You are a precise task complexity analyzer. Respond only with JSON.".to_string()),
                tool_calls: None,
                tool_call_id: None,
            },
            ChatMessage {
                role: Role::User,
                content: Some(prompt),
                tool_calls: None,
                tool_call_id: None,
            },
        ];

        // Use a fast, cheap model for complexity estimation
        let model = "openai/gpt-4.1-mini";
        
        match ctx.llm.chat_completion(model, &messages, None).await {
            Ok(response) => {
                let content = response.content.unwrap_or_default();
                let complexity = self.parse_response(&content);
                
                // Estimate cost (rough: ~1000 tokens for this request)
                let cost_cents = 1; // Very cheap operation
                
                AgentResult::success(
                    format!(
                        "Complexity: {:.2} - {}",
                        complexity.score(),
                        if complexity.should_split() { "Should split" } else { "Execute directly" }
                    ),
                    cost_cents,
                )
                .with_model(model)
                .with_data(json!({
                    "score": complexity.score(),
                    "reasoning": complexity.reasoning(),
                    "should_split": complexity.should_split(),
                    "estimated_tokens": complexity.estimated_tokens(),
                }))
            }
            Err(e) => {
                // On error, return moderate complexity as fallback
                let fallback = Complexity::moderate(format!("LLM error, using fallback: {}", e));
                
                AgentResult::success(
                    "Using fallback complexity estimate due to LLM error",
                    0,
                )
                .with_data(json!({
                    "score": fallback.score(),
                    "reasoning": fallback.reasoning(),
                    "should_split": fallback.should_split(),
                    "estimated_tokens": fallback.estimated_tokens(),
                    "fallback": true,
                }))
            }
        }
    }
}

impl LeafAgent for ComplexityEstimator {
    fn capability(&self) -> LeafCapability {
        LeafCapability::ComplexityEstimation
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_json_response() {
        let estimator = ComplexityEstimator::new();
        
        let json_response = r#"{"score": 0.7, "reasoning": "Complex task", "estimated_tokens": 3000, "should_split": true}"#;
        let complexity = estimator.parse_response(json_response);
        
        assert!((complexity.score() - 0.7).abs() < 0.01);
        assert!(complexity.should_split());
    }

    #[test]
    fn test_parse_text_response() {
        let estimator = ComplexityEstimator::new();
        
        let text_response = "This is a very complex task";
        let complexity = estimator.parse_response(text_response);
        
        assert!(complexity.score() > 0.6);
    }
}

