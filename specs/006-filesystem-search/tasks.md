# Tasks: v0.5 Filesystem Search

**Input**: Design documents from `/specs/006-filesystem-search/`

**Prerequisites**: [plan.md](./plan.md), [spec.md](./spec.md), [research.md](./research.md), [data-model.md](./data-model.md), [contracts/](./contracts/), [quickstart.md](./quickstart.md)

**Tests**: Included. Per `CONTRIBUTING.md`'s Testing Guidelines (referenced as a strict rule in `CLAUDE.md`), this feature changes behavior/workflows and must cover at least one happy path and one critical failure/safety path — implemented as `assert_cmd`-based integration tests in `tests/*.rs`, matching the existing project convention (no inline `#[cfg(test)]` modules).

**Organization**: Tasks are grouped by user story (P1/P2/P3 from spec.md) so each can be implemented and tested independently.

> Revised after `/speckit-analyze`: renumbered from the original draft to insert a permission-boundary test (T009, finding G1) and to fix the session-start context-selection design (T016/T017, finding I1). See `research.md` Decision 4 and `contracts/context-builder-integration.md` for the resolved behavior.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no unmet dependencies)
- **[Story]**: Maps the task to US1, US2, or US3
- File paths are exact and relative to the repository root

## Path Conventions

Single Rust project, flat `src/` module layout (per `CLAUDE.md`; the nested `core/`/`search/` layout in `PROJECR_STRUCTURE.md` is not introduced). Integration tests live in `tests/*.rs`.

---

## Phase 1: Setup

**Purpose**: Wire the new module into the build so later tasks compile incrementally.

- [X] T001 Add `pub mod search;` to `src/lib.rs` and create `src/search.rs` with a module-level doc comment describing it as the Layer 5 Search Engine (per `ARCHITECTURE.md`)

**Checkpoint**: `cargo build` succeeds with an empty `search` module.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: The shared ranking engine and permission-filtered candidate collection that both the standalone search command (US1/US3) and the Context Builder integration (US2) depend on. Per `data-model.md` and `contracts/context-builder-integration.md`, this logic is written once and reused by both.

**⚠️ CRITICAL**: No user story task can begin until this phase is complete.

- [X] T002 Define `SourceKind`, `SearchQuery`, `SearchCandidate`, `SearchResult`, `SearchOutcome` types in `src/search.rs` per `data-model.md` (depends on T001)
- [X] T003 Implement `score_candidate` — case-insensitive term-frequency keyword scoring with a filename-match bonus — in `src/search.rs` per `research.md` Decision 1 (depends on T002)
- [X] T004 Implement `extract_snippet` — bounded (~160 char), word-boundary-trimmed excerpt around the best match — in `src/search.rs` per `research.md` Decision 6 (depends on T002)
- [X] T005 Implement `pub fn rank_candidates(candidates: &[SearchCandidate], query: &SearchQuery) -> Vec<SearchResult>` in `src/search.rs`, combining T003/T004, sorted by `relevance_score` descending with `source_kind` priority (project > base > session) then path as tie-break, excluding zero-score files (depends on T003, T004)
- [X] T006 Implement `collect_search_candidates(paths: &WorkspacePaths, project: Option<&Project>) -> Result<Vec<SearchCandidate>>` in `src/session.rs`, explicitly traversing all three permission-scoped roots via `workspace::collect_markdown_files` — the Base path, the active project's `Knowledge/` directory, and the active project's `Session/` directory — filtered to the current `SessionPermissionSet`'s `default_grants` and active `temporary_grants`, reusing `initialize_permission_session`/`view_permissions` as-is (no changes to `workspace.rs`) (depends on T002)

**Checkpoint**: `search.rs`'s ranking engine and `session.rs`'s candidate collection compile and are unit-testable in isolation. User story implementation can now begin.

---

## Phase 3: User Story 1 - Find relevant documents in a growing workspace (Priority: P1) 🎯 MVP

**Goal**: `ai search --query TEXT [--project PROJECT] [--limit N]` returns topically relevant, ranked results with snippets, clearly reports when nothing matches, and never leaks content outside the caller's granted paths.

