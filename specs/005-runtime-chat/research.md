# Research: v0.4.1 Runtime Chat Loop

## Decision 1: Blocking HTTP client (`ureq`) instead of an async stack

- Decision: Use `ureq`, a synchronous/blocking HTTP client, for the one call Runtime Manager needs to make to Ollama's local API.
- Rationale: The entire codebase is synchronous today (no `tokio`, no `async fn` anywhere). Ollama is reached only over `127.0.0.1`, so there is no latency profile that benefits from async I/O, and pulling in an async runtime for a single local call would add a large dependency surface and a second execution model to reason about, contradicting the constitution's "simplest implementation that satisfies... goals" and "minimal, explicit technology choices" principles.
- Alternatives considered: `reqwest` + `tokio` (rejected — async runtime unjustified for one local blocking call); hand-rolled `TcpStream` + manual HTTP/1.1 framing, as `runtime.rs` already does for the readiness check (rejected — reimplementing JSON escaping and response framing by hand is more code and more risk than one small, dependency-light crate).

## Decision 2: `/api/chat` with `stream: false`, not `/api/generate` or streaming NDJSON

- Decision: Call Ollama's `POST /api/chat` endpoint with a `messages` array (`role`/`content` pairs) and `"stream": false`, receiving one JSON response object per turn.
- Rationale: `/api/chat` is designed for exactly this shape — a growing conversation with role-tagged turns — which maps directly onto the multi-turn loop confirmed with the user, and onto framing the assembled context as a single `system` message. Non-streaming keeps response handling to "parse one JSON object" instead of incrementally parsing NDJSON chunks, which is a meaningful implementation-complexity reduction for a first version.
- Alternatives considered: `/api/generate` with a single concatenated prompt string (rejected — loses the natural role separation between context, question, and answer, and would need manual prompt-templating to fake multi-turn history); streaming responses (deferred — real value for a future UX polish pass, not required to close the "context is never sent" gap, and adds incremental-parsing complexity now).

## Decision 3: Model resolution as workspace default + project override

- Decision: Add `default_model` to `WorkspaceConfig` (`workspace.toml`) and an optional `model` field to `ProjectMetadata` (`project.toml`); a session resolves its model as project override, falling back to workspace default.
- Rationale: Confirmed directly with the user. Matches the existing pattern of workspace-wide settings with project-level specifics (e.g. `default_project_directories` at the workspace level, per-project directories in `project.toml`), and keeps zero-config `ai init` working since the workspace default always has a built-in value.
- Alternatives considered: CLI flag per invocation (rejected — user would have to remember and retype it every session); workspace-only configuration with no per-project override (rejected — user explicitly wanted per-project flexibility).

## Decision 4: Chat history stays in-memory, never persisted

- Decision: The growing `ChatMessage` history for a session lives only in `session.rs`'s local state for the duration of `run_session`, and is discarded when the session ends.
- Rationale: Matches the constitution's "temporary session state must remain temporary" and the existing pattern for temporary permissions (revoked automatically at session end, never a hidden persistent grant). Nothing in spec.md's requirements calls for cross-session conversation memory.
- Alternatives considered: Persisting conversation transcripts under `Projects/<project>/Session/` (rejected for v0.4.1 — out of scope, and would need its own retention/privacy design).

## Decision 5: A single failed turn does not end the session

- Decision: On an HTTP/parse failure for one chat turn, `session.rs` reports the error via `ui.rs` and continues the loop rather than propagating the error out of `run_session`.
- Rationale: Directly required by spec.md FR-005 and the constitution's "Explain → Suggest → Recover instead of immediately terminating whenever possible" error-handling principle.
- Alternatives considered: Ending the session on any chat error (rejected — too brittle for a local LLM runtime that can have transient hiccups, e.g. model still loading).
