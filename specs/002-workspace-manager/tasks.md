# Tasks: Workspace Manager v0.2

**Input**: Design documents from /specs/002-workspace-manager/

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/cli.md

**Tests**: No explicit test-first or TDD requirement is stated in the feature spec, so test tasks are not listed as mandatory implementation tasks in this phase.

**Organization**: Tasks are grouped by user story so each story can be implemented and validated independently.

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Prepare command, module, and dependency scaffolding for workspace manager capabilities.

- [X] T001 Add TOML parsing and serialization dependency entries in Cargo.toml
- [X] T002 Create configuration module scaffold in src/config.rs
- [X] T003 Wire configuration module export in src/lib.rs
- [X] T004 Extend command enum and help text scaffolding for workspace-manager commands in src/app.rs

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Build shared primitives required by all user stories.

**CRITICAL**: Complete this phase before starting user story implementation.

- [X] T005 Implement workspace configuration model and read/write helpers in src/config.rs
- [X] T006 [P] Add configuration and validation error variants in src/error.rs
- [X] T007 [P] Add workspace-config and project-metadata path helpers in src/paths.rs
- [X] T008 Implement project metadata structs and load/save helpers in src/workspace.rs
- [X] T009 Implement reusable project name validation and normalization rules in src/workspace.rs
- [X] T010 Implement shared validation domain types for issues and results in src/workspace.rs
- [X] T011 [P] Add shared UI render helpers for config, project metadata, and validation results in src/ui.rs
- [X] T012 Connect foundational command handlers and dispatch plumbing in src/session.rs and src/app.rs
- [X] T034 Implement default workspace configuration bootstrap during init/first-run with deterministic overwrite rules in src/config.rs and src/session.rs
- [X] T035 Implement permission error handling contract and user-facing messages for create/show/validate flows in src/error.rs, src/session.rs, and src/ui.rs
- [X] T037 Implement blocking validation-gate primitives and reusable pre-action check helper in src/workspace.rs and src/session.rs

**Checkpoint**: Foundation complete. User stories can now proceed independently.

---

## Phase 3: User Story 1 - Create an Isolated Project Workspace (Priority: P1) MVP

**Goal**: Create a new isolated project with standard structure and metadata, while preventing duplicates and partial writes.

**Independent Test**: Run project creation with a valid name and verify the project directory, subdirectories, README, and project metadata file exist; rerun with same name and verify rejection without data mutation.

### Implementation for User Story 1

- [X] T013 [US1] Implement ai project create argument parsing in src/app.rs
- [X] T014 [P] [US1] Implement default project directory creation helper for Knowledge, Specs, Resources, and Session in src/workspace.rs
- [X] T015 [P] [US1] Implement project metadata file creation with defaults in src/workspace.rs
- [X] T016 [US1] Implement create-project workflow with duplicate checks and failure-safe behavior in src/workspace.rs
- [X] T017 [US1] Connect project-create command flow in src/session.rs
- [X] T018 [US1] Render project-create success and failure output in src/ui.rs
- [X] T038 [US1] Enforce pre-action validation gate for ai project create with blocking-issue refusal messaging in src/session.rs and src/ui.rs

**Checkpoint**: User Story 1 is independently functional and demonstrable.

---

## Phase 4: User Story 2 - Inspect and Maintain Workspace Definitions (Priority: P2)

**Goal**: Allow users to inspect workspace configuration and per-project metadata clearly.

**Independent Test**: Run config show and project show commands against an existing workspace and confirm current values, defaults, and project identity details are visible.

### Implementation for User Story 2

- [X] T019 [US2] Implement ai config show argument parsing in src/app.rs
- [X] T020 [P] [US2] Implement workspace config load and display preparation in src/config.rs
- [X] T021 [US2] Implement ai project show argument parsing in src/app.rs
- [X] T022 [P] [US2] Implement project metadata inspection loader in src/workspace.rs
- [X] T023 [US2] Connect config-show and project-show command handlers in src/session.rs
- [X] T024 [US2] Render workspace configuration and project metadata views in src/ui.rs
- [X] T036 [US2] Ensure inspection commands reflect user-maintained on-disk definition edits without AI-driven writes in src/config.rs, src/workspace.rs, and src/session.rs
- [X] T039 [US2] Enforce pre-action validation gate for ai project show with blocking-issue refusal messaging in src/session.rs and src/ui.rs

