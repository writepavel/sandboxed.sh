//! Task executor agent - the main worker that uses tools.
//!
//! This is a refactored version of the original agent loop,
//! now as a leaf agent in the hierarchical tree.

use async_trait::async_trait;
use serde_json::json;

use crate::agents::{
    Agent, AgentContext, AgentId, AgentResult, AgentType, LeafAgent, LeafCapability,
};
use crate::llm::{ChatMessage, Role, ToolCall};
use crate::task::Task;
use crate::tools::ToolRegistry;

/// Agent that executes tasks using tools.
/// 
/// # Algorithm
/// 1. Build system prompt with available tools
/// 2. Call LLM with task description
/// 3. If LLM requests tool call: execute, feed back result
/// 4. Repeat until LLM produces final response or max iterations
/// 
/// # Budget Management
/// - Tracks token usage and costs
/// - Stops if budget is exhausted
pub struct TaskExecutor {
    id: AgentId,
}

impl TaskExecutor {
    /// Create a new task executor.
    pub fn new() -> Self {
        Self { id: AgentId::new() }
    }

    /// Build the system prompt for task execution.
    fn build_system_prompt(&self, workspace: &str, tools: &ToolRegistry) -> String {
        let tool_descriptions = tools
            .list_tools()
            .iter()
            .map(|t| format!("- **{}**: {}", t.name, t.description))
            .collect::<Vec<_>>()
            .join("\n");

        format!(
            r#"You are an autonomous task executor with access to tools. 
You operate in the workspace: {workspace}

## Available Tools
{tool_descriptions}

## Rules
1. Use tools to accomplish the task - don't just describe what to do
2. Read files before editing them
3. Verify your work when possible
4. If stuck, explain what's blocking you
5. When done, summarize what you accomplished

## Response
When task is complete, provide a clear summary of:
- What you did
- Files created/modified
- How to verify the result"#,
            workspace = workspace,
            tool_descriptions = tool_descriptions
        )
    }

    /// Execute a single tool call.
    async fn execute_tool_call(
        &self,
        tool_call: &ToolCall,
        ctx: &AgentContext,
    ) -> anyhow::Result<String> {
        let args: serde_json::Value = serde_json::from_str(&tool_call.function.arguments)
            .unwrap_or(serde_json::Value::Null);

        ctx.tools
            .execute(&tool_call.function.name, args, &ctx.workspace)
            .await
    }

    /// Run the agent loop for a task.
    async fn run_loop(
        &self,
        task: &Task,
        model: &str,
        ctx: &AgentContext,
    ) -> (String, u64, Vec<String>) {
        let mut total_cost = 0u64;
        let mut tool_log = Vec::new();

        // Build initial messages
        let system_prompt = self.build_system_prompt(&ctx.workspace_str(), &ctx.tools);
        let mut messages = vec![
            ChatMessage {
                role: Role::System,
                content: Some(system_prompt),
                tool_calls: None,
                tool_call_id: None,
            },
            ChatMessage {
                role: Role::User,
                content: Some(task.description().to_string()),
                tool_calls: None,
                tool_call_id: None,
            },
        ];

        // Get tool schemas
        let tool_schemas = ctx.tools.get_tool_schemas();

        // Agent loop
        for iteration in 0..ctx.max_iterations {
            tracing::debug!("TaskExecutor iteration {}", iteration + 1);

            // Check budget
            let remaining = task.budget().remaining_cents();
            if remaining == 0 && total_cost > 0 {
                return (
                    "Budget exhausted before task completion".to_string(),
                    total_cost,
                    tool_log,
                );
            }

            // Call LLM
            let response = match ctx.llm.chat_completion(model, &messages, Some(&tool_schemas)).await {
                Ok(r) => r,
                Err(e) => {
                    return (
                        format!("LLM error: {}", e),
                        total_cost,
                        tool_log,
                    );
                }
            };

            // Estimate cost (rough: ~1 cent per request for cheap models)
            total_cost += 2;

            // Check for tool calls
            if let Some(tool_calls) = &response.tool_calls {
                if !tool_calls.is_empty() {
                    // Add assistant message with tool calls
                    messages.push(ChatMessage {
                        role: Role::Assistant,
                        content: response.content.clone(),
                        tool_calls: Some(tool_calls.clone()),
                        tool_call_id: None,
                    });

                    // Execute each tool call
                    for tool_call in tool_calls {
                        tool_log.push(format!(
                            "Tool: {} Args: {}",
                            tool_call.function.name,
                            tool_call.function.arguments
                        ));

                        let result = match self.execute_tool_call(tool_call, ctx).await {
                            Ok(output) => output,
                            Err(e) => format!("Error: {}", e),
                        };

                        // Add tool result
                        messages.push(ChatMessage {
                            role: Role::Tool,
                            content: Some(result),
                            tool_calls: None,
                            tool_call_id: Some(tool_call.id.clone()),
                        });
                    }

                    continue;
                }
            }

            // No tool calls - final response
            if let Some(content) = response.content {
                return (content, total_cost, tool_log);
            }

            // Empty response
            return (
                "LLM returned empty response".to_string(),
                total_cost,
                tool_log,
            );
        }

        (
            format!("Max iterations ({}) reached", ctx.max_iterations),
            total_cost,
            tool_log,
        )
    }
}

impl Default for TaskExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Agent for TaskExecutor {
    fn id(&self) -> &AgentId {
        &self.id
    }

    fn agent_type(&self) -> AgentType {
        AgentType::TaskExecutor
    }

    fn description(&self) -> &str {
        "Executes tasks using tools (file ops, terminal, search, etc.)"
    }

    async fn execute(&self, task: &mut Task, ctx: &AgentContext) -> AgentResult {
        // Use default model or from context
        let model = &ctx.config.default_model;

        let (output, cost, tool_log) = self.run_loop(task, model, ctx).await;

        // Update task budget
        let _ = task.budget_mut().try_spend(cost);

        AgentResult::success(&output, cost)
            .with_model(model)
            .with_data(json!({
                "tool_calls": tool_log.len(),
                "tools_used": tool_log,
            }))
    }
}

impl LeafAgent for TaskExecutor {
    fn capability(&self) -> LeafCapability {
        LeafCapability::TaskExecution
    }
}

