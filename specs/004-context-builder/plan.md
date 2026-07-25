# Implementation Plan: v0.4 Context Builder

**Branch**: `004-context-builder` | **Date**: 2026-07-25 | **Spec**: /specs/004-context-builder/spec.md

**Input**: Feature specification from `/specs/004-context-builder/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command; its definition describes the execution workflow.

## Summary

Deliver v0.4 context building for local AI workspace sessions by assembling a bounded, deterministic context package from shared base knowledge and the selected project's knowledge. The feature extends the existing Rust CLI and context modules while preserving the repository's local-first, filesystem-backed architecture and project isolation boundaries.

## Technical Context

**Language/Version**: Rust 2024 edition

**Primary Dependencies**: Existing Rust modules in `src/context.rs`, `src/session.rs`, `src/ui.rs`, and `src/workspace.rs`; no new external libraries are required for the initial implementation.

**Storage**: Local filesystem only; context is assembled from files already present under `Base/` and `Projects/<project>/`.

**Testing**: `cargo test` plus targeted unit tests for source selection, ordering, and size-limit behavior; manual CLI validation through the quickstart scenarios.

**Target Platform**: Fedora-compatible Linux terminals

**Project Type**: Local CLI application

**Performance Goals**: Context assembly should complete quickly for typical workspace sizes while keeping the generated prompt bounded and useful.

**Constraints**: Local-first and offline-capable; no cloud dependencies; prompt size must remain bounded; project isolation must be preserved; missing sources must not break session startup.

**Scale/Scope**: Single-user local workflow, a small number of projects, and a moderate number of Markdown files in base and project knowledge directories.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- Filesystem source of truth: PASS
- Local-first and Fedora-native constraints: PASS
- Documentation-first flow: PASS
- Behavior/scope/permission ambiguities resolved: PASS
- Architecture update required: NOT REQUIRED (feature extends existing responsibilities without changing the architecture contract)
- Layer boundaries preserved in plan: PASS
- Independently testable stories: PASS

Post-Design Re-Check (after Phase 1 artifacts):

- research.md aligns with filesystem-backed context assembly and bounded prompt design: PASS
- data-model.md preserves the distinction between base, project, and session sources: PASS
- contracts/cli.md keeps output and summary responsibilities in the CLI/session flow while context logic remains in the context layer: PASS
- quickstart.md validates startup, project switching, and bounded context behavior without introducing architecture changes: PASS

## Project Structure

### Documentation (this feature)

```text
specs/004-context-builder/
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
├── app_version_tests.rs
├── config_tests.rs
└── workspace_tests.rs
```

**Structure Decision**: Keep the existing single Rust CLI project structure and add context-building behavior through the existing context/session/UI modules. The context module will own source discovery and package assembly, while session orchestration will select the active project and UI will render the resulting summary.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

| Violation | Why Needed | Simpler Alternative Rejected Because |
| -- | -- | -- |
| None | N/A | N/A |