**Independent Test**: In a workspace with dozens of Markdown documents, search for a topic covered by only a few of them and confirm the returned documents are the on-topic ones, not an arbitrary subset — runnable via `cargo run -- search --query ... --project ...` per `quickstart.md` Scenario 1.

### Tests for User Story 1

- [X] T007 [P] [US1] Integration test in `tests/search_tests.rs`: a project with several Markdown files where only some mention a given topic; assert `ai search --query <topic> --project <name>` returns exactly the on-topic files, ranked with the strongest match first
- [X] T008 [P] [US1] Integration test in `tests/search_tests.rs`: `ai search --query <text that matches nothing> --project <name>` exits `0` and prints a distinct "no results" message (not an error), per `contracts/cli-search.md`
- [X] T009 [P] [US1] Integration test in `tests/search_tests.rs`: a Markdown file placed outside any Default/Temporary permission grant (e.g. under a directory never passed to `ai permission grant`), whose content matches the query, is never present in `ai search` output even though it would otherwise score highly — per FR-002/SC-004 and `quickstart.md` Scenario 3

### Implementation for User Story 1

- [X] T010 [US1] Add an `AppError` variant (and `Display` message) for a missing/empty `--query` usage error in `src/error.rs`
- [X] T011 [US1] Add `Command::Search { query, project, limit }` parsing in `src/app.rs` (`--query` required, `--project` optional, `--limit` optional positive integer defaulting to `10`, same flag-parsing style as `parse_permission_grant_command`), dispatch to a new `session::run_search`, and add the `ai search ...` line to `print_help` (depends on T010)
- [X] T012 [US1] Implement `render_search_results(&SearchOutcome)` and `render_no_search_results(&SearchOutcome)` in `src/ui.rs`, rendering rank/source tag/path/score/snippet per `contracts/cli-search.md`'s output format (pure presentation, no filesystem access)
- [X] T013 [US1] Implement `pub fn run_search(paths: &WorkspacePaths, query: &str, project: Option<&str>, limit: usize) -> Result<()>` in `src/session.rs`: call `enforce_validation_gate`, resolve `project` by name via existing project lookup (error if unknown), build candidates via `collect_search_candidates` (T006) — which already excludes anything outside the current permission grants — rank via `search::rank_candidates` (T005), truncate to `limit`, and hand the resulting `SearchOutcome` to `ui::render_search_results`/`render_no_search_results` (T012) (depends on T005, T006, T011, T012)

**Checkpoint**: `ai search --query ... [--project ...]` works end-to-end, respects permission boundaries, and is independently testable/demoable.

---

## Phase 4: User Story 2 - Focus session context on the current task (Priority: P2)

