# Tasks: v0.3 Permission Model

**Input**: Design documents from /specs/003-permission-model/

**Prerequisites**: plan.md (required), spec.md (required), research.md, data-model.md, contracts/

**Tests**: No explicit TDD or test-first requirement is defined in the feature spec. Validation tasks focus on quickstart and full-suite execution after implementation.

**Organization**: Tasks are grouped by user story to enable independent implementation and validation.

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Prepare command and module scaffolding for permission features.

- [ ] T001 Add permission command scaffolding for grant, view, and revoke in src/app.rs
- [ ] T002 Add permission-output rendering scaffolding in src/ui.rs
- [ ] T003 Wire permission command dispatch entry points in src/session.rs
- [ ] T004 Align initial permission command surface documentation in specs/003-permission-model/contracts/cli.md

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Build shared primitives required before user story implementation.

**CRITICAL**: Complete this phase before starting any user story.

- [ ] T005 Define permission domain structs and enums in src/workspace.rs
- [ ] T006 [P] Implement canonical-path and approved-root helper functions in src/paths.rs
- [ ] T007 [P] Add permission-specific error variants and mappings in src/error.rs
- [ ] T008 Implement session permission set load/save helpers in src/workspace.rs
- [ ] T009 Implement local structured permission event helper utilities in src/workspace.rs
- [ ] T010 [P] Define permission viewer query types for sorting, paging, and filtering in src/workspace.rs
- [ ] T011 Wire permission-event logging integration points in src/session.rs
- [ ] T012 Implement shared session lifecycle hooks for permission initialization and cleanup in src/session.rs and src/workspace.rs

**Checkpoint**: Foundation complete. User stories can now proceed.

---

## Phase 3: User Story 1 - Grant Temporary Access (Priority: P1) MVP

**Goal**: Let users grant temporary session permissions with safe boundary validation and idempotent duplicate handling.

**Independent Test**: Start a session, grant a valid path, retry the same path, and verify one active temporary grant with expected success output.

### Implementation for User Story 1

- [ ] T013 [US1] Implement ai permission grant argument parsing and input checks in src/app.rs
- [ ] T014 [P] [US1] Implement grant validation for local existence and approved-root boundaries in src/workspace.rs
- [ ] T015 [P] [US1] Implement canonicalized idempotent upsert for temporary grants in src/workspace.rs
- [ ] T016 [US1] Implement grant workflow orchestration and error mapping in src/session.rs
- [ ] T017 [US1] Emit structured grant and deny events for grant attempts in src/session.rs and src/workspace.rs
- [ ] T018 [US1] Render permission grant success and actionable deny output in src/ui.rs

**Checkpoint**: User Story 1 is independently functional and demonstrable.

---

## Phase 4: User Story 2 - Inspect Active Permissions (Priority: P2)

**Goal**: Provide a deterministic, scalable permission viewer with unavailable-path awareness.

**Independent Test**: With default and temporary permissions present, run permission view with paging and filtering and verify deterministic ordering and status visibility.

### Implementation for User Story 2

- [ ] T019 [US2] Implement ai permission view option parsing for filter/page/page-size in src/app.rs
- [ ] T020 [P] [US2] Implement deterministic permission sorting for viewer snapshots in src/workspace.rs
- [ ] T021 [P] [US2] Implement permission pagination and path filtering in src/workspace.rs
- [ ] T022 [US2] Implement unreachable-path status transition handling (active to unavailable) in src/workspace.rs
- [ ] T023 [US2] Enforce blocked access checks for unavailable grants in src/workspace.rs
- [ ] T024 [US2] Emit structured status-change events for unavailable and recovered grants in src/session.rs and src/workspace.rs
- [ ] T025 [US2] Render permission viewer output with category and status grouping in src/ui.rs
- [ ] T026 [US2] Wire permission view execution flow in src/session.rs

**Checkpoint**: User Story 2 is independently functional and demonstrable.

---

## Phase 5: User Story 3 - Automatic Permission Expiry (Priority: P3)

