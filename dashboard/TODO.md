# TODO (Open Agent / oh-my-opencode agent mismatch)

## What I did (in this sandbox)
- Identified root cause: mission workspaces sync `.opencode/oh-my-opencode.json` from the **Library**. `LibraryStore::get_opencode_settings` only copied the system config when the Library file is **missing**. If the Library already had an older file, new agents (e.g., `prometheus`) from updated oh-my-opencode never propagate, so `sisyphus` works while `prometheus` fails.
- Implemented a fix in a writable copy of the backend repo at:
  - `/Users/thomas/work/open_agent/dashboard/_backend_copy/src/library/mod.rs`
  - The fix **merges missing agents from the system oh-my-opencode.json into the Library** (preserves library overrides). If the Library file is empty, it prefers the system config. It also writes back the merged Library file and logs a message.
- Added a unit test in the same file to lock the behavior:
  - `merges_missing_agents_from_system_config`
- Verified the unit test passes in the copy:
  - `cargo test merges_missing_agents_from_system_config` (in `/Users/thomas/work/open_agent/dashboard/_backend_copy`)

## What I could NOT do here
- No outbound network / SSH: `ssh root@95.216.112.253` fails with “Operation not permitted.”
- Cannot call the real MISSION_API endpoints (same network block).
- Cannot write to `/Users/thomas/work/open_agent` from this sandbox (write perms restricted to `/Users/thomas/work/open_agent/dashboard`).

## What is left to do (once you restart me with SSH permissions)

### 1) Apply the fix to the real backend repo
Copy the changed file into the real repo (or apply a diff):
```bash
cp /Users/thomas/work/open_agent/dashboard/_backend_copy/src/library/mod.rs \
   /Users/thomas/work/open_agent/src/library/mod.rs
```
Then run the test in the real repo:
```bash
cd /Users/thomas/work/open_agent
cargo test merges_missing_agents_from_system_config
```

### 2) Deploy to Thomas backend
```bash
cd /Users/thomas/work/open_agent
cargo build
scp -i ~/.ssh/agent.thomas.md target/debug/open_agent root@95.216.112.253:/usr/local/bin/
ssh -i ~/.ssh/agent.thomas.md root@95.216.112.253 "systemctl restart open_agent"
```

### 3) Verify with MISSION_API on Thomas backend
- Login to get JWT:
```bash
TOKEN=$(curl -s https://agent-backend.thomas.md/api/auth/login \
  -H 'Content-Type: application/json' \
  -d '{"password":"3knzssZU7cIJMdKOqhskuoNM"}' | jq -r .token)
```
- Choose a workspace:
```bash
curl -s https://agent-backend.thomas.md/api/workspaces \
  -H "Authorization: Bearer $TOKEN"
```
- Create+load missions with `agent: "sisyphus"` and `agent: "prometheus"` (backend: opencode), then send a message to each:
```bash
# Example for prometheus
MISSION_ID=$(curl -s https://agent-backend.thomas.md/api/control/missions \
  -H "Authorization: Bearer $TOKEN" -H 'Content-Type: application/json' \
  -d '{"title":"Prometheus test","workspace_id":"<uuid>","agent":"prometheus","backend":"opencode"}' | jq -r .id)

curl -s https://agent-backend.thomas.md/api/control/missions/$MISSION_ID/load \
  -H "Authorization: Bearer $TOKEN"

curl -s https://agent-backend.thomas.md/api/control/message \
  -H "Authorization: Bearer $TOKEN" -H 'Content-Type: application/json' \
  -d '{"content":"Say hello","agent":"prometheus"}'
```
- Confirm both agents work and that the Library’s `opencode/oh-my-opencode.json` now contains `prometheus` after sync.

## Notes
- The fix is limited to Library agent propagation. It does not change oh-my-opencode installation logic.
- The test is self-contained and uses a temp dir + `OPENCODE_CONFIG_DIR` to simulate system config.
