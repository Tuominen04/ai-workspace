# Implementation Plan: AI CLI Foundation v0.1

**Branch**: `001-ai-cli-foundation` | **Date**: 2026-07-20 | **Spec**: /specs/001-ai-cli-foundation/spec.md

**Input**: Feature specification from `/specs/001-ai-cli-foundation/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command; its definition describes the execution workflow.

## Summary

Deliver the first runnable CLI version of AI Workspace with command ai. The feature
focuses on interactive workspace selection, Base-only AI context visibility
(~/AI/Base), local runtime status handling for Ollama, and clean session shutdown.
Implementation follows a layered Rust module design with filesystem-first state and no
external runtime dependencies beyond local system tools.

## Technical Context

**Language/Version**: Rust 2024 edition

**Primary Dependencies**: Rust standard library only for v0.1

**Storage**: Local filesystem layout under ~/AI (or AI_WORKSPACE_HOME override)

**Testing**: cargo test and manual CLI validation scenarios from quickstart

**Target Platform**: Fedora-compatible Linux terminals

**Project Type**: Local CLI application

**Performance Goals**: Reach usable session startup in under 10 seconds for local valid inputs

**Constraints**: Local-first, Base-only AI-readable context, no hidden non-filesystem state, minimal dependencies

**Scale/Scope**: Single-user local CLI for v0.1; one interactive session at a time

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- Filesystem source of truth: PASS
- Local-first Fedora-native direction: PASS
- Documentation-first before implementation: PASS (spec and plan created under specs/001-ai-cli-foundation)
- Ambiguous behavior/scope decisions unresolved: PASS (no unresolved clarification markers)
- Architecture update required: NOT REQUIRED (no architecture redefinition; implementation conforms to existing architecture docs)
- Layer boundary preservation in plan: PASS
- User story decomposition for independent testing: PASS

Post-Design Re-Check (after Phase 1 artifacts):

- research.md created and aligns with constitution constraints: PASS
- data-model.md preserves filesystem-first and Base-only context policy: PASS
- contracts/cli.md preserves explicit command and runtime boundaries: PASS
- quickstart.md validates policy and edge-case behavior: PASS

## Project Structure

### Documentation (this feature)

```text
specs/001-ai-cli-foundation/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
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
├── context.rs
├── runtime.rs
├── ui.rs
└── error.rs

Cargo.toml
specs/001-ai-cli-foundation/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
└── contracts/
    └── cli.md
```

**Structure Decision**: Single Rust CLI project with layered modules in src/ to keep
workspace orchestration, runtime lifecycle, context policy, and UI output separated.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

| Violation | Why Needed | Simpler Alternative Rejected Because |
| -- | -- | -- |
| None | N/A | N/A |
