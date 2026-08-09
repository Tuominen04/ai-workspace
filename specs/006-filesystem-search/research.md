# Phase 0 Research: v0.5 Filesystem Search

No `[NEEDS CLARIFICATION]` markers remain in the Technical Context — this feature has no unfamiliar language/runtime/storage choices, so research here focuses on the design decisions needed to turn the spec's functional requirements into a concrete, minimal implementation.

## Decision 1: Ranking algorithm

**Decision**: Simple, case-insensitive keyword scoring — split the query into terms, count term occurrences (whole-word, case-insensitive) in each candidate file's content, with a small bonus for matches in the filename/title line. No stemming, fuzzy matching, or external ranking library.

**Rationale**: `ARCHITECTURE.md`'s Search Abstraction explicitly phases this: "Phase 1: Filesystem lookup" → "Phase 2: Document search" → "Phase 3: RAG" → "Phase 4: Hybrid search." v0.5 is Phase 1/2 territory ("simple filesystem search" per the Layer 5 description), and the constitution's "Minimal, Explicit Technology Choices" principle directs toward the simplest implementation that satisfies the goal. Term-frequency keyword scoring is enough to satisfy FR-003 (rank by relevance) and SC-001/SC-003 without adding a dependency or persistent index.

**Alternatives considered**:
- *TF-IDF*: rejected — needs corpus-wide statistics (document frequency) recomputed per search over a changing filesystem; adds complexity disproportionate to workspace-scale document counts.
- *Fuzzy/edit-distance matching crate*: rejected — new dependency, and exact/substring keyword matching is sufficient for the "simple filesystem search" scope; fuzzy matching can be layered in later without changing the public shape of `SearchResult`.
- *Embedded full-text engine (e.g., an inverted-index crate)*: rejected — implies a persistent index, which is explicitly a later phase (RAG/vector database) and conflicts with "no hidden state outside the filesystem."

## Decision 2: Module boundary

**Decision**: New `src/search.rs` module owns scoring, ranking, and snippet extraction. It does not perform its own directory traversal or permission checks — it takes a list of candidate `(source_kind, PathBuf)` pairs (already permission-filtered) from `workspace.rs`, reads their contents, and returns ranked `SearchResult`s.

**Rationale**: Matches `ARCHITECTURE.md` Layer 5 ("The rest of the application should not know which implementation is used") and keeps `workspace.rs` as the single owner of permission/traversal logic per `CLAUDE.md`'s module responsibilities. This keeps `search.rs` swappable later (RAG/embeddings) without touching `workspace.rs` or `context.rs` call sites beyond the `search.rs` API surface itself.

**Alternatives considered**:
- *Put scoring directly in `context.rs`*: rejected — conflates two responsibilities (assembly/caps vs. ranking) and makes standalone `ai search` (User Story 3) harder to serve without going through context assembly.
- *Put scoring in `workspace.rs`*: rejected — `workspace.rs` is already the largest module (865 lines); ranking/snippets are a distinct concern from project/permission/discovery logic.

## Decision 3: Permission scoping

**Decision**: Reuse the existing permission session machinery unchanged. `search.rs` never touches permissions directly; callers (`session.rs`) resolve the searchable candidate set by combining `workspace::collect_markdown_files` over three explicit roots — the Base path, the active project's `Knowledge/` directory, and the active project's `Session/` directory — with the current `SessionPermissionSet`'s `default_grants` and active (`PermissionStatus::Active`) `temporary_grants`, exactly as `context.rs` already does for its `base_files`/`project_files`/`session_files` split. Search scope explicitly includes Session content, not just Base + Knowledge (see spec.md FR-001). No new permission category or bypass path is introduced.

**Rationale**: Directly satisfies FR-002/FR-007/SC-004 and the constitution's "never assume broader scope than the user has granted." Reusing `initialize_permission_session` (already called in `run_session`) means search inherits the same idempotent-by-canonical-path, audit-logged behavior with zero new permission code. Explicitly enumerating all three roots (rather than leaving Session's inclusion implicit) closes a scope ambiguity flagged during `/speckit-analyze`: FR-001 and the `SourceKind::Session` variant in `data-model.md` must agree on what's searchable, and a dedicated permission-boundary test (see tasks.md) verifies files outside any of the three roots never appear in results.

