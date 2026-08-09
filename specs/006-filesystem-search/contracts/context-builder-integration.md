# Contract: Context Builder Relevance Integration

Internal API contract between `search.rs` and `context.rs`/`session.rs`, covering User Story 2 (FR-006, FR-007, FR-008, FR-009).

## `build_context_snapshot` signature change

```text
// v0.4 (current)
build_context_snapshot(paths: &WorkspacePaths, project: Option<&Project>) -> Result<ContextSnapshot>

// v0.5 (this feature)
build_context_snapshot(paths: &WorkspacePaths, project: Option<&Project>, query: Option<&str>) -> Result<ContextSnapshot>
```

### Behavior by `query`

- **`None`** — reproduces v0.4 exactly: all candidate Base/Project/Session Markdown files, sorted by fixed priority (`project > base > session`, then path), first-fit under `DEFAULT_MAX_CONTEXT_SOURCES` / `DEFAULT_MAX_CONTEXT_BYTES`. Used for the initial system prompt built in `run_session` before any user question exists.
- **`Some(text)`** — candidates are scored via `search.rs` against `text`; selection order becomes relevance score descending, with the v0.4 priority order and path used only to break exact ties. Same caps apply. Used on every chat turn in `run_chat_loop`, with `text` = the user's latest message.

### Invariants preserved from v0.4

- Never exceeds `DEFAULT_MAX_CONTEXT_SOURCES` (8) sources or `DEFAULT_MAX_CONTEXT_BYTES` (12000) bytes (FR-008).
- A file that was matched but becomes unreadable between search and assembly is skipped, not treated as a fatal error (`fs::read_to_string` failures already `continue` in `context.rs`; FR-009 keeps this behavior for relevance-selected sources too).
- Permission scoping is unchanged — `build_context_snapshot` still only ever sees candidates already confined to the caller's granted paths.

## `run_chat_loop` integration

```text
// v0.4 (current)
fn run_chat_loop(runtime: &RuntimeManager, model: &str, context: &ContextSnapshot)

// v0.5 (this feature)
fn run_chat_loop(paths: &WorkspacePaths, project: &Project, runtime: &RuntimeManager, model: &str, initial_context: ContextSnapshot)
```

Per turn: after the user submits `message` and before calling `runtime.send_chat_message`, re-run `build_context_snapshot(paths, Some(project), Some(&message))` and replace `history[0]` (the system message) with `context.as_system_prompt()` from the freshly assembled snapshot. Prior user/assistant turns already in `history` are left untouched — only the system message is refreshed, so the model still has the conversation so far plus newly focused context for the current question.

## `search.rs` public surface consumed by the above

```text
pub fn rank_candidates(candidates: &[SearchCandidate], query: &SearchQuery) -> Vec<SearchResult>
```

Pure function: no filesystem traversal, no permission logic — callers supply an already permission-filtered candidate list (see data-model.md `SearchCandidate`). This keeps `search.rs` the single place ranking logic lives, reusable identically by both `session::run_search` (standalone command) and `context::build_context_snapshot` (session integration), per the constitution's "major components must remain replaceable" principle.
