# Harness System

Open Agent supports multiple execution backends (harnesses) for running agent missions. This document explains the harness architecture, configuration, and how to add new backends.

## Overview

A **harness** (also called a backend) is an execution engine that runs agent missions. Open Agent currently supports:

| Harness | Description | Configuration Model |
|---------|-------------|---------------------|
| **OpenCode** | OpenCode-based execution with custom agents | Centralized (`oh-my-opencode.json`) |
| **Claude Code** | Claude CLI subprocess execution | Workspace-centric (`CLAUDE.md`, `.claude/settings.local.json`) |

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         Mission Runner                           │
│                     (src/api/mission_runner.rs)                  │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────────┐
│                       Backend Trait                              │
│                     (src/backend/mod.rs)                         │
│  - id() / name()                                                 │
│  - list_agents()                                                 │
│  - create_session()                                              │
│  - send_message_streaming()                                      │
└──────────────┬─────────────────────────────────┬────────────────┘
               │                                 │
               ▼                                 ▼
┌──────────────────────────┐    ┌──────────────────────────────────┐
│     OpenCodeBackend      │    │      ClaudeCodeBackend           │
│  (src/backend/opencode/) │    │   (src/backend/claudecode/)      │
│                          │    │                                   │
│  - HTTP/SSE to OpenCode  │    │  - Subprocess to Claude CLI      │
│  - oh-my-opencode agents │    │  - Built-in Claude agents        │
└──────────────────────────┘    └──────────────────────────────────┘
```

## Backend Trait

All backends implement the `Backend` trait defined in `src/backend/mod.rs`:

```rust
#[async_trait]
pub trait Backend: Send + Sync {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    async fn list_agents(&self) -> Result<Vec<AgentInfo>, Error>;
    async fn create_session(&self, config: SessionConfig) -> Result<Session, Error>;
    async fn send_message_streaming(
        &self,
        session: &Session,
        message: &str,
    ) -> Result<(mpsc::Receiver<ExecutionEvent>, JoinHandle<()>), Error>;
}
```

### ExecutionEvent

Backends emit a unified event stream:

| Event | Description |
|-------|-------------|
| `Thinking { content }` | Agent reasoning/thinking text |
| `TextDelta { content }` | Streaming text delta |
| `ToolCall { id, name, args }` | Tool invocation |
| `ToolResult { id, name, result }` | Tool execution result |
| `MessageComplete { session_id }` | Message/turn complete |
| `Error { message }` | Error occurred |

## OpenCode Backend

OpenCode is the default backend that communicates with an OpenCode server via HTTP/SSE.

### Configuration

**Settings page** → Backends → OpenCode:
- **Base URL**: OpenCode server endpoint (default: `http://127.0.0.1:4096`)
- **Default Agent**: Pre-selected agent for new missions
- **Permissive Mode**: Auto-allow tool permissions

**Library page** → Configs → OpenCode tab:
- Edit `oh-my-opencode.json` for agent definitions, models, and plugins
- Configure agent visibility in mission dialogs

### Agents

OpenCode agents are defined in `oh-my-opencode.json`:

```json
{
  "agents": {
    "Sisyphus": {
      "model": "anthropic/claude-opus-4-5"
    },
    "document-writer": {
      "model": "google/gemini-3-flash-preview"
    }
  }
}
```

## Claude Code Backend

Claude Code executes missions via the Claude CLI subprocess with JSON streaming.

### Configuration

**Settings page** → Backends → Claude Code:
- **API Key**: Anthropic API key (stored in secrets vault)
- **Default Model**: Model for missions (e.g., `claude-sonnet-4-20250514`)
- **CLI Path**: Path to Claude CLI executable (default: `claude` from PATH)

### Workspace Configuration

Unlike OpenCode's centralized config, Claude Code generates configuration per-workspace from your Library:

| Generated File | Source | Purpose |
|----------------|--------|---------|
| `CLAUDE.md` | `skills/*.md` | System prompt and context |
| `.claude/settings.local.json` | `mcps/`, `tools/` | MCP servers and tool permissions |

### Agents

Claude Code has built-in agents:

| Agent | Description |
|-------|-------------|
| `general-purpose` | General-purpose coding agent |
| `Bash` | Shell command specialist |
| `Explore` | Codebase exploration |
| `Plan` | Implementation planning |

### CLI Protocol

Claude Code communicates via NDJSON streaming:

```bash
echo "prompt" | claude \
  --print \
  --output-format stream-json \
  --verbose \
  --include-partial-messages \
  --dangerously-skip-permissions \
  --model "claude-sonnet-4-20250514" \
  --session-id "uuid"
```

Event types:
- `system` (init) → Session initialization
- `stream_event` → Streaming deltas
- `assistant` → Complete messages and tool calls
- `user` → Tool results
- `result` → Final completion

## Enabling/Disabling Backends

Backends can be enabled or disabled in Settings → Backends. Disabled backends:
- Don't appear in mission creation dialogs
- Don't appear in Library Configs tabs
- Cannot be selected for new missions

## Adding a New Backend

To add a new backend (e.g., Codex):

1. **Create backend module**: `src/backend/codex/mod.rs`
   - Implement `Backend` trait
   - Define event parsing and conversion

2. **Register in routes.rs**:
   ```rust
   backend_registry.write().await.register(
       crate::backend::codex::registry_entry()
   );
   ```

3. **Add API endpoints** in `src/api/backends.rs`:
   - GET/PUT config handlers
   - Secrets management

4. **Update dashboard**:
   - Add tab to Settings → Backends
   - Add tab to Library → Configs
   - Update mission creation dialog

## Mission Runner Integration

The mission runner (`src/api/mission_runner.rs`) selects the backend based on `backend_id`:

```rust
let result = match backend_id.as_str() {
    "claudecode" => run_claudecode_turn(...).await,
    "opencode" => run_opencode_turn(...).await,
    _ => Err(anyhow!("Unknown backend")),
};
```

Each backend handles its own:
- Session management
- Message execution
- Event streaming
- Error handling

## Secrets Management

Backend API keys are stored in the secrets vault:

| Backend | Secret Key |
|---------|------------|
| Claude Code | `claudecode.api_key` |
| OpenCode | Configured via AI Providers |

Access via: `secrets.get_secret("claudecode", "api_key")`

## References

- Backend trait: `src/backend/mod.rs`
- OpenCode backend: `src/backend/opencode/`
- Claude Code backend: `src/backend/claudecode/`
- Mission runner: `src/api/mission_runner.rs`
- Backend API: `src/api/backends.rs`
- Workspace config generation: `src/workspace.rs`
