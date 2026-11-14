# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Intent

`ai-workspace` (binary name `ai-workspace`) is a local-first AI workspace manager for Fedora. It is **not** a chatbot — it launches project-aware sessions that assemble context from the filesystem and hand it to a local LLM runtime (Ollama). The filesystem is always the source of truth; the app must never become a hidden store of state.

Read `README.md` for the full vision/phase breakdown before making architectural changes — it explains *why* the layering below exists.

## Commands

```bash
cargo build                 # debug build
cargo build --release       # release build
cargo test                  # run all tests (unit + tests/*.rs integration tests)
cargo test --test workspace_tests   # run a single integration test file
cargo test some_test_name    # run a single test by name (substring match)
cargo run -- <args>          # run the `ai-workspace` CLI, e.g. `cargo run -- status`
```

There is no lint/format CI config in this repo (no `rustfmt.toml`/`clippy.toml`); still run `cargo fmt` and `cargo clippy` before committing since `rust.instructions.md` requires idiomatic, panic-free code.

The workspace root is resolved by `WorkspacePaths::detect_with_source` in `paths.rs`: the `AI_WORKSPACE_HOME` env var, else `workspace_root` in the per-user file `~/.config/ai-workspace/config.toml` (written by `ai-workspace config set-root PATH`), else `~/AI`. Set the env var when manually testing `ai-workspace` commands to avoid touching a real workspace; tests that run the binary must also set `HOME` and `XDG_CONFIG_HOME` to a temp directory (see `tests/root_setting_tests.rs`):

```bash
AI_WORKSPACE_HOME=/tmp/ai-test cargo run -- init
```

## Releases

A release is a Fedora `.rpm` built by `.github/workflows/release.yml` when a `v<version>` tag is pushed; it lands as a **draft** release the maintainer publishes. The package is described under `[package.metadata.generate-rpm]` in `Cargo.toml` and built with `cargo generate-rpm --auto-req builtin` (`cargo-generate-rpm` 0.21.1). The tag must be on `main` and equal `version` in `Cargo.toml`. See `docs/release.md`. Never push a tag or publish a release without the maintainer asking for it.

## Architecture

The codebase is currently a **flat module structure** under `src/` (`app.rs`, `config.rs`, `context.rs`, `error.rs`, `paths.rs`, `runtime.rs`, `session.rs`, `ui.rs`, `workspace.rs`, all re-exported from `lib.rs`). Note: `PROJECR_STRUCTURE.md` describes an *aspirational* nested-folder layout (`core/`, `project/`, `permissions/`, `search/`, `context/`, `ui/`); the real code has not been split into those subdirectories yet — don't assume the folders exist, and don't invent them speculatively. `ARCHITECTURE.md` describes the same layering conceptually and is the more reliable reference for responsibilities.

Module responsibilities (dependencies flow one direction — UI never contains business logic, and only `paths`/`workspace`/`config` touch the filesystem):

- **`error.rs`** — `AppError` enum + `Result<T>` alias used everywhere. Every fallible path returns a typed variant (`ConfigMissing`, `ProjectNotFound`, `ValidationGateBlocked`, `PermissionDenied`, etc.) with a human-readable `Display` impl. Avoid `unwrap()`/`expect()` outside tests.
- **`paths.rs`** — `WorkspacePaths` resolves the `~/AI` layout (`Base/`, `Projects/`, `Cache/`, `Logs/`, `Config/`) from `AI_WORKSPACE_HOME` or `$HOME`, and creates it (`ensure_layout`). Also owns canonicalizing/validating permission paths.
- **`config.rs`** — loads/writes `Config/workspace.toml` (`WorkspaceConfig`), the human-editable source of workspace-wide settings (default project subdirectories, metadata filename, etc.).
- **`workspace.rs`** — the largest module; owns project discovery/creation (`Projects/<name>/`), project metadata (`project.toml`), workspace **validation** (`validate_workspace`, gates other commands), and the entire **permission session** lifecycle (grant/revoke/view, idempotent-by-canonical-path, session-scoped, persisted to `Projects/<project>/Session/permissions.toml`, audit-logged to `Logs/permissions-events.log`). Also provides `collect_markdown_files` used by the context builder.
- **`context.rs`** — the Context Builder (v0.4, extended in v0.5). Merges Base + Project + Session markdown into a single `ContextSnapshot`, capped by `DEFAULT_MAX_CONTEXT_SOURCES` (8) and `DEFAULT_MAX_CONTEXT_BYTES` (12000). `build_context_snapshot` takes an optional `query: Option<&str>`: with a query it orders sources by relevance (via `search::order_by_relevance`, priority order as tie-break only); without one (only when no project is selected) it falls back to the v0.4 fixed priority order project > base > session. The LLM never reads files directly — everything goes through this snapshot.
- **`search.rs`** — the Search Engine (v0.5, `ARCHITECTURE.md` Layer 5). Simple, dependency-free case-insensitive/whole-word keyword scoring, ranking, and snippet extraction over an already permission-filtered candidate list. Owns no filesystem traversal or permission logic — callers (`session.rs`) collect candidates via `workspace.rs`'s existing APIs first. Exposes `rank_candidates` (drops non-matches, caps to a limit — used by the standalone search command) and `order_by_relevance` (keeps every candidate, only reorders — used by the Context Builder, so a workspace that fits under the caps still gets everything regardless of relevance).
- **`runtime.rs`** — `RuntimeManager` starts/detects/stops Ollama (port 11434) as a child process; killed on `Drop`. This is the one runtime implementation behind what `ARCHITECTURE.md` calls the Runtime abstraction — expect future runtimes (LM Studio, vLLM) to need this interface generalized, not the caller changed.
- **`ui.rs`** — all `println!`/prompt rendering for the CLI. Pure presentation; takes data structures from `workspace`/`context`/`search`/`runtime`/`config` and prints them. Never put filesystem or decision logic here.
- **`app.rs`** — CLI argument parsing (hand-rolled, no clap) and command dispatch table.
- **`lib.rs`** — re-exports the modules and defines `COMMAND_NAME` (`"ai-workspace"`), the single source for the command's name. Build every user-facing string that names the command (help, usage errors, hints, the version line) from it; never write the name as a literal.
- **`session.rs`** — orchestration layer called by `app.rs`: wires `paths` + `config` + `workspace` + `context` + `search` + `runtime` + `ui` together per command (`run_session`, `create_managed_project`, `grant_permission`, `run_search`, etc.). This is "Workspace Core" from `ARCHITECTURE.md`. `run_session` seeds the session-start context query from the project's description; `run_chat_loop` re-derives context from the user's message on every turn.

