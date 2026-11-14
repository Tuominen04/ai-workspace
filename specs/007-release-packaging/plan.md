# Implementation Plan: Release Packaging

**Branch**: `007-release-packaging` | **Date**: 2026-11-14 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/007-release-packaging/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command; its definition describes the execution workflow.

## Summary

Make every version installable on Fedora the way Learning Corner is: pushing a `v<version>` tag runs a GitHub Actions workflow that checks the tag, runs the tests, builds a release binary from the committed lock, packs it into an `.rpm`, smoke-tests that package in a Fedora container, and attaches it with a `SHA256SUMS` file to a **draft** release the maintainer publishes by hand.

Three things ship together (spec FR-004, accepted exception to one-feature-per-PR):

1. **Rename** the command from `ai` to `ai-workspace` — binary name, every usage/help/error string, tests, and current documentation.
2. **License** the project `MIT OR Apache-2.0` and add both license texts.
3. **Package and release** — RPM metadata in `Cargo.toml`, one release workflow, and install/verify/release documentation.

No change to what the workspace, permission, context, search, or runtime layers do.

## Technical Context

**Language/Version**: Rust, 2024 edition (matches `Cargo.toml`). The release workflow uses the current stable toolchain; edition 2024 needs Rust 1.85 or newer.

**Primary Dependencies**: No new crate dependencies. One new **build-time tool**, `cargo-generate-rpm` pinned to `0.21.1`, installed only inside the release workflow (and optionally locally for the quickstart). Existing `serde`/`serde_json`/`toml`/`ureq` are unchanged.

**Storage**: None. The package installs three files (the binary and two license texts) and touches nothing under the user's home directory.

**Testing**: `cargo test --locked`. Existing integration tests switch from `cargo_bin("ai")` to `cargo_bin("ai-workspace")`; `tests/app_version_tests.rs` asserts the new version line; one new test asserts `--help` names the command `ai-workspace` and no usage line still says `ai`. The package itself is verified in the workflow by installing it in a `fedora:44` container, and by hand through `quickstart.md`.

**Target Platform**: Fedora Linux, x86_64 only.

**Project Type**: Single Rust CLI binary (existing flat `src/` module layout) plus one CI workflow.

**Performance Goals**: A draft release appears within 15 minutes of pushing a tag (SC-002). No run-time performance change.

**Constraints**: The installed tool stays offline — no network at run time, no telemetry, no background service (FR-018). The release job that can write to the repository runs no project or dependency code (FR-017). Installing, upgrading, or removing the package never touches the workspace folder (FR-008). The repository is private: no build attestation, no tag rulesets.

**Scale/Scope**: One package, one architecture, one workflow file, roughly 45 string literals renamed across five source files, four current documentation files updated.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **I. Filesystem First.** ✅ No state is added anywhere. The package owns only files under `/usr`; the workspace folder stays the sole source of truth and is never touched by install, upgrade, or removal.
- **II. Local, Fedora, and User Controlled.** ✅ with one explicit, requested exception. The *release process* runs on GitHub Actions — a cloud service — because the user asked for it ("like Learning Corner"). It is build-time only; the tool that gets installed makes no network calls. The output is a Fedora-native `.rpm` installed with `dnf`.
- **III. Documentation-First Spec-Gated Work.** ✅ `spec.md` precedes this plan; both clarifications were answered by the user. ⚠️ **Accepted exception**: the rename, the license, and the packaging ship in one pull request by the user's decision (spec FR-004 and Assumptions). See Complexity Tracking.
- **IV. Layered and Replaceable Architecture.** ✅ No layer boundary moves. The rename is confined to strings and the binary name; the command name becomes one shared constant instead of ~45 literals. No `std::fs` is added to `ui.rs`, `app.rs`, or `runtime.rs`. `ARCHITECTURE.md` needs no change; the packaging, naming, and license decisions are recorded as a new ADR in `DECISIONS.md`.
- **V. Minimal, Explicit Technology Choices.** ✅ Rust stays the implementation language and TOML carries the package metadata (in `Cargo.toml`). ⚠️ One new format, YAML, for the workflow file — required by GitHub Actions, no alternative. One new tool, `cargo-generate-rpm`, chosen as the simplest option (see research.md Decision 1).
- **Workspace, Data, and Permission Constraints.** ✅ Unaffected; no permission or project-isolation code changes.
- **Quality Gates.** ✅ Docs are updated alongside the behavior change (README, `CLAUDE.md`, `docs/workspace.md`, `docs/permissions.md`, new `docs/release.md`); the architecture-relevant decision gets an ADR.

