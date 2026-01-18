use uuid::Uuid;

pub struct ClaudeCodeClient;

impl ClaudeCodeClient {
    pub fn new() -> Self {
        Self
    }

    pub fn create_session_id(&self) -> String {
        Uuid::new_v4().to_string()
    }
}
