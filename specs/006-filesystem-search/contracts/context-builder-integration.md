# Contract: Context Builder Relevance Integration

Internal API contract between `search.rs` and `context.rs`/`session.rs`, covering User Story 2 (FR-006, FR-007, FR-008).

## `build_context_snapshot` signature change

```text
// v0.4 (current)
build_context_snapshot(paths: &WorkspacePaths, project: Option<&Project>) -> Result<ContextSnapshot>

// v0.5 (this feature)
build_context_snapshot(paths: &WorkspacePaths, project: Option<&Project>, query: Option<&str>) -> Result<ContextSnapshot>
```

### Behavior by `query`

- **`Some(text)`** — candidates are scored via `search.rs` against `text`; selection order becomes relevance score descending, with the v0.4 priority order and path used only to break exact ties. Same caps apply. This is the path used in both places `build_context_snapshot` is called:
  - **Session start**: `run_session` derives `text` from the selected project's `metadata.description` (falling back to `metadata.display_name` when empty) — so the very first system prompt is already relevance-selected, satisfying FR-006's "session starts" clause.
  - **Every chat turn**: `run_chat_loop` uses the user's latest message as `text`.
- **`None`** — reproduces the raw v0.4 fallback: all candidate Base/Project/Session Markdown files, sorted by fixed priority (`project > base > session`, then path), first-fit under `DEFAULT_MAX_CONTEXT_SOURCES` / `DEFAULT_MAX_CONTEXT_BYTES`. Reserved for the no-project-selected case (Base-only context), where there is no project-specific text to search against. Not used by `run_session`/`run_chat_loop` when a project is active.

> Resolves an earlier spec/plan conflict (flagged by `/speckit-analyze` as finding I1): FR-006 and User Story 2's AC1 require relevance-driven selection at session start, not just per chat turn. An earlier draft of this contract passed `None` at session start, which contradicted that. The description-derived query above is the fix.

### Invariants preserved from v0.4

- Never exceeds `DEFAULT_MAX_CONTEXT_SOURCES` (8) sources or `DEFAULT_MAX_CONTEXT_BYTES` (12000) bytes (FR-007).
- A file that was matched but becomes unreadable between search and assembly is skipped, not treated as a fatal error (`fs::read_to_string` failures already `continue` in `context.rs`; FR-008 keeps this behavior for relevance-selected sources too).
- Permission scoping is unchanged — `build_context_snapshot` still only ever sees candidates already confined to the caller's granted paths.

## `run_chat_loop` integration

```text
// v0.4 (current)
fn run_chat_loop(runtime: &RuntimeManager, model: &str, context: &ContextSnapshot)

// v0.5 (this feature)
fn run_chat_loop(paths: &WorkspacePaths, project: &Project, runtime: &RuntimeManager, model: &str, initial_context: ContextSnapshot)
```

Per turn: after the user submits `message` and before calling `runtime.send_chat_message`, re-run `build_context_snapshot(paths, Some(project), Some(&message))` and replace `history[0]` (the system message) with `context.as_system_prompt()` from the freshly assembled snapshot. Prior user/assistant turns already in `history` are left untouched — only the system message is refreshed, so the model still has the conversation so far plus newly focused context for the current question.

`initial_context` (the value `run_chat_loop` is handed before the loop starts) is itself now built with a query: `run_session` computes `let initial_query = project.metadata.description.clone().filter(|d| !d.is_empty()).unwrap_or_else(|| project.metadata.display_name.clone());` and calls `build_context_snapshot(paths, Some(project), Some(&initial_query))` before invoking `run_chat_loop`.

## `search.rs` public surface consumed by the above

```text
pub fn rank_candidates(candidates: &[SearchCandidate], query: &SearchQuery) -> Vec<SearchResult>
```

Pure function: no filesystem traversal, no permission logic — callers supply an already permission-filtered candidate list (see data-model.md `SearchCandidate`). This keeps `search.rs` the single place ranking logic lives, reusable identically by both `session::run_search` (standalone command) and `context::build_context_snapshot` (session integration), per the constitution's "major components must remain replaceable" principle.
