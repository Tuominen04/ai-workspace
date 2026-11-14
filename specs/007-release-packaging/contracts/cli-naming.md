# Contract: Command Name

Covers FR-003 and FR-004 (see spec.md). The command is `ai-workspace`. Only the name changes; every subcommand, flag, exit code, and output format is otherwise identical to v0.5.

## Name

| Thing | v0.5 | v0.6 |
| --- | --- | --- |
| Command (binary) | `ai` | `ai-workspace` |
| Cargo binary target | `ai` | `ai-workspace` |
| Crate / library | `ai-workspace` / `ai_workspace` | unchanged |
| Workspace override variable | `AI_WORKSPACE_HOME` | unchanged |
| Default workspace folder | `~/AI` | unchanged |

There is no `ai` alias: neither the package nor `cargo build` produces a command called `ai`.

## Single source

`src/lib.rs` defines `pub const COMMAND_NAME: &str = "ai-workspace";`. Every string that names the command is built from it. No source file contains the command name as a literal inside a user-facing message.

## Version line

```text
ai-workspace 0.6.0
```

`ai-workspace --version` prints `COMMAND_NAME`, a space, and the project version (`CARGO_PKG_VERSION`), then a newline. Exit code `0`.

## Help text

`ai-workspace --help` keeps the v0.5 layout with the name replaced on every usage line:

```text
AI Workspace

Usage:
  ai-workspace           Launch an interactive workspace session
  ai-workspace init      Create the standard ~/AI directory layout
  ai-workspace status    Show discovered workspaces and root paths
  ...
  ai-workspace search --query TEXT [--project PROJECT] [--limit N]
  ai-workspace --version Show the ai-workspace binary version
  ai-workspace --help    Show this help message

Environment:
  AI_WORKSPACE_HOME  Override the default workspace root (~/AI)
```

The product title line stays `AI Workspace`.

## Other messages that name the command

All of these use `COMMAND_NAME`:

- Usage errors — `usage: ai-workspace <subcommand> ...` (in `app.rs` and `error.rs`).
- Unknown-command error — points at `ai-workspace --help`.
- Validation-gate labels and hints — `ai-workspace project create`, ``Run `ai-workspace validate` ...`` (in `session.rs`, `ui.rs`).
- Validation suggested fixes — ``run `ai-workspace init` ...`` (in `workspace.rs`).
- No-results hint in search — ``check `ai-workspace permission view` ...`` (in `ui.rs`).
- Empty-workspace hint — `... and rerun ai-workspace.` (in `session.rs`).

## Test obligations

- `--version` prints exactly `ai-workspace <version>`.
- `--help` output contains `ai-workspace` usage lines and no line matching the old form (two spaces, `ai`, space).
- Every existing integration test invokes the binary as `ai-workspace` and still passes with its assertions unchanged except where they quote the command name.