**Alternatives considered**: A dedicated "searchable roots" permission concept — rejected as unnecessary scope creep; the existing Default/Temporary categories already answer "what can this session see."

## Decision 4: Context Builder integration

**Decision**: `build_context_snapshot` gains an `query: Option<&str>` parameter.
- `query = Some(text)` scores every candidate source via `search.rs`, sorts by relevance score descending, and breaks ties using the existing priority order — still bounded by `DEFAULT_MAX_CONTEXT_SOURCES`/`DEFAULT_MAX_CONTEXT_BYTES`.
- `query = None` preserves the raw v0.4 fallback (fixed priority order project > base > session, first-fit under the caps) — retained only for the case where no project is selected at all (Base-only context), since there is no meaningful project-specific text to search against there.

Both the session-start context and every subsequent chat turn now pass `Some(text)`, not `None` — this is a direct fix for a spec/plan conflict caught during `/speckit-analyze` (finding I1): FR-006 and User Story 2's Acceptance Scenario 1 require relevance-driven selection "whenever a session starts **or** a question is asked," but an earlier version of this decision only ran search per chat turn, leaving the session-start context on the old fixed-order behavior. Concretely:
- **At session start**, `session::run_session` builds the initial query from the selected project's `metadata.description` (falling back to `metadata.display_name` when the description is empty) and calls `build_context_snapshot(paths, Some(project), Some(&initial_query))` for the first system prompt.
- **On each chat turn**, `session::run_chat_loop` calls `build_context_snapshot(paths, Some(project), Some(&message))` using the user's latest message, and the resulting system prompt replaces (not appends to) the previous one before the message is sent to the runtime. Prior user/assistant turns in `history` are preserved as-is.

**Rationale**: This is the concrete mechanism behind FR-006/FR-007 and User Story 2's acceptance scenarios ("the assembled context is selected using search relevance," "a workspace small enough that everything fits... still includes all... material"). Deriving the session-start query from the project's own description is a minimal, no-new-input way to give search *something* to rank against before the user has typed anything, satisfying AC1's "session starts" clause without inventing new user-facing prompts. Keeping `query = None` as a narrow fallback (no project selected) means the function never needs to guess a query out of nothing, and satisfies the edge case "the active project changes between when a search runs and when its results would be used" by always re-deriving context fresh per turn rather than caching stale results.

**Alternatives considered**:
- *Keep `query = None` at session start (original decision)*: rejected on review — it silently contradicted FR-006/AC1's explicit "session starts" clause rather than resolving the ambiguity, which the constitution requires be asked/resolved rather than guessed.
- *Prompt the user for an explicit topic before starting the session*: rejected — adds a new interaction step the spec never asked for; the project description is already-available, zero-new-input signal.
- *Cache/persist the last N queries' rankings*: rejected — adds hidden state outside the filesystem for no measured benefit at this scale (SC-003's 2-second budget is comfortably met by scoring tens-to-hundreds of files per turn).

## Decision 5: Standalone command surface

**Decision**: `ai search --query TEXT [--project PROJECT] [--limit N]` (default `--limit 10`). Without `--project`, search is scoped to Base only. With `--project`, it is scoped to Base + that project's Knowledge/Session content, subject to the same permission filtering described in Decision 3. Follows the same hand-rolled flag-parsing style as `ai project create`/`ai permission grant`, and — like `ai project show` — runs `enforce_validation_gate` first, since resolving a project and reading its permission-scoped content depends on a structurally valid workspace/project.

**Rationale**: Delivers User Story 3 and FR-009 (search without starting a session) using the CLI's existing conventions, so it composes naturally with `ai --help` and the existing command dispatch table in `app.rs`.

**Alternatives considered**: Making search only available inside an interactive session — rejected, it directly contradicts FR-009 and Story 3's independent test ("usable purely as a lookup tool without requiring the user to start a session").

## Decision 6: Snippet extraction

**Decision**: For the highest-scoring match location in a file, extract a bounded plain-text window (~160 characters) around it, trimmed to word boundaries, with a leading/trailing ellipsis when truncated. No markdown rendering/highlighting.

**Rationale**: Satisfies FR-005 ("enough information... to judge why it was considered relevant") with a cheap, dependency-free string operation, consistent with Decision 1's minimal scoring approach.

**Alternatives considered**: Returning the full file content in results — rejected, defeats the purpose of a snippet and would blow past reasonable CLI output size for `ai search`.
