# Tasks: v0.4 Context Builder

**Input**: Design documents from `/specs/004-context-builder/`

**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Tests**: Tests are OPTIONAL for this feature. This task list includes focused unit tests because the spec calls for deterministic and bounded context behavior.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Prepare the existing Rust CLI project for the context-builder feature.

- [ ] T001 Create or update the context-builder implementation plan artifacts in specs/004-context-builder/
- [ ] T002 [P] Add unit-test scaffolding for context assembly behavior in tests/
- [ ] T003 [P] Review current context/session/UI module boundaries and confirm the feature stays within the existing architecture

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Establish the shared context-building primitives that all stories depend on.

- [ ] T004 Implement a context-source selection model in src/context.rs for base, project, and session sources
- [ ] T005 Implement deterministic source ordering and filtering in src/context.rs
- [ ] T006 Implement bounded context assembly in src/context.rs with explicit size limits and graceful handling of missing sources
- [ ] T007 Extend the session startup flow in src/session.rs to build and pass a context package for the selected project
- [ ] T008 Update the session summary rendering in src/ui.rs to show base and project context counts clearly

**Checkpoint**: Foundation ready - user story implementation can now begin in parallel

---

## Phase 3: User Story 1 - Start with relevant context (Priority: P1) 🎯 MVP

**Goal**: Deliver a context package that includes shared base knowledge and the active project's knowledge for a new session.

**Independent Test**: A user can start a session and see that base and project context are both present in the startup summary.

### Tests for User Story 1

- [ ] T009 [P] [US1] Add a unit test for base and project source inclusion in tests/
- [ ] T010 [P] [US1] Add a unit test for project switching so a new session uses the newly selected project context in tests/

### Implementation for User Story 1

- [ ] T011 [P] [US1] Add project-aware context discovery in src/context.rs for the selected project path
- [ ] T012 [US1] Assemble a combined context package in src/context.rs that includes both base and project sources
- [ ] T013 [US1] Surface the assembled context summary in src/ui.rs so the current session clearly shows base and project context usage
- [ ] T014 [US1] Ensure missing project context degrades gracefully and still includes base knowledge in src/context.rs

**Checkpoint**: At this point, User Story 1 should be fully functional and testable independently

---

## Phase 4: User Story 2 - Keep prompts focused (Priority: P2)

**Goal**: Deliver a bounded and relevant context package that stays useful even when many files are available.

**Independent Test**: A user can start a session in a workspace with many documents and the context summary remains bounded and focused.

### Tests for User Story 2

- [ ] T015 [P] [US2] Add a unit test for size-bounded context assembly in tests/
- [ ] T016 [P] [US2] Add a unit test for deterministic ordering of included sources in tests/

### Implementation for User Story 2

- [ ] T017 [P] [US2] Implement a size cap and source-count cap for context assembly in src/context.rs
- [ ] T018 [US2] Ensure context sources are ordered deterministically by path and selected consistently across runs in src/context.rs
- [ ] T019 [US2] Update the startup summary messaging in src/ui.rs to explain that the prompt is bounded and relevant

**Checkpoint**: At this point, User Stories 1 and 2 should both work independently

---

## Phase 5: User Story 3 - Build a coherent prompt from multiple sources (Priority: P3)

**Goal**: Deliver a coherent context package that can combine base, project, and session sources for the active task.

**Independent Test**: A user can start a session with base, project, and session resources and see a coherent combined context package.

### Tests for User Story 3

- [ ] T020 [P] [US3] Add a unit test for combining base, project, and session sources in tests/
- [ ] T021 [P] [US3] Add a unit test for missing or unreadable sources being skipped without breaking assembly in tests/

### Implementation for User Story 3

- [ ] T022 [P] [US3] Extend context assembly to include available session resources in src/context.rs
- [ ] T023 [US3] Preserve the distinction between shared base knowledge and project-specific knowledge in the assembled context package
- [ ] T024 [US3] Ensure the summary output in src/ui.rs clearly distinguishes base, project, and session context contributions

**Checkpoint**: All user stories should now be independently functional

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Improve the implementation across all stories and verify the behavior end to end.

- [ ] T025 [P] Update any relevant documentation in docs/ or README.md to reflect the new context builder behavior
- [ ] T026 Run the full test suite with cargo test and address regressions
- [ ] T027 Validate the quickstart scenarios from specs/004-context-builder/quickstart.md against the CLI flow

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - blocks all user stories
- **User Stories (Phase 3+)**: All depend on Foundational completion
- **Polish (Phase 6)**: Depends on all user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational - no dependencies on other stories
- **User Story 2 (P2)**: Can start after Foundational - may integrate with US1 but should be independently testable
- **User Story 3 (P3)**: Can start after Foundational - may integrate with US1/US2 but should be independently testable

### Parallel Opportunities

- Tasks T002 and T003 can be done in parallel during setup
- Tasks T009 and T010 can be done in parallel for US1
- Tasks T015 and T016 can be done in parallel for US2
- Tasks T020 and T021 can be done in parallel for US3
- Tasks T025 and T026 are independent once implementation is complete
