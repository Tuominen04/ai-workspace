# Development

For working on AI Workspace itself. Building, testing and the code layout are in
[CLAUDE.md](../CLAUDE.md) (commands) and [ARCHITECTURE.md](../ARCHITECTURE.md); decisions are
recorded in [DECISIONS.md](../DECISIONS.md); how a release is made is in
[release.md](release.md); the desktop app's internals are in [desktop.md](desktop.md).

## Running from source

- **Core and command line:** `cargo build`, `cargo test --workspace`, `cargo run -- <args>`.
- **Desktop app** (needs WebKitGTK and Node): from `desktop/`, `npm ci`, then `npm run tauri dev`.

Set `AI_WORKSPACE_HOME` to a scratch folder while trying things, so your real workspace is
never touched:

```bash
AI_WORKSPACE_HOME=/tmp/ai-test cargo run -- init
```

## Command line (from source)

The command-line front end is kept in the repository and tested, but no longer
released (ADR-0010). Run it from a checkout with `cargo run -- <args>`; the
commands below are written as the binary is named, `ai-workspace`. The app does
the same things from its window (ADR-0011).

### Workspace commands

The v0.2 workspace-manager layer adds explicit commands for configuration,
project creation, project inspection, and validation:

```text
ai-workspace config show
ai-workspace project create --name NAME [--description TEXT]
ai-workspace project show PROJECT
ai-workspace project set-model PROJECT [MODEL]
ai-workspace validate
```

Validation behavior is safety-first:

- `ai-workspace validate` reports actionable workspace and project issues.
- `ai-workspace project create` and `ai-workspace project show` are enforcement-gated and refuse to
    continue while blocking validation issues exist.

### Permission commands

The v0.3 permission layer adds explicit commands for temporary session grants,
permission inspection, and manual revoke:

```text
ai-workspace permission grant [--project PROJECT] --path PATH
ai-workspace permission view [--project PROJECT] [--filter TEXT] [--page N] [--page-size N]
ai-workspace permission revoke [--project PROJECT] --path PATH
```

Permission behavior is explicit and session-scoped:

- grants are idempotent by canonical path
- invalid paths are denied with actionable errors
- unreachable granted paths remain visible as unavailable
- temporary grants are removed automatically at session end
