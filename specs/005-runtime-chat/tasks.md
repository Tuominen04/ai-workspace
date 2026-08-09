# Tasks: v0.4.1 Runtime Chat Loop

**Input**: Design documents from `/specs/005-runtime-chat/`

**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Tests**: Tests are included because the spec requires graceful degradation and failure handling that must not regress.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Add the new dependency and document the architecture change before touching behavior.

- [ ] T001 Add `ureq` and `serde_json` to `[dependencies]` in Cargo.toml; bump `version` to `0.4.1`
- [ ] T002 [P] Update ARCHITECTURE.md Layer 6 (Runtime Manager) and Runtime Abstraction section to document chat message transport
- [ ] T003 [P] Add ADR-0004 to DECISIONS.md for the blocking-HTTP, non-streaming `/api/chat` decision

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Establish the shared primitives (config fields, message types, transport, error variant) all stories depend on.

- [ ] T004 Add `default_model: String` to `WorkspaceConfig` in src/config.rs with a backward-compatible serde default and a value set in `default_for_paths`
- [ ] T005 Add `model: Option<String>` to `ProjectMetadata` in src/workspace.rs with a backward-compatible serde default
- [ ] T006 Add a `resolve_model(config: &WorkspaceConfig, metadata: &ProjectMetadata) -> String` helper in src/workspace.rs (project override wins, else workspace default)
- [ ] T007 Add `ContextSnapshot::as_system_prompt(&self) -> String` in src/context.rs
- [ ] T008 Add `ChatMessage { role, content }` and `RuntimeManager::send_chat_message(base_url, model, messages)` in src/runtime.rs using `ureq` + `serde_json`, per contracts/ollama-chat-api.md
- [ ] T009 Add `AppError::RuntimeRequestFailed(String)` in src/error.rs

**Checkpoint**: Foundation ready - user story implementation can now begin

---

## Phase 3: User Story 1 - Ask a question grounded in workspace context (Priority: P1) 🎯 MVP

**Goal**: A session with Ollama running lets the user ask questions and get answers grounded in the assembled context, across multiple turns.

**Independent Test**: Start a session in a workspace with base/project knowledge, ask a question that depends on that knowledge, confirm the answer reflects it; ask a follow-up and confirm history carries forward.

### Tests for User Story 1

- [ ] T010 [P] [US1] Add a `tests/runtime_tests.rs` integration test that mocks a `/api/chat` response via a local `TcpListener` and asserts `send_chat_message` parses it into the expected `ChatMessage`

### Implementation for User Story 1

- [ ] T011 [US1] Add `ui::prompt_for_chat_message() -> Result<Option<String>>` in src/ui.rs (returns `None` on `exit`/`quit`/EOF)
- [ ] T012 [US1] Add `ui::render_ai_response(content: &str)` in src/ui.rs
- [ ] T013 [US1] Add a private `run_chat_loop` in src/session.rs that seeds history with the system prompt, loops prompt → send → render, and exits on `None` from the prompt
- [ ] T014 [US1] Wire `run_chat_loop` into `run_session` in src/session.rs when `runtime_status.running` is true, using the resolved model from `resolve_model`
- [ ] T015 [US1] Extend `ui::render_session_summary` trailing text in src/ui.rs to describe the chat prompt instead of only "press Enter to close" when chat will follow

**Checkpoint**: User Story 1 fully functional and testable independently

---

## Phase 4: User Story 2 - Continue working when the AI is unavailable (Priority: P2)

**Goal**: Sessions without Ollama, or with a mid-session failure, degrade gracefully instead of crashing or losing the session.

**Independent Test**: Start a session with Ollama stopped/not installed and confirm today's exact fallback behavior; start a session, stop Ollama mid-session, ask a question, confirm the session reports the error and keeps going.

### Tests for User Story 2

- [ ] T016 [P] [US2] Add `tests/runtime_tests.rs` cases for a non-200 / malformed mock response mapping to `AppError::RuntimeRequestFailed`

### Implementation for User Story 2

- [ ] T017 [US2] In `run_session` (src/session.rs), keep the existing `ui::wait_for_exit()` path unchanged when `runtime_status.running` is false (no chat prompt offered)
- [ ] T018 [US2] Add `ui::render_chat_error(message: &str)` in src/ui.rs
- [ ] T019 [US2] In `run_chat_loop` (src/session.rs), catch `send_chat_message` errors, render via `render_chat_error`, and continue the loop without appending a broken turn to history

**Checkpoint**: User Stories 1 and 2 both work independently

---

## Phase 5: User Story 3 - Know which model is answering (Priority: P3)

**Goal**: Workspace-wide default model with an optional per-project override, resolved automatically.

**Independent Test**: Set a workspace default, start a session with no project override and confirm that model is used; set a project override and confirm it takes priority for that project.

### Tests for User Story 3

- [ ] T020 [P] [US3] Add a `workspace.rs` (or `config_tests.rs`) unit test for `resolve_model` covering both the override and fallback cases
- [ ] T021 [P] [US3] Add a `config_tests.rs`/`workspace_tests.rs` test confirming existing `workspace.toml`/`project.toml` files without the new fields still deserialize successfully

### Implementation for User Story 3

- [ ] T022 [US3] Confirm `run_session` (src/session.rs) loads project metadata (reusing the existing `inspect_project` pattern) and passes the resolved model into `run_chat_loop`

**Checkpoint**: All user stories independently functional

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Verify the full feature end to end and leave the docs/tests in a clean state.

- [ ] T023 [P] Run `cargo fmt` and `cargo clippy`, address findings
- [ ] T024 Run the full test suite with `cargo test` and confirm `context_tests.rs`, `config_tests.rs`, `workspace_tests.rs` still pass unmodified
- [ ] T025 Validate the quickstart scenarios from specs/005-runtime-chat/quickstart.md against a real local Ollama installation

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - blocks all user stories
- **User Stories (Phase 3+)**: All depend on Foundational completion
- **Polish (Phase 6)**: Depends on all user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational - no dependencies on other stories
- **User Story 2 (P2)**: Can start after Foundational - integrates with US1's chat loop but is independently testable via the no-Ollama and mid-session-failure paths
- **User Story 3 (P3)**: Can start after Foundational - integrates with US1's `run_chat_loop` call but is independently testable via `resolve_model` directly

### Parallel Opportunities

- Tasks T002 and T003 can be done in parallel during setup
- Tasks T004, T005, T007, T009 touch different files and can be done in parallel before T006/T008 (which depend on T004/T005 and T009 respectively)
- Task T010 can be written in parallel with T011-T012 (test vs. UI code)
- Task T016 can be written in parallel with T017-T018
- Tasks T020 and T021 can be done in parallel for US3
