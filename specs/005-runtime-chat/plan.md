# Implementation Plan: v0.4.1 Runtime Chat Loop

**Branch**: `005-runtime-chat` | **Date**: 2026-08-05 | **Spec**: /specs/005-runtime-chat/spec.md

**Input**: Feature specification from `/specs/005-runtime-chat/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command; its definition describes the execution workflow.

## Summary

Wire the v0.4 Context Builder's assembled context into an actual local AI response by having the Runtime Manager send chat requests to Ollama's `/api/chat` endpoint. The session flow becomes a multi-turn chat loop: the assembled context seeds the conversation as a system message, the user asks questions, and each turn (plus history) is sent to Ollama and the reply is shown. Sessions where Ollama is unavailable keep today's exact fallback behavior. The model name resolves from a new per-project override falling back to a new workspace-wide default.

## Technical Context

**Language/Version**: Rust 2024 edition

**Primary Dependencies**: Existing `src/context.rs`, `src/runtime.rs`, `src/session.rs`, `src/ui.rs`, `src/config.rs`, `src/workspace.rs`, `src/error.rs`. New external crates: `ureq` (blocking HTTP client, no TLS feature needed for localhost-only traffic) and `serde_json` (chat request/response bodies).

**Storage**: Local filesystem only for configuration (`workspace.toml`, `project.toml`); chat history is in-memory for the duration of one session and is never persisted, matching the existing "temporary session state stays temporary" constraint.

**Testing**: `cargo test`, including a new `tests/runtime_tests.rs` integration test that mocks Ollama's HTTP response via a local `TcpListener` (no live Ollama required for automated tests), matching the repository's existing convention of integration-style tests in `tests/*.rs` rather than inline `#[cfg(test)]` modules; manual quickstart validation against a real local Ollama installation.

**Target Platform**: Fedora-compatible Linux terminals

**Project Type**: Local CLI application

**Performance Goals**: A single chat turn should complete within the time the local model normally takes to generate a response; no additional overhead beyond the HTTP round-trip itself.

**Constraints**: Local-first and offline-capable (Ollama runs on `127.0.0.1` only, no cloud fallback); must not require network access beyond localhost; must not break sessions when Ollama is absent; must not persist chat history to disk; project isolation must be preserved (per-project model override lives in that project's own `project.toml`).

**Scale/Scope**: Single-user local workflow, one active chat session at a time, conversation length bounded by what one interactive session in the terminal is expected to carry.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- Filesystem source of truth: PASS (chat history is intentionally session-scoped and not persisted, matching the constitution's "temporary session state must remain temporary"; model configuration is stored in the existing `workspace.toml`/`project.toml` files)
- Local-first and Fedora-native constraints: PASS (Ollama traffic stays on `127.0.0.1`; no cloud dependency introduced)
- Documentation-first flow: PASS (this spec/plan precede the code changes)
- Behavior/scope/permission ambiguities resolved: PASS (multi-turn chat loop vs. one-shot, and model-config placement, were confirmed with the user before planning)
- Architecture update required: **REQUIRED** — this is the first time Runtime Manager sends application data (chat messages) to Ollama over HTTP, rather than only managing its process lifecycle and doing a raw TCP readiness check. `ARCHITECTURE.md` (Layer 6 Runtime Manager, Runtime Abstraction section) and `DECISIONS.md` (new ADR) must be updated alongside this feature.
- Layer boundaries preserved in plan: PASS — Context Builder (`context.rs`) frames what the AI sees (`as_system_prompt`), Runtime Manager (`runtime.rs`) only transports messages, Workspace Core (`session.rs`) orchestrates the loop and owns conversation state, UI (`ui.rs`) stays limited to stdin/stdout rendering.
- Independently testable stories: PASS (each user story in spec.md has an independent test)

Post-Design Re-Check (after Phase 1 artifacts):

- research.md aligns with keeping the codebase synchronous and Ollama traffic localhost-only: PASS
- data-model.md preserves the Context Builder / Runtime Manager / Workspace Core separation: PASS
- contracts/ollama-chat-api.md documents the exact wire contract Runtime Manager depends on, keeping that dependency explicit and swappable: PASS
- quickstart.md validates the happy path, the no-Ollama fallback, and a failed-turn retry without introducing scope beyond spec.md: PASS

## Project Structure

### Documentation (this feature)

```text
specs/005-runtime-chat/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   └── ollama-chat-api.md
├── checklists/
│   └── requirements.md
└── tasks.md
```

### Source Code (repository root)

```text
src/
├── app.rs
├── config.rs
├── context.rs
├── error.rs
├── lib.rs
├── main.rs
├── paths.rs
├── runtime.rs
├── session.rs
├── ui.rs
└── workspace.rs

tests/
├── app_version_tests.rs
├── config_tests.rs
├── context_tests.rs
├── runtime_tests.rs
└── workspace_tests.rs
```

**Structure Decision**: Keep the existing single Rust CLI project structure. No new modules — the feature extends `context.rs` (system-prompt framing), `runtime.rs` (chat transport), `session.rs` (loop orchestration and model resolution), `ui.rs` (chat I/O rendering), `config.rs`/`workspace.rs` (model configuration fields), and `error.rs` (one new error variant).

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

| Violation | Why Needed | Simpler Alternative Rejected Because |
| -- | -- | -- |
| New runtime dependencies (`ureq`, `serde_json`) | Ollama's API is JSON-over-HTTP; the codebase has no HTTP client today | Hand-rolling raw TCP + HTTP/1.1 framing (as `runtime.rs` already does for the readiness check) was rejected for the chat call because it would mean reimplementing chunked/content-length handling and JSON escaping by hand, adding more code and risk than one small, synchronous, dependency-light crate |
