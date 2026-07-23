# Implementation Plan: v0.3 Permission Model

**Branch**: `003-permission-model` | **Date**: 2026-07-22 | **Spec**: /specs/003-permission-model/spec.md

**Input**: Feature specification from `/specs/003-permission-model/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command; its definition describes the execution workflow.

## Summary

Deliver v0.3 permission management for local AI workspace sessions with explicit
temporary grants, session-scoped expiry, permission visibility, and permission-event
logging. The feature extends existing Rust CLI and workspace/session layers while
preserving local-first behavior, filesystem-source-of-truth constraints, and project
isolation.

## Technical Context

**Language/Version**: Rust 2024 edition

**Primary Dependencies**: Rust standard library, existing project modules, minimal
TOML support already used by workspace metadata/configuration, structured logging
support already present in the codebase approach

**Storage**: Local filesystem only; session-scoped permission state stored in
`Projects/<project>/Session/permissions.toml` and runtime state constrained to the
active session lifecycle

**Testing**: `cargo test` plus targeted unit/integration tests for grant validation,
session cleanup, and viewer/logging behavior; manual CLI validation via quickstart

**Target Platform**: Fedora-compatible Linux terminals

**Project Type**: Local CLI application

**Performance Goals**: Permission grant/deny decisions should complete within 200 ms
for typical local paths; permission view responses should render first page in under
500 ms for up to 1,000 active grants

**Constraints**: Local-first and offline-capable; no cloud dependencies; explicit
and revocable temporary permissions; local-path boundary enforcement; deterministic
sorting and pagination for viewer scalability; temporary permissions auto-expire at
session end

**Scale/Scope**: Single-user local workflow, tens of projects, and up to low-thousands
of session permission entries in edge-case active sessions

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- Filesystem source of truth: PASS
- Local-first and Fedora-native constraints: PASS
- Documentation-first flow: PASS (spec and clarification log complete before planning)
- Behavior/scope/permission ambiguities resolved: PASS (no remaining clarification
  markers)
- Architecture update required: NOT REQUIRED (feature extends existing responsibilities
  without changing architecture contract)
- Layer boundaries preserved in plan: PASS
- Independently testable stories: PASS

Post-Design Re-Check (after Phase 1 artifacts):

- research.md aligns with local-first, explicit permissions, and replaceable
  architecture constraints: PASS
- data-model.md preserves session-scoped temporary state and project isolation: PASS
- contracts/cli.md keeps presentation concerns in CLI surface while permission logic
  remains in workspace/session domain: PASS
- quickstart.md validates expected behavior and failure paths without architecture
  changes: PASS

## Project Structure

### Documentation (this feature)

```text
specs/003-permission-model/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   └── cli.md
└── tasks.md
```

### Source Code (repository root)

```text
src/
├── app.rs
├── config.rs
├── context.rs
├── error.rs
├── lib.rs
├── main.rs
├── paths.rs
├── runtime.rs
├── session.rs
├── ui.rs
└── workspace.rs

tests/
├── config_tests.rs
└── workspace_tests.rs

specs/003-permission-model/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
└── contracts/
  └── cli.md
```

**Structure Decision**: Keep the existing single Rust CLI project structure and add
permission behavior through existing workspace/session-oriented modules rather than
introducing new architectural layers. Keep CLI interface additions in command handling
flow, keep permission-domain decisions in workspace/session logic, and keep output
formatting responsibilities in UI presentation code.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

| Violation | Why Needed | Simpler Alternative Rejected Because |
| -- | -- | -- |
| None | N/A | N/A |
