# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Intent

`ai-workspace` (binary name `ai-workspace`) is a local-first AI workspace manager for Linux. It is **not** a chatbot — it launches project-aware sessions that assemble context from the filesystem and hand it to a local LLM runtime (Ollama). The filesystem is always the source of truth; the app must never become a hidden store of state.

Read `docs/vision.md` for the full vision/phase breakdown before making architectural changes — it explains *why* the layering below exists. `README.md` is the user-facing overview and quick start, and `docs/user-guide.md` the full user guide; keep both in step with user-visible changes.

## Commands

```bash
cargo build                 # debug build of every crate
cargo build --release       # release build
cargo test --workspace      # run all tests of both crates
cargo test -p ai-workspace-core --test workspace_tests   # run a single integration test file
cargo test some_test_name    # run a single test by name (substring match)
cargo run -- <args>          # run the `ai-workspace` CLI, e.g. `cargo run -- status`
```

These build only the core and the CLI (`default-members`). The desktop app (`desktop/`, ADR-0009) needs WebKitGTK and Node; from `desktop/`: `npm ci`, `npm run tauri dev` to run it, and `npm run lint && npm run typecheck && npm test && npm run build`, `npm run bindings` (shell tests, regenerates `src/lib/bindings/`), `npm run e2e` (browser checks with a stub engine) to check it. See `docs/desktop.md`.

`.github/workflows/ci.yml` runs, on every pull request and push to `main`: `cargo fmt --all --check`, clippy `-D warnings` and tests for the core and CLI (`--workspace --exclude ai-workspace-desktop`), and separate desktop jobs (shell clippy and tests plus a check that the committed bindings are current; interface lint, typecheck, unit tests and build; browser checks). Run the ones covering what you changed before committing (there is no `rustfmt.toml`/`clippy.toml`, so the defaults apply). Alongside it: `audit.yml` (cargo-deny with `deny.toml`, and `npm audit` for what the app ships), `codeql.yml` and `scorecard.yml` (code scanning; skipped while the repository is private), and Dependabot (`.github/dependabot.yml`). The GitHub settings that protect the repository are listed in `docs/repository-settings.md`.

The workspace root is resolved by `WorkspacePaths::detect_with_source` in `paths.rs`: the `AI_WORKSPACE_HOME` env var, else `workspace_root` in the per-user file `~/.config/ai-workspace/config.toml` (written by `ai-workspace config set-root PATH`), else `~/AI`. Set the env var when manually testing `ai-workspace` commands to avoid touching a real workspace; tests that run the binary must also set `HOME` and `XDG_CONFIG_HOME` to a temp directory (see `crates/ai-workspace-cli/tests/root_setting_tests.rs`):

```bash
AI_WORKSPACE_HOME=/tmp/ai-test cargo run -- init
```

## Releases

A release is three Linux files of **the desktop app only**: a Fedora `.rpm`, a `.deb` (Ubuntu 24.04+, Debian 13+, Mint 22+) and an AppImage (ADR-0010, ADR-0017). `.github/workflows/release.yml` builds them when a `v<version>` tag is pushed, by calling `.github/workflows/packages.yml` (which also runs on pull requests touching the packaging and smoke-tests each file in containers), and attests them; it lands as a **draft** release the maintainer publishes. The packages and the installed command are all `ai-workspace`: Tauri's bundler (`npm run tauri build` in `desktop/`) builds them from the `bundle` section of `desktop/src-tauri/tauri.conf.json`, taking the package name from `productName` ("AI Workspace") and the binary name from `mainBinaryName` ("ai-workspace") — don't change either without changing the packages. The CLI is **not released**: its code stays, CI tests it, and it runs from source with `cargo run -- <args>`; its unused `[package.metadata.generate-rpm]` stays in `crates/ai-workspace-cli/Cargo.toml`. `npm run tauri build` writes `target/release/ai-workspace`, the same path as a release build of the CLI; the last build wins. The tag must be on `main` and equal `version` under `[workspace.package]` in the root `Cargo.toml`. See `docs/release.md`. Never push a tag or publish a release without the maintainer asking for it.

## Architecture

The repository is a **Cargo workspace with two crates** (ADR-0008 in `DECISIONS.md`):