**Goal**: Ensure temporary permissions are revoked at session end and never leak across sessions or projects.

**Independent Test**: Grant temporary permissions, end session, start a new session, and verify temporary entries are absent while defaults remain.

### Implementation for User Story 3

- [ ] T027 [US3] Implement session-end temporary permission revocation orchestration in src/session.rs
- [ ] T028 [P] [US3] Implement persistent temporary-grant purge on session termination in src/workspace.rs
- [ ] T029 [P] [US3] Emit structured revoke events during automatic session cleanup in src/session.rs and src/workspace.rs
- [ ] T030 [US3] Ensure new session initialization loads default-only permission state in src/session.rs and src/workspace.rs
- [ ] T031 [US3] Enforce cross-project isolation of temporary permission state in src/workspace.rs
- [ ] T032 [US3] Implement ai permission revoke command parsing and dispatch in src/app.rs and src/session.rs
- [ ] T033 [US3] Render revoke and session-cleanup outcomes in src/ui.rs

**Checkpoint**: User Story 3 is independently functional and demonstrable.

---

## Phase 6: Polish and Cross-Cutting Concerns

**Purpose**: Finalize documentation, verification, and cleanup across all stories.

- [ ] T034 [P] Update permission behavior documentation in README.md and docs/permissions.md
- [ ] T035 [P] Align implementation-facing docs with final behavior in specs/003-permission-model/contracts/cli.md and specs/003-permission-model/data-model.md
- [ ] T036 Run quickstart scenarios and record implementation validation notes in specs/003-permission-model/quickstart.md
- [ ] T037 Run full test suite for regression verification in tests/workspace_tests.rs and tests/config_tests.rs
- [ ] T038 Final cleanup and consistency pass in src/app.rs, src/session.rs, src/workspace.rs, src/paths.rs, src/ui.rs, and src/error.rs

---

## Dependencies and Execution Order

### Phase Dependencies

- Setup (Phase 1): No dependencies.
- Foundational (Phase 2): Depends on Setup and blocks all user stories.
- User Story phases (Phase 3 to Phase 5): Depend on Foundational completion.
- Polish (Phase 6): Depends on all targeted user stories.

### User Story Dependencies

- US1 (P1): Depends only on Phase 2.
- US2 (P2): Depends only on Phase 2.
- US3 (P3): Depends only on Phase 2.

Recommended completion order for incremental delivery: US1 -> US2 -> US3.

### Within Each User Story

- Parsing and input handling precede session orchestration.
- Domain-state updates precede UI rendering.
- Event logging is implemented with the state transition that triggers it.

---

## Parallel Opportunities

- Phase 2: T006, T007, and T010 can run in parallel after T005 starts.
- US1: T014 and T015 can run in parallel after T013.
- US2: T020 and T021 can run in parallel after T019.
- US3: T028 and T029 can run in parallel after T027 begins.
- Phase 6: T034 and T035 can run in parallel.

## Parallel Example: User Story 1

- Task T014 [P] [US1] in src/workspace.rs
- Task T015 [P] [US1] in src/workspace.rs

## Parallel Example: User Story 2

- Task T020 [P] [US2] in src/workspace.rs
- Task T021 [P] [US2] in src/workspace.rs

## Parallel Example: User Story 3

- Task T028 [P] [US3] in src/workspace.rs
- Task T029 [P] [US3] in src/session.rs and src/workspace.rs

---

## Implementation Strategy

### MVP First (US1)

1. Complete Phase 1 and Phase 2.
2. Complete Phase 3 (US1).
3. Validate grant success, duplicate idempotency, and deny behavior.
4. Demo MVP before moving to US2.

### Incremental Delivery

1. Deliver US1 for immediate permission-control value.
2. Deliver US2 for transparent and scalable permission visibility.
3. Deliver US3 for lifecycle safety and isolation guarantees.
4. Finish with documentation and validation pass in Phase 6.

### Parallel Team Strategy

1. One developer focuses on command parsing and session dispatch.
2. One developer focuses on permission domain and persistence logic.
3. One developer focuses on UI rendering and documentation updates.
