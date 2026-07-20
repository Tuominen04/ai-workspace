# Tasks: AI CLI Foundation v0.1

**Input**: Design documents from /specs/001-ai-cli-foundation/

**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Tests**: No mandatory automated tests were explicitly requested in the feature specification. Validation tasks below use quickstart scenarios.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Initialize repository-level CLI project files and planning artifacts required for v0.1 execution.

- [ ] T001 Create Rust package manifest for ai binary in Cargo.toml
- [ ] T002 Create module export surface in src/lib.rs
- [ ] T003 [P] Create feature planning artifacts in specs/001-ai-cli-foundation/research.md
- [ ] T004 [P] Create feature data model in specs/001-ai-cli-foundation/data-model.md
- [ ] T005 [P] Create CLI contract in specs/001-ai-cli-foundation/contracts/cli.md
- [ ] T006 [P] Create manual validation scenarios in specs/001-ai-cli-foundation/quickstart.md

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Implement shared foundations required by all user stories.

**CRITICAL**: No user story work begins until this phase is complete.

- [ ] T007 Implement shared error types and result alias in src/error.rs
- [ ] T008 Implement workspace root detection and layout creation in src/paths.rs
- [ ] T009 Implement project discovery utilities in src/workspace.rs
- [ ] T010 [P] Implement top-level command parsing and dispatch in src/app.rs
- [ ] T011 [P] Implement CLI entrypoint error handling in src/main.rs
- [ ] T012 Implement status and init session actions in src/session.rs
- [ ] T013 Implement status and init terminal rendering in src/ui.rs

**Checkpoint**: Foundation complete. User story implementation can proceed.

---

## Phase 3: User Story 1 - Start Local Workspace Session (Priority: P1) MVP

**Goal**: User can run ai, select a workspace, enter an active session, and close it cleanly.

**Independent Test**: Run ./target/debug/ai with at least one project under ~/AI/Projects, select one project, and close with Enter.

### Implementation for User Story 1

- [ ] T014 [US1] Implement interactive session orchestration in src/session.rs
- [ ] T015 [P] [US1] Implement project list display and selection prompt in src/ui.rs
- [ ] T016 [US1] Implement no-projects startup error handling in src/session.rs
- [ ] T017 [P] [US1] Implement startup banner and session summary output in src/ui.rs
- [ ] T018 [US1] Wire default ai command to interactive run path in src/app.rs
- [ ] T019 [US1] Add user-controlled shutdown wait behavior in src/ui.rs

**Checkpoint**: User Story 1 is fully functional and independently testable.

---

## Phase 4: User Story 2 - Enforce Base-Only AI Context (Priority: P2)

**Goal**: AI-readable context is restricted to ~/AI/Base only and clearly visible to the user.

**Independent Test**: Add markdown files in both ~/AI/Base and ~/AI/Projects/<name>, run ai, verify only Base files are shown as AI-readable context.

### Implementation for User Story 2

- [ ] T020 [US2] Implement Base-only context snapshot model in src/context.rs
- [ ] T021 [US2] Enforce markdown discovery under Base only in src/context.rs
- [ ] T022 [US2] Integrate context builder into session startup flow in src/session.rs
- [ ] T023 [US2] Render Base-only access policy and file list in src/ui.rs
- [ ] T024 [US2] Update feature requirements for Base-only policy in specs/001-ai-cli-foundation/spec.md

**Checkpoint**: User Story 2 is independently testable and policy-compliant.

---

## Phase 5: User Story 3 - Runtime Lifecycle and Visibility (Priority: P3)

**Goal**: Session reports runtime state, reuses running Ollama, attempts startup when available, and degrades gracefully when unavailable.

**Independent Test**: Run ai with Ollama already running, then stopped, then unavailable; verify status messaging and clean command behavior.

### Implementation for User Story 3

- [ ] T025 [US3] Implement runtime status model and manager in src/runtime.rs
- [ ] T026 [US3] Implement runtime health check and reuse logic in src/runtime.rs
- [ ] T027 [US3] Implement runtime startup attempt and readiness wait in src/runtime.rs
- [ ] T028 [US3] Implement graceful missing-runtime behavior in src/runtime.rs
- [ ] T029 [US3] Integrate runtime manager lifecycle into session orchestration in src/session.rs
- [ ] T030 [US3] Render runtime detail in session summary output in src/ui.rs

**Checkpoint**: User Story 3 is independently testable and robust across runtime states.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Final consistency and validation across all delivered stories.

- [ ] T031 [P] Update contributor guidance for docs-first and architecture workflow in CONTRIBUTING.md
- [ ] T032 [P] Update workspace instructions and file-scoped instructions in .github/copilot-instructions.md
- [ ] T033 Run full quickstart validation scenarios in specs/001-ai-cli-foundation/quickstart.md
- [ ] T034 Run cargo fmt and cargo test to verify build health in Cargo.toml
- [ ] T035 Review implementation against constitution constraints in .specify/memory/constitution.md

---

## Dependencies & Execution Order

### Phase Dependencies

- Setup (Phase 1): no dependencies
- Foundational (Phase 2): depends on Setup completion
- User Story phases (Phase 3 to Phase 5): depend on Foundational completion
- Polish (Phase 6): depends on completion of selected user stories

### User Story Dependencies

- User Story 1 (P1): starts after Foundational, no dependency on other stories
- User Story 2 (P2): starts after Foundational, may use US1 session flow but remains independently testable
- User Story 3 (P3): starts after Foundational, may use US1 session flow but remains independently testable

### Within Each User Story

- Implement core behavior first
- Integrate with session flow second
- Render visibility and user feedback last
- Validate the story independently before moving to next priority

### Parallel Opportunities

- Phase 1: T003, T004, T005, T006 can run in parallel
- Phase 2: T010 and T011 can run in parallel after T007 and T008
- User Story 1: T015 and T017 can run in parallel after T014 starts
- User Story 2: T024 can run in parallel with T020-T023
- User Story 3: T026 and T027 can run in parallel after T025 skeleton exists
- Phase 6: T031 and T032 can run in parallel

---

## Parallel Example: User Story 1

- Task: T015 [P] [US1] Implement project list display and selection prompt in src/ui.rs
- Task: T017 [P] [US1] Implement startup banner and session summary output in src/ui.rs

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational
3. Complete Phase 3: User Story 1
4. Validate independent test for User Story 1
5. Demo CLI startup and clean shutdown

### Incremental Delivery

1. Complete Setup + Foundational
2. Deliver User Story 1 (interactive session)
3. Deliver User Story 2 (Base-only AI context)
4. Deliver User Story 3 (runtime lifecycle visibility)
5. Execute polish validation and constitution review

### Parallel Team Strategy

1. Team completes Setup and Foundational tasks together
2. Split stories across developers after Phase 2
3. Merge each story after passing independent story-level validation

---

## Notes

- [P] tasks touch separate files or independent documentation and can run concurrently.
- [USx] labels ensure traceability from tasks to user stories.
- Keep changes small and architecture-aligned per constitution.
- Stop and ask for clarification if behavior, scope, permissions, or architecture become ambiguous.
