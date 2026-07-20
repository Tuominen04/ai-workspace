# CLI Contract: AI Workspace v0.1

## Command Surface

### ai
- Purpose: Start interactive local workspace session.
- Behavior:
  - Ensures workspace filesystem layout exists.
  - Discovers project workspaces.
  - Prompts user to select project (unless only one).
  - Builds Base-only AI context snapshot from ~/AI/Base.
  - Checks and reports runtime status.
  - Waits for user to end session.
- Exit behavior:
  - Exit 0 on clean session close.
  - Non-zero on startup failure or user cancellation before session activation.

### ai init
- Purpose: Create expected local workspace directory layout.
- Behavior:
  - Creates root, Base, Projects, Cache, Logs, Config and standard Base sections.
- Exit behavior:
  - Exit 0 when layout exists or is created.
  - Non-zero on filesystem errors.

### ai status
- Purpose: Print non-interactive status of root paths and discovered projects.
- Behavior:
  - Ensures layout exists.
  - Prints configured root paths and project count/list.
- Exit behavior:
  - Exit 0 on success.
  - Non-zero on filesystem errors.

### ai help / ai --help
- Purpose: Display command usage and environment variable guidance.

## Environment Variables

### AI_WORKSPACE_HOME
- Purpose: Override default workspace root (default: ~/AI).
- Type: absolute or relative filesystem path.
- Constraints:
  - Path must be local filesystem path.
  - Must be readable and writable for initialization commands.

## Context Access Policy (v0.1)
- AI-readable files are restricted to ~/AI/Base only.
- Selected project files are not included in AI context snapshot.

## Runtime Policy (v0.1)
- Runtime target is local Ollama.
- If already running, reuse and report status.
- If missing or unavailable, report degraded runtime state without crashing command startup logic by default.