### Validation gate

`ai-workspace validate` is always runnable and reports issues but never blocks. `ai-workspace project create`, `ai-workspace project show`, and all `ai-workspace permission *` commands call `enforce_validation_gate` first (see `session.rs`) and refuse to run while blocking (`Error`-severity) issues exist. When adding a new mutating command, decide whether it needs the same gate.

### Permission model specifics

- Two categories: `Default` (Base + selected project, always granted) and `Temporary` (explicit `ai-workspace permission grant --path`).
- Grants are idempotent by **canonical path**; invalid/nonexistent paths are denied with an actionable error and logged as a `deny` event.
- A previously-granted path that later disappears stays visible with `Unavailable` status rather than being silently dropped.
- All permission state changes are appended as structured events (`grant`/`deny`/`revoke`/`status-change`) to `Logs/permissions-events.log` — see `docs/permissions.md` for the full behavior spec.

## Working Rules (from `AGENTS.md` / `.github/copilot-instructions.md`)

These apply regardless of which assistant is editing this repo:

- **Documentation before implementation is a strict rule.** Start new capabilities with a spec (see `specs/NNN-*/spec.md` for the pattern) or clear written intent before writing code.
- **One meaningful feature per pull request** — also a strict rule.
- If a change alters architecture, update `ARCHITECTURE.md` **and** record the decision in `DECISIONS.md` (ADR format, see existing entries).
- Ask a clarifying question instead of guessing when behavior, scope, permissions, or architecture are underspecified — this is a hard rule, not a suggestion.
- Do not introduce cloud dependencies unless explicitly requested; the project is offline/local-first by design (no telemetry, no automatic writes, no hidden background services — see `SECURITY.md`).
- Prefer Rust for implementation, Markdown for knowledge/docs, TOML for configuration — don't introduce other formats/languages without reason.
- Keep filesystem access confined to `paths.rs`/`workspace.rs`/`config.rs`. Don't add `std::fs` calls in `ui.rs`, `app.rs`, or `runtime.rs`.

## Testing Guidelines (from `CONTRIBUTING.md`)

- Add tests when a change affects behavior, workflows, validation, or safety — not for every tiny refactor or rename.
- Cover at least one happy path and the critical failure/safety path (e.g. an invalid permission path, a blocked validation gate).
- Integration tests in `tests/*.rs` use `assert_cmd` to invoke the real `ai-workspace` binary end-to-end; set `AI_WORKSPACE_HOME` to a temp directory in new tests to avoid touching the real workspace (see existing tests in `tests/config_tests.rs`, `tests/workspace_tests.rs`, `tests/context_tests.rs` for the pattern).

## Spec-Kit Workflow

This repo uses a "speckit" spec-driven workflow: numbered feature specs live under `specs/NNN-feature-name/` (each with `spec.md`, `plan.md`, `tasks.md`, `data-model.md`, `research.md`, `quickstart.md`, `contracts/`, `checklists/`). `.specify/` holds the templates and scripts driving this, and `.github/prompts/speckit.*.prompt.md` / `.github/agents/speckit.*.agent.md` define the slash-command-style agents (`/speckit.specify`, `/speckit.plan`, `/speckit.tasks`, `/speckit.implement`, etc.) used to drive each phase. When asked to implement a new versioned feature, check whether a corresponding `specs/NNN-*/` directory already exists and follow its `plan.md`/`tasks.md` rather than improvising.

Current shipped versions: v0.1 (Foundation) → v0.2 (Workspace Manager) → v0.3 (Permissions) → v0.4 (Context Builder) → v0.5 (Filesystem Search) → v0.6 (Release packaging, current `Cargo.toml` version). See `ROADMAP.md` for what's next.
