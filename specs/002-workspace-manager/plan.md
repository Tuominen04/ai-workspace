# Implementation Plan: Workspace Manager v0.2

**Branch**: `002-workspace-manager` | **Date**: 2026-07-22 | **Spec**: /specs/002-workspace-manager/spec.md

**Input**: Feature specification from `/specs/002-workspace-manager/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command; its definition describes the execution workflow.

## Summary

Deliver the v0.2 Workspace Manager layer for AI Workspace. The feature adds
filesystem-visible workspace configuration, per-project metadata, project creation,
metadata inspection, and non-mutating validation. Implementation stays within the
existing single-binary Rust CLI and extends the current workspace module design rather
than redefining architecture.

## Technical Context

**Language/Version**: Rust 2024 edition

**Primary Dependencies**: Rust standard library plus minimal TOML serialization
support for user-visible configuration and metadata files

**Storage**: Local filesystem under `~/AI` (or `AI_WORKSPACE_HOME` override), with
workspace config in `Config/workspace.toml` and per-project metadata in
`Projects/<project>/project.toml`

**Testing**: `cargo test` plus manual CLI validation scenarios from quickstart

**Target Platform**: Fedora-compatible Linux terminals

**Project Type**: Local CLI application

**Performance Goals**: Project creation and metadata inspection should complete in
under 2 seconds of command time for valid local inputs; validation should complete in
under 5 seconds for a workspace with dozens of projects

**Constraints**: Local-first, offline-capable, filesystem-visible definitions only,
project isolation preserved, TOML configuration, no automatic permission or resource
management in this phase

**Scale/Scope**: Single-user local workspace manager for tens of projects with a
single shared Base directory and per-project isolated metadata

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- Filesystem source of truth: PASS
- Local-first Fedora-native direction: PASS
- Documentation-first before implementation: PASS (spec and planning artifacts live
  under `specs/002-workspace-manager/`)
- Ambiguous behavior, scope, permissions, or architecture decisions unresolved: PASS
  (no remaining clarification markers)
- Architecture update required: NOT REQUIRED (the feature extends current workspace
  and CLI responsibilities without redefining the architecture)
- Layer boundary preservation in plan: PASS
- User story decomposition for independent testing: PASS

Post-Design Re-Check (after Phase 1 artifacts):

- research.md aligns with filesystem-first and TOML configuration choices: PASS
- data-model.md preserves project isolation and user-visible metadata rules: PASS
- contracts/cli.md keeps workspace, configuration, validation, and UI responsibilities
  explicit: PASS
- quickstart.md validates behavior without requiring architecture changes: PASS

## Project Structure

### Documentation (this feature)

```text
specs/002-workspace-manager/
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
├── main.rs
├── lib.rs
├── app.rs
├── session.rs
├── paths.rs
├── workspace.rs
├── config.rs
├── context.rs
├── runtime.rs
├── ui.rs
└── error.rs

Cargo.toml
specs/002-workspace-manager/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
└── contracts/
    └── cli.md
```

**Structure Decision**: Keep the existing single Rust CLI project and extend it with a
dedicated configuration module for TOML parsing and serialization. Keep workspace
creation, discovery, metadata loading, and validation behavior within the workspace
layer, while `app.rs` and `session.rs` continue to coordinate command flow and `ui.rs`
continues to own presentation.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

| Violation | Why Needed | Simpler Alternative Rejected Because |
|:----------|:-----------|:-------------------------------------|
| None      | N/A        | N/A                                  |