- **`crates/ai-workspace-core`** (library `ai_workspace_core`) — all engine behavior. It returns data and **never prints, reads the terminal, or names the command**: no `println!`/`print!`/stdin, no `COMMAND_NAME`. Cargo prevents it from depending on the CLI.
- **`crates/ai-workspace-cli`** (package `ai-workspace-cli`, binary `ai-workspace`) — argument parsing, rendering, prompts, usage text. Not released since ADR-0010 (a developer tool run with `cargo run`), but still tested and kept working.
- **`desktop/`** — the Tauri desktop app (ADR-0009), and the only thing released, as the `ai-workspace` package and command (ADR-0010): `desktop/src-tauri` is crate `ai-workspace-desktop` (a workspace member but not a default member), `desktop/src` the React + TypeScript interface. Commands in `commands.rs` are thin: one core call each, converted to wire types in `dto.rs`, from which `ts-rs` generates `desktop/src/lib/bindings/` (never edit those by hand; `npm run bindings` regenerates them). `desktop/src/lib/engine.ts` is the only file that calls `invoke`. It covers what the CLI does (ADR-0011): read-only views, chat, creating projects, permissions grant/revoke, choosing the workspace folder (closes an open chat session first) and init. The desktop names no CLI command and keeps no workspace state; its only own state is the theme choice in the webview's local storage.

Version, edition and license live once, in `[workspace.package]` in the root `Cargo.toml`. Inside each crate the modules are flat. `PROJECR_STRUCTURE.md` describes an *aspirational* nested layout (`core/`, `project/`, `permissions/`, ...) that does not exist — don't assume those folders, and don't invent them speculatively. `ARCHITECTURE.md` (see its Code Layout section) is the reliable reference.

Core modules (dependencies flow one direction; only `paths`/`workspace`/`config` touch the filesystem):