**Checkpoint**: User Story 2 is independently functional and demonstrable.

---

## Phase 5: User Story 3 - Validate Workspace State Before Use (Priority: P3)

**Goal**: Provide read-only validation that reports actionable workspace and project issues.

**Independent Test**: Run validate on a correct workspace and a malformed workspace, then verify pass/fail reporting identifies exact affected paths and guidance without mutating existing data.

### Implementation for User Story 3

- [X] T025 [US3] Implement ai validate argument parsing in src/app.rs
- [X] T026 [US3] Implement workspace-level validation checks for layout and workspace config in src/workspace.rs
- [X] T027 [P] [US3] Implement per-project metadata and required-directory validation checks in src/workspace.rs
- [X] T028 [US3] Connect validation command flow and exit semantics in src/session.rs
- [X] T029 [US3] Render actionable validation reports in src/ui.rs
- [X] T040 [US3] Render explicit gated-command guidance in validation output (which commands are blocked and how to clear issues) in src/ui.rs

**Checkpoint**: User Story 3 is independently functional and demonstrable.

---

## Phase 6: Polish and Cross-Cutting Concerns

**Purpose**: Final alignment, documentation, and execution-readiness checks across stories.

- [X] T030 [P] Align workspace-manager command contract details with delivered behavior in specs/002-workspace-manager/contracts/cli.md
- [X] T031 [P] Update user-facing command documentation for workspace-manager features in README.md and docs/workspace.md
- [X] T032 Run quickstart scenarios and record implementation validation notes in specs/002-workspace-manager/quickstart.md
- [X] T033 Perform final code cleanup and formatting in src/app.rs, src/session.rs, src/workspace.rs, src/config.rs, src/ui.rs, src/error.rs, and src/paths.rs

---

## Dependencies and Execution Order

### Phase Dependencies

- Setup (Phase 1): No dependencies.
- Foundational (Phase 2): Depends on Setup completion and blocks all user stories.
- User Story phases (Phase 3 to Phase 5): Each depends on Foundational completion.
- Polish (Phase 6): Depends on completion of all targeted user stories.

### User Story Dependencies

- US1 (P1): Depends only on Foundational completion.
- US2 (P2): Depends only on Foundational completion.
- US3 (P3): Depends only on Foundational completion.

### Within Each User Story

- Command parsing tasks precede command handler wiring.
- Data read/write helpers precede workflow orchestration.
- Workflow orchestration precedes UI output polishing.

---

## Parallel Execution Opportunities

- Phase 2: T006, T007, and T011 can run in parallel.
- US1: T014 and T015 can run in parallel once T013 is complete.
- US2: T020 and T022 can run in parallel once T019 and T021 are complete.
- US3: T027 can run in parallel with T026 after T025 is complete.
- Phase 6: T030 and T031 can run in parallel.

## Parallel Example: User Story 1

- T014 [P] [US1] Implement default project directory creation helper for Knowledge, Specs, Resources, and Session in src/workspace.rs
- T015 [P] [US1] Implement project metadata file creation with defaults in src/workspace.rs

## Parallel Example: User Story 2

- T020 [P] [US2] Implement workspace config load and display preparation in src/config.rs
- T022 [P] [US2] Implement project metadata inspection loader in src/workspace.rs

## Parallel Example: User Story 3

- T026 [US3] Implement workspace-level validation checks for layout and workspace config in src/workspace.rs
- T027 [P] [US3] Implement per-project metadata and required-directory validation checks in src/workspace.rs

---

## Implementation Strategy

### MVP First (US1)

1. Complete Phase 1 and Phase 2.
2. Complete Phase 3 (US1).
3. Validate project creation and duplicate rejection behavior.
4. Demo MVP before moving to US2 and US3.

### Incremental Delivery

1. Deliver US1 after foundation for immediate value.
2. Deliver US2 for inspectability and operator confidence.
3. Deliver US3 for safety and readiness checks.
4. Finish with documentation and quickstart validation in Phase 6.

### Team Parallelization

1. One developer completes command parsing and dispatch backbone.
2. One developer focuses on workspace and metadata domain logic.
3. One developer focuses on UI rendering and documentation updates.
