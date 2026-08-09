# Implementation Plan: v0.5 Filesystem Search

**Branch**: `006-filesystem-search` | **Date**: 2026-08-09 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/006-filesystem-search/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command; its definition describes the execution workflow.

## Summary

Add a simple, dependency-free filesystem search over the workspace's Markdown knowledge (Base + active Project + Session), scoped to whatever paths the user currently holds permission for. Search doubles as (a) a standalone `ai search` lookup command usable without starting a session, and (b) the relevance mechanism the Context Builder (v0.4) now uses to pick which documents go into the assembled context — replacing the current "first N under the cap, in fixed priority order" behavior with relevance-ranked selection whenever a query (the user's question) is available, while keeping the existing priority order and caps as the tie-break and ceiling.

## Technical Context

**Language/Version**: Rust, 2024 edition (matches `Cargo.toml`)

**Primary Dependencies**: None new. Reuses the existing `std` filesystem/string APIs already used by `context.rs` and `workspace.rs`; `serde`/`toml`/`ureq` remain unchanged. No search/indexing crate is introduced (see research.md).

**Storage**: Filesystem only — Markdown files already under `Base/`, `Projects/<name>/Knowledge/`, and `Projects/<name>/Session/`. Search reads existing content; it does not write or cache anything new to disk (filesystem stays the sole source of truth).

**Testing**: `cargo test` — new `tests/search_tests.rs` integration tests via `assert_cmd`, following the existing pattern in `tests/context_tests.rs` and `tests/workspace_tests.rs`; existing `context_tests.rs` updated for the new `build_context_snapshot` signature.

**Target Platform**: Linux (Fedora), same as the rest of the project.

**Project Type**: Single Rust CLI binary (existing flat `src/` module layout).

**Performance Goals**: Search over a workspace with hundreds of Markdown documents completes in under 2 seconds (SC-003); per-turn context re-assembly in the chat loop must not add noticeable latency on top of the existing Ollama round trip for typical (tens of documents) workspaces.

**Constraints**: Offline/local-only — no network calls involved in search; must not exceed or bypass the Context Builder's existing `DEFAULT_MAX_CONTEXT_SOURCES` (8) / `DEFAULT_MAX_CONTEXT_BYTES` (12000) caps; must never surface a file outside the user's current Default/Temporary permission grants.

**Scale/Scope**: Workspace-scale document counts (tens to low hundreds of Markdown files per project/base). Not designed for corpora large enough to need indexing, embeddings, or a vector store — those are explicitly later phases (Search Abstraction Phase 2+ in `ARCHITECTURE.md`).

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **Filesystem remains the source of truth; no hidden state outside the filesystem.** ✅ Search reads Markdown already on disk via the existing `collect_markdown_files`/permission machinery; it introduces no cache, index file, or database.
- **Work stays local-first and Fedora-native.** ✅ No network dependency; no new external service.
- **New behavior preceded by documentation.** ✅ `specs/006-filesystem-search/spec.md` precedes this plan.
- **Questions affecting behavior/scope/permissions/architecture resolved before continuing.** ✅ The one spec-level ambiguity (standalone-only vs. integrated into context assembly) was resolved with the user during `/speckit-specify` — integrated into context assembly.
- **Architecture changes require `ARCHITECTURE.md` + `DECISIONS.md` updates.** ⚠️ Action required, not a violation: `ARCHITECTURE.md` already anticipates a "Layer 5 — Search Engine" module, but `CLAUDE.md`'s module-responsibility list and `build_context_snapshot`'s public signature (gains a `query` parameter) need updating. Tracked as an implementation task (see tasks phase), not a Complexity Tracking violation — the module boundary was already documented, this plan fills it in.
- **Runtime/workspace/permissions/search/project/UI boundaries stay separate.** ✅ New `search.rs` owns scoring/ranking/snippets; `workspace.rs` continues to own permission and file-collection logic; `context.rs` continues to own assembly/caps; `ui.rs` only renders; `app.rs`/`session.rs` continue to own dispatch/orchestration. No `std::fs` is added to `ui.rs`, `app.rs`, or `runtime.rs`.
- **Multiple meaningful deliverables split into independently testable user stories.** ✅ Already done in spec.md (P1 search-and-rank, P2 context integration, P3 result visibility).

No violations requiring justification; Complexity Tracking is omitted.

## Project Structure

### Documentation (this feature)

```text
specs/006-filesystem-search/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md         # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
│   └── cli-search.md
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)

```text
# Single project (existing flat module layout, no new subdirectories)
src/
├── app.rs        # MODIFIED: add `ai search` command parsing/dispatch
├── config.rs     # unchanged
├── context.rs    # MODIFIED: build_context_snapshot gains an optional query; relevance-ranked
│                 #   selection when a query is present, existing priority-order/caps otherwise
├── error.rs      # MODIFIED: add error variant(s) for empty/invalid search queries
├── paths.rs      # unchanged
├── runtime.rs    # unchanged
├── search.rs     # NEW: Layer 5 Search Engine — scoring, ranking, snippet extraction
├── session.rs    # MODIFIED: `run_search` orchestration; run_chat_loop re-assembles context
│                 #   per user turn using the message as the query
├── ui.rs         # MODIFIED: render search results / "no matches" state
└── workspace.rs  # unchanged (existing collect_markdown_files + permission APIs are reused as-is)

tests/
├── app_version_tests.rs  # unchanged
├── config_tests.rs       # unchanged
├── context_tests.rs      # MODIFIED: cover relevance-based selection + updated call signature
├── runtime_tests.rs      # unchanged
├── search_tests.rs       # NEW: standalone search happy path, no-results, permission boundary
└── workspace_tests.rs    # unchanged
```

**Structure Decision**: Extends the existing single flat `src/` layout (per `CLAUDE.md`, the nested `core/`/`search/`/etc. folders in `PROJECR_STRUCTURE.md` remain aspirational and are not introduced here). One new module, `src/search.rs`, is added as the Search Engine layer already named in `ARCHITECTURE.md`; all filesystem access it needs is obtained through `workspace.rs`'s existing `collect_markdown_files` and permission APIs rather than duplicating traversal logic, and file-content reads follow the same `std::fs::read_to_string` pattern `context.rs` already uses (not the restricted `ui.rs`/`app.rs`/`runtime.rs` set).

## Post-Design Constitution Check

*Re-evaluated after Phase 1 (data-model.md, contracts/, quickstart.md).*

- Filesystem-as-source-of-truth, local-first, layer-separation, and permission-boundary gates all still hold — confirmed concretely in `contracts/context-builder-integration.md` (search stays a pure function over caller-supplied, already permission-filtered candidates; no cache or index is introduced) and `data-model.md` (no entity is persisted).
- The one action item flagged pre-design (update `CLAUDE.md`'s module list for `search.rs`, update `ARCHITECTURE.md`/`DECISIONS.md` for the `build_context_snapshot` signature change) is now precisely scoped by `contracts/context-builder-integration.md` and carries into the tasks phase as concrete documentation tasks, not an open risk.
- No new violations introduced by the Phase 1 design. Complexity Tracking remains empty.

## Complexity Tracking

*No violations — table omitted.*