- **`engine.rs`** — `Engine`, the front-end API: one method per workspace operation (`init`, `projects`, `archived_projects`, `config`, `validate`, `create_project`, `project_details`, `set_project_model`, `archive_project`, `restore_project`, `clone_project`, `rename_project`, `delete_project`, `grant`, `revoke`, `permissions`, `search`) plus `set_workspace_root`. Each returns its result as data. Runs the validation gate and collects permission-scoped search candidates. This is "Workspace Core" from `ARCHITECTURE.md`.
- **`session.rs`** — `Session`, one interactive project session: `open` (default grants, first context from the project's description, runtime), `summary`, `send` (re-derives context from each message, returns the reply text), `draft_note` (asks the model for a session note without adding to the history), `close` (revokes temporary grants). Saving a note is `Engine::save_session_note`, only on the person's request (ADR-0012). Owns the `RuntimeManager`; dropping an unclosed session still closes its permission session and stops a runtime it started.
- **`error.rs`** — `AppError` + `Result<T>` used everywhere, and `GatedOperation`. Every fallible path returns a typed variant with a front-end-neutral `Display`. `ValidationGateBlocked` carries the operation and the full `ValidationResult`. Avoid `unwrap()`/`expect()` outside tests.
- **`paths.rs`** — `WorkspacePaths` resolves the workspace layout (`Base/`, `Projects/`, `Cache/`, `Logs/`, `Config/`) and creates it (`ensure_layout`); the per-user settings file; canonicalizing permission paths. All OS-specific folder lookups (`HOME`, XDG) sit in its small `platform` module.
- **`config.rs`** — loads/writes `Config/workspace.toml` (`WorkspaceConfig`).
- **`workspace.rs`** — the largest module: project discovery/creation, project metadata (`project.toml`), workspace **validation** (`validate_workspace`; issues carry a `SuggestedFix` naming an `Action` such as `Init`, never a command line), and the entire **permission session** lifecycle (grant/revoke/view, idempotent-by-canonical-path, persisted to `Projects/<project>/Session/permissions.toml`, audit-logged to `Logs/permissions-events.log`). Also provides `collect_markdown_files`/`scan_markdown_files`, which apply the built-in secret block list (`looks_like_secret`, ADR-0012) so secret-looking names never reach context or search.
- **`context.rs`** — the Context Builder. `readable_sources` is the one definition of what the model may read (ADR-0014): Markdown in all of Base and the whole selected project folder, plus the session's active temporary grants (a file or a folder), with the secret block list applied; search uses it too. Merges those sources into a `ContextSnapshot` (base/project/session/granted files) within a `ContextBudget`: from the model's `num_ctx` in a chat session (`ContextBudget::for_window`, ADR-0012), else the defaults `DEFAULT_MAX_CONTEXT_SOURCES` (8) and `DEFAULT_MAX_CONTEXT_BYTES` (12000). A file that does not fit is cut with a visible `[truncated: N of M bytes shown]` marker (`truncated_files`). With a query it orders sources by relevance (`search::order_by_relevance`); without one, fixed priority project > base > session. The LLM never reads files directly.
- **`search.rs`** — the Search Engine: dependency-free keyword scoring, ranking and snippets over an already permission-filtered candidate list. `rank_candidates` (standalone search) and `order_by_relevance` (Context Builder).
- **`runtime.rs`** — `RuntimeManager` starts/detects/stops Ollama (port 11434): `ensure_runtime` uses whatever already answers, else starts `ollama serve` or, with `runtime = "podman"` in `workspace.toml`, the Ollama image as the container `ai-workspace-ollama` (`podman_run_args`; never pulls, localhost only, ADR-0016); what it started is stopped on `Drop`. Chat replies are streamed (`stream_chat_message` with a per-piece callback; `send_chat_message` is the same with no callback), with a connect timeout and a per-piece read timeout but no whole-answer limit (ADR-0013). `Session::send_streaming` passes pieces on; the desktop's `chat_send` forwards them over a Tauri `Channel<String>`. The one runtime behind `ARCHITECTURE.md`'s Runtime abstraction.

The lower modules are public because the core's integration tests drive them directly; front ends should use `Engine`/`Session` and the data types they return.

CLI modules:

- **`app.rs`** — hand-rolled argument parsing (no clap), usage text, dispatch to `Engine`, the interactive session loop over `Session`, and `report_error` (prints a refused gate's blocking issues, then `error: ...`; rewords errors whose core text isn't CLI vocabulary).
- **`ui.rs`** — all rendering and terminal prompts. Pure presentation; turns core `Action`s into command lines. Output goes through `out!`/`outln!` (defined in `lib.rs`), which don't panic on a closed pipe.
- **`lib.rs`** — defines `COMMAND_NAME` (`"ai-workspace"`), the single source for the command's name. Build every user-facing string that names the command from it; never write the name as a literal.

### Validation gate

`ai-workspace validate` is always runnable and reports issues but never blocks. `ai-workspace project create`, `ai-workspace project show`, `ai-workspace project set-model`, all `ai-workspace permission *` commands, and the desktop's project clone and rename run the validation gate first (`Engine::enforce_validation_gate` in `engine.rs`) and refuse to run while blocking (`Error`-severity) issues exist. When adding a new mutating command, decide whether it needs the same gate.

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
- Keep filesystem access confined to the core's `paths.rs`/`workspace.rs`/`config.rs`. Don't add `std::fs` calls in `engine.rs`, `session.rs`, `runtime.rs`, or the CLI crate's `src/`.
- Keep printing and terminal input in the CLI crate. New engine behavior goes in the core and returns data; front ends (CLI, desktop) only convert and render.

## Testing Guidelines (from `CONTRIBUTING.md`)

- Add tests when a change affects behavior, workflows, validation, or safety — not for every tiny refactor or rename.
- Cover at least one happy path and the critical failure/safety path (e.g. an invalid permission path, a blocked validation gate).
- The workspace format is frozen for 1.x (ADR-0016, `docs/workspace-format.md`): `crates/ai-workspace-core/tests/fixtures/workspace-1.0/` was written by 1.0 and `format_tests.rs` must keep reading it. Never edit the fixture to make a test pass; new keys need defaults, and unknown keys stay ignored.
- Tests that only use the library live in `crates/ai-workspace-core/tests/` and build their own temp `WorkspacePaths` (see `workspace_tests.rs`, `engine_tests.rs`). Tests that run the binary live in `crates/ai-workspace-cli/tests/` and use `assert_cmd`; set `AI_WORKSPACE_HOME`, `HOME` and `XDG_CONFIG_HOME` to temp directories so the real workspace and `~/.config` are never touched (see `root_setting_tests.rs`).

## Spec-Kit Workflow

This repo uses a "speckit" spec-driven workflow: numbered feature specs live under `specs/NNN-feature-name/` (each with `spec.md`, `plan.md`, `tasks.md`, `data-model.md`, `research.md`, `quickstart.md`, `contracts/`, `checklists/`). `.specify/` holds the templates and scripts driving this, and `.github/prompts/speckit.*.prompt.md` / `.github/agents/speckit.*.agent.md` define the slash-command-style agents (`/speckit.specify`, `/speckit.plan`, `/speckit.tasks`, `/speckit.implement`, etc.) used to drive each phase. When asked to implement a new versioned feature, check whether a corresponding `specs/NNN-*/` directory already exists and follow its `plan.md`/`tasks.md` rather than improvising.

Current shipped versions: v0.1 (Foundation) → v0.2 (Workspace Manager) → v0.3 (Permissions) → v0.4 (Context Builder) → v0.5 (Filesystem Search) → v0.6 (Release packaging) → v0.7 (Desktop app, the only released package) → v0.8 (Project agents) → v0.9 (Project management) → v0.10 (Documentation, no release) → v1.0 (Stable release; current `Cargo.toml` version). See `ROADMAP.md` for what's next.