**Goal**: The Context Builder selects Base/Project/Session documents by relevance — both at session start (seeded from the project's own description) and on every subsequent question (seeded from the message) — instead of the v0.4 fixed priority-order-then-cap behavior.

**Independent Test**: In a workspace whose documentation exceeds the Context Builder's caps, start a session and confirm the *initial* context already reflects the project's description before any question is asked, then ask a targeted question and confirm the context shifts to reflect it — per `quickstart.md` Scenarios 4a, 4b, and 5.

### Tests for User Story 2

- [X] T014 [P] [US2] Integration/unit test in `tests/context_tests.rs`: with more Markdown files than `DEFAULT_MAX_CONTEXT_SOURCES`, `build_context_snapshot(paths, project, Some(query))` includes the files relevant to `query` even when they weren't first in `project > base > session` order — covers both an explicit question and a project-description-style query, since both now flow through the same `Some(text)` path
- [X] T015 [P] [US2] Regression test in `tests/context_tests.rs`: `build_context_snapshot(paths, None, None)` (no project selected) produces the same selection/ordering as v0.4 (fixed priority order, first-fit under caps) — the one remaining case where `query = None` is expected

### Implementation for User Story 2

- [X] T016 [US2] Change `build_context_snapshot`'s signature to `build_context_snapshot(paths: &WorkspacePaths, project: Option<&Project>, query: Option<&str>) -> Result<ContextSnapshot>` in `src/context.rs`: when `query` is `Some`, order all candidate sources by relevance descending (existing priority order as tie-break only); when `query` is `None`, keep the existing v0.4 selection logic unchanged — this fallback path is reserved for the no-project-selected case, not for session start with an active project (see T017). Caps (`DEFAULT_MAX_CONTEXT_SOURCES`/`DEFAULT_MAX_CONTEXT_BYTES`) and the graceful-skip-on-unreadable-file behavior are preserved in both branches (depends on T005). **Implementation note**: `search::rank_candidates` (T005) drops zero-score candidates, which is correct for the standalone command but would silently break User Story 2 AC2 ("small workspace still gets everything") — an unrelated-but-fitting file would get excluded even with room to spare. Added a second `search.rs` entry point, `order_by_relevance`, that sorts by the same score but never drops zero-score candidates; `context.rs` uses that one, `session::run_search` keeps using `rank_candidates`.
- [X] T017 [US2] Update `src/session.rs`: in `run_session`, before building the initial context, compute `initial_query` from the selected project's `metadata.description` (falling back to `metadata.display_name` when the description is empty) and call `build_context_snapshot(paths, Some(project), Some(&initial_query))` — **not** `None` — so the session-start prompt is already relevance-selected (this is the fix for finding I1: FR-006/AC1 require relevance at session start, not just per question). Change `run_chat_loop`'s signature to take `paths: &WorkspacePaths` and `project: &Project`, and on each user turn call `build_context_snapshot(paths, Some(project), Some(&message))` before sending to the runtime, replacing `history[0]` with the freshly assembled system prompt while leaving prior turns in `history` untouched (depends on T016)
- [X] T018 [US2] Update the existing v0.4 callers/tests of `build_context_snapshot` (`tests/context_tests.rs`) to pass the new `query` argument, verifying no behavior change for the `project = None, query = None` case (depends on T016)

**Checkpoint**: US1 and US2 both work independently; a live chat session's context is relevance-driven from the moment it starts, not just after the first question.

---

## Phase 5: User Story 3 - See what a search found before it's used (Priority: P3)

**Goal**: Search results are self-explanatory and usable purely as a lookup tool, with no session required.

**Independent Test**: Run `ai search` without ever starting an interactive session and confirm each result shows its file location and a snippet explaining the match — per `quickstart.md` Scenario 1/3 and `spec.md` Story 3.

### Tests for User Story 3

- [X] T019 [P] [US3] Integration test in `tests/search_tests.rs`: invoking `ai search --query ... --project ...` as the very first command against a freshly initialized workspace (no prior `ai` session run) succeeds and each printed result includes a file path and a non-empty snippet

### Implementation for User Story 3

- [X] T020 [US3] Extend `render_no_search_results` in `src/ui.rs` to include the permission hint ("check `ai permission view` if you expected a path to be included") per `contracts/cli-search.md`'s no-matches output (depends on T012)

**Checkpoint**: All three user stories are independently functional and testable.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Documentation obligations flagged in `plan.md`'s Constitution Check, plus final validation.

- [X] T021 [P] Update `CLAUDE.md`'s module-responsibilities list to document `search.rs` (Layer 5 Search Engine: scoring, ranking, snippet extraction) and note `context.rs`'s `build_context_snapshot` now takes an optional query
- [X] T022 [P] Add a `DECISIONS.md` ADR entry for "search-driven context selection" (relevance ranking, seeded from the project description at session start and from the user's message thereafter, replaces fixed priority-order-then-cap) and confirm `ARCHITECTURE.md`'s Layer 5 description matches the shipped design
- [X] T023 [P] Run `cargo fmt` and `cargo clippy` across all changed files and address any warnings — run on 2026-11-14: `cargo fmt` applied; `cargo clippy --all-targets` clean after fixing two warnings (`needless_borrow` in `src/session.rs`, `new_without_default` for `RuntimeManager`).
- [X] T024 Manually run all `quickstart.md` scenarios (including 4a's session-start check and 6's hundreds-of-document timing pass for SC-003) against a scratch `AI_WORKSPACE_HOME` and confirm actual output matches each scenario's expected result — all 6 scenarios verified: Scenario 1 (ranked results), 2 (no-results message), 3 (permission boundary), 6 (200 files, ~5ms — well under the 2s budget), 4a (session-start context already includes a Base file the fixed v0.4 order would have excluded, driven by the project description), 5 (small workspace still gets every file regardless of relevance)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — start immediately.
- **Foundational (Phase 2)**: Depends on Setup (T001). Blocks all user stories.
- **User Story 1 (Phase 3)**: Depends on Foundational (T002-T006). No dependency on US2/US3.
- **User Story 2 (Phase 4)**: Depends on Foundational (T002-T006), specifically `rank_candidates` (T005). Does not depend on US1's CLI/UI work (T010-T013) — it integrates `search.rs` directly into `context.rs`/`session.rs`.
- **User Story 3 (Phase 5)**: Depends on US1's `render_no_search_results` (T012) existing to extend. Practically sequenced after US1, though its own test (T019) only exercises behavior US1 already implements.
- **Polish (Phase 6)**: Depends on all desired user stories being complete.

### Within Each User Story

- Tests are written before their corresponding implementation task and should fail first (T007/T008/T009 before T010-T013; T014/T015 before T016-T018; T019 before T020).
- Foundational engine (`search.rs`) before any story wiring.
- `error.rs` → `app.rs` → `ui.rs` → `session.rs` ordering within US1 (T010 → T011 → T012 → T013) reflects real compile dependencies (session::run_search calls into ui and app dispatches into session).

### Parallel Opportunities

- Foundational: T003 and T004 touch the same file (`src/search.rs`) and are left unmarked for parallel execution to avoid merge conflicts, even though they're logically independent.
- US1 tests T007, T008, and T009 (`[P]`) can be drafted in parallel — same file, but independent scenarios, safe to write before implementation exists.
- US2 tests T014 and T015 (`[P]`) likewise.
- Polish tasks T021, T022, T023 (`[P]`) touch different files (`CLAUDE.md`, `DECISIONS.md`/`ARCHITECTURE.md`, and cargo tooling) and have no dependencies on each other.
- US1 (Phase 3) and US2 (Phase 4) implementation can proceed in parallel by different contributors once Foundational is done, since US2 does not depend on US1's CLI surface.

---

## Parallel Example: Foundational → User Story 1 and User Story 2 together

```bash
# After Phase 2 (T001-T006) completes:
Task: "Integration test in tests/search_tests.rs: ranked results for a multi-doc project (T007)"
Task: "Integration/unit test in tests/context_tests.rs: relevance overrides file order (T014)"
# US1 and US2 implementation tasks can then proceed on separate branches/by separate contributors,
# since US2 (context.rs/session.rs) does not touch US1's app.rs/ui.rs CLI surface.
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001)
2. Complete Phase 2: Foundational (T002-T006) — the ranking engine is the hard part; get it right once
3. Complete Phase 3: User Story 1 (T007-T013)
4. **STOP and VALIDATE**: run `quickstart.md` Scenarios 1-3 by hand, then `cargo test --test search_tests`
5. `ai search` is now a usable, demoable feature on its own — the roadmap's "Filesystem search" half of v0.5

### Incremental Delivery

1. Setup + Foundational → ranking engine ready
2. Add User Story 1 → validate → demoable standalone search (MVP)
3. Add User Story 2 → validate → chat sessions now use relevance-driven context from the first prompt onward (the roadmap's "Relevant document loading" half)
4. Add User Story 3 → validate → search result transparency confirmed/hardened
5. Polish → docs and quickstart sign-off

### Notes

- No new dependencies are introduced (see `research.md` Decision 1) — no `Cargo.toml` changes are needed at any phase.
- T016/T017 change two public-ish function signatures (`build_context_snapshot`, `run_chat_loop`); do these together (US2) rather than splitting across contributors to avoid a half-migrated state.
- SC-003 (search stays under 2 seconds over hundreds of documents) is validated manually via `quickstart.md` Scenario 6 (T024) rather than an automated timing test, to avoid a flaky CI-dependent benchmark.
- Commit after each task or logical group; stop at either checkpoint to validate a story independently before continuing.