One exception requires justification; see Complexity Tracking.

## Project Structure

### Documentation (this feature)

```text
specs/007-release-packaging/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
│   ├── cli-naming.md        # what the command calls itself
│   ├── rpm-package.md       # package name, metadata, installed files
│   └── release-workflow.md  # trigger, gates, outputs of the release process
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)

```text
.github/
└── workflows/
    └── release.yml   # NEW: tag-triggered build → package → smoke test → draft release

Cargo.toml            # MODIFIED: [[bin]] name = "ai-workspace"; version 0.6.0; license;
                      #   [package.metadata.generate-rpm] (name, assets, license files)
Cargo.lock            # MODIFIED: version only
LICENSE-MIT           # NEW
LICENSE-APACHE        # NEW

src/
├── lib.rs        # MODIFIED: add `pub const COMMAND_NAME: &str = "ai-workspace";`
├── app.rs        # MODIFIED: help text, usage errors, version line use COMMAND_NAME
├── error.rs      # MODIFIED: one usage message
├── session.rs    # MODIFIED: gate labels and the "rerun" hint
├── ui.rs         # MODIFIED: validation-gate and no-results hints
├── workspace.rs  # MODIFIED: `suggested_fix` strings
├── config.rs, context.rs, paths.rs, runtime.rs, search.rs, main.rs   # unchanged

tests/
├── app_version_tests.rs  # MODIFIED: binary name, version line, new help-naming test
├── search_tests.rs       # MODIFIED: binary name
└── (others)              # MODIFIED only where they invoke the binary or assert on `ai ...` text

README.md             # MODIFIED: command name; new Install, Verify, License sections
CLAUDE.md             # MODIFIED: binary name, commands, current version
ROADMAP.md            # MODIFIED: v0.6 is release packaging
DECISIONS.md          # MODIFIED: ADR-0006 (packaging, command name, license)
docs/
├── release.md        # NEW: how to cut a release; how to verify a download
├── workspace.md      # MODIFIED: command name
└── permissions.md    # MODIFIED: command name
```

**Structure Decision**: Extends the existing flat `src/` layout; no new module. The only new top-level items are the two license files and `.github/workflows/release.yml`. Earlier specs under `specs/001-*` to `specs/006-*` are historical records and keep the `ai` name as written.

## Post-Design Constitution Check

*Re-evaluated after Phase 1 (data-model.md, contracts/, quickstart.md).*

- Filesystem-first and workspace-safety gates hold concretely: `contracts/rpm-package.md` lists exactly three installed files, all under `/usr`, and no install scripts.
- The local-first gate holds: `contracts/release-workflow.md` confines network use to the release process; nothing in `contracts/cli-naming.md` adds run-time behavior.
- The write-permission separation (FR-017) is fixed by the two-job shape in `contracts/release-workflow.md`.
- Layering is unchanged; the single `COMMAND_NAME` constant in `contracts/cli-naming.md` reduces duplication rather than adding a layer.
- No new violations. The one accepted exception below stands as recorded.

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| Three changes in one pull request (rename, license, packaging) against "one meaningful feature per pull request" | The package fixes the installed command's name and must declare a license; shipping `ai-workspace` as a package requires all three at once | Splitting was offered (rename first, then packaging) and the maintainer chose one pull request on 2026-11-14. The work is still organised as separately reviewable commits and independently testable user stories. |
