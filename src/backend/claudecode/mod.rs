mod client;

use anyhow::Error;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

use crate::backend::events::ExecutionEvent;
use crate::backend::{AgentInfo, Backend, Session, SessionConfig};

use client::ClaudeCodeClient;

pub struct ClaudeCodeBackend {
    id: String,
    name: String,
    client: ClaudeCodeClient,
}

impl ClaudeCodeBackend {
    pub fn new() -> Self {
        Self {
            id: "claudecode".to_string(),
            name: "Claude Code".to_string(),
            client: ClaudeCodeClient::new(),
        }
    }
}

#[async_trait]
impl Backend for ClaudeCodeBackend {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    async fn list_agents(&self) -> Result<Vec<AgentInfo>, Error> {
        Ok(vec![])
    }

    async fn create_session(&self, config: SessionConfig) -> Result<Session, Error> {
        Ok(Session {
            id: self.client.create_session_id(),
            directory: config.directory,
            model: config.model,
            agent: config.agent,
        })
    }

    async fn send_message_streaming(
        &self,
        session: &Session,
        _message: &str,
    ) -> Result<(mpsc::Receiver<ExecutionEvent>, JoinHandle<()>), Error> {
        let (tx, rx) = mpsc::channel(4);
        let session_id = session.id.clone();
        let handle = tokio::spawn(async move {
            let _ = tx
                .send(ExecutionEvent::Error {
                    message: "Claude Code backend is not configured".to_string(),
                })
                .await;
            let _ = tx
                .send(ExecutionEvent::MessageComplete { session_id })
                .await;
        });
        Ok((rx, handle))
    }
}

pub fn registry_entry() -> Arc<dyn Backend> {
    Arc::new(ClaudeCodeBackend::new())
}
