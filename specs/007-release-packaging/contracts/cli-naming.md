# Contract: Command Name

Covers FR-003 and FR-004 (see spec.md). The command is `local-ai`. Only the name changes; every subcommand, flag, exit code, and output format is otherwise identical to v0.5.

## Name

| Thing | v0.5 | v0.6 |
| --- | --- | --- |
| Command (binary) | `ai` | `local-ai` |
| Cargo binary target | `ai` | `local-ai` |
| Crate / library | `ai-workspace` / `ai_workspace` | unchanged |
| Workspace override variable | `AI_WORKSPACE_HOME` | unchanged |
| Default workspace folder | `~/AI` | unchanged |

There is no `ai` alias: neither the package nor `cargo build` produces a command called `ai`.

## Single source

`src/lib.rs` defines `pub const COMMAND_NAME: &str = "local-ai";`. Every string that names the command is built from it. No source file contains the command name as a literal inside a user-facing message.

## Version line

```text
local-ai 0.6.0
```

`local-ai --version` prints `COMMAND_NAME`, a space, and the project version (`CARGO_PKG_VERSION`), then a newline. Exit code `0`.

## Help text

`local-ai --help` keeps the v0.5 layout with the name replaced on every usage line:

```text
AI Workspace

Usage:
  local-ai           Launch an interactive workspace session
  local-ai init      Create the standard ~/AI directory layout
  local-ai status    Show discovered workspaces and root paths
  ...
  local-ai search --query TEXT [--project PROJECT] [--limit N]
  local-ai --version Show the local-ai binary version
  local-ai --help    Show this help message

Environment:
  AI_WORKSPACE_HOME  Override the default workspace root (~/AI)
```

The product title line stays `AI Workspace`.

## Other messages that name the command

All of these use `COMMAND_NAME`:

- Usage errors — `usage: local-ai <subcommand> ...` (in `app.rs` and `error.rs`).
- Unknown-command error — points at `local-ai --help`.
- Validation-gate labels and hints — `local-ai project create`, ``Run `local-ai validate` ...`` (in `session.rs`, `ui.rs`).
- Validation suggested fixes — ``run `local-ai init` ...`` (in `workspace.rs`).
- No-results hint in search — ``check `local-ai permission view` ...`` (in `ui.rs`).
- Empty-workspace hint — `... and rerun local-ai.` (in `session.rs`).

## Test obligations

- `--version` prints exactly `local-ai <version>`.
- `--help` output contains `local-ai` usage lines and no line matching the old form (two spaces, `ai`, space).
- Every existing integration test invokes the binary as `local-ai` and still passes with its assertions unchanged except where they quote the command name.
