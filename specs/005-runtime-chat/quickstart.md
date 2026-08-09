# Quickstart: v0.4.1 Runtime Chat Loop

## Prerequisites

- Fedora-compatible Linux environment.
- Rust toolchain installed (`cargo`, `rustc`).
- Ollama installed with at least one model pulled (e.g. `ollama pull qwen3.6:latest`) for the happy-path scenarios.
- Repository checkout with the v0.4.1 spec and plan artifacts.

## Setup

1. Build the binary:
   - `cargo build`
2. Initialize a scratch workspace (use a temp `AI_WORKSPACE_HOME` to avoid touching `~/AI`):
   - `AI_WORKSPACE_HOME=/tmp/ai-test cargo run -- init`
3. Create a project:
   - `AI_WORKSPACE_HOME=/tmp/ai-test cargo run -- project create --name demo-chat`
4. Add some project knowledge so the AI has context to draw on:
   - `printf '# Demo notes\n\nThis project tracks a weather CLI written in Rust.\n' > /tmp/ai-test/Projects/demo-chat/Knowledge/notes.md`

## Validation Scenarios

### Scenario 1: Ask a context-grounded question

1. Ensure Ollama is running (`ollama serve`, or let the session start it).
2. Start a session:
   - `AI_WORKSPACE_HOME=/tmp/ai-test cargo run -- `
3. At the chat prompt, ask: `what does this project track?`
4. Expected: the response references the weather CLI content from `notes.md`.

### Scenario 2: Follow-up question in the same session

1. Continuing Scenario 1's session, ask a follow-up: `what language is it written in?`
2. Expected: the response answers correctly without the user having to repeat the project description, showing the earlier turn was carried forward.

### Scenario 3: End the session

1. Type `exit` at the chat prompt.
2. Expected: the session closes, temporary permissions are revoked, and the existing "Session cleanup" message is shown — matching pre-existing shutdown behavior.

### Scenario 4: No Ollama available

1. Stop Ollama and ensure it is not on `PATH`, or use an environment where it isn't installed.
2. Start a session:
   - `AI_WORKSPACE_HOME=/tmp/ai-test cargo run -- `
3. Expected: the session shows the existing static summary and "Press Enter to close the workspace" prompt, with no chat prompt offered — identical to v0.4 behavior.

### Scenario 5: A failed turn does not end the session

1. Start a session with Ollama running, then stop Ollama mid-session (e.g. `pkill ollama`) without closing the CLI session.
2. Ask a question.
3. Expected: an error is shown explaining the request failed; the chat prompt appears again so the user can retry or type `exit`.

## Verification Commands

- `cargo test`
- `AI_WORKSPACE_HOME=/tmp/ai-test cargo run -- validate`
- `AI_WORKSPACE_HOME=/tmp/ai-test cargo run -- `
