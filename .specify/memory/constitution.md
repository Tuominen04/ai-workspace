<!--
Sync Impact Report
- Version change: template placeholder -> 1.0.0
- Modified principles: template placeholders -> I. Filesystem First; II. Local, Fedora, and User Controlled; III. Documentation-First Spec-Gated Work; IV. Layered and Replaceable Architecture; V. Minimal, Explicit Technology Choices
- Added sections: Workspace, Data, and Permission Constraints; Development Workflow and Quality Gates
- Removed sections: none
- Templates updated: ✅ .specify/templates/plan-template.md; ✅ .specify/templates/tasks-template.md; ✅ CONTRIBUTING.md
- Deferred items: none
-->

# AI Workspace Constitution

## Core Principles

### I. Filesystem First
The filesystem is the source of truth for project state, workspace knowledge, and user
data. Avoid hidden state outside the filesystem unless an external dependency requires
it. Base knowledge is shared; project knowledge stays within the project; temporary
session state must remain temporary.

### II. Local, Fedora, and User Controlled
Default to local-first, offline-capable behavior on Fedora-native tooling. Do not add
cloud dependencies unless the user explicitly asks. Keep permissions explicit,
temporary, and revocable. Never assume write access or broader scope than the user has
granted.

### III. Documentation-First Spec-Gated Work
New capabilities start with a spec or equally clear written intent. Documentation comes
before implementation, and ambiguous decisions must be resolved by asking the user
instead of guessing. Each meaningful feature should move as one focused pull request.
Specs, plans, and tasks must remain the primary planning artifacts.

### IV. Layered and Replaceable Architecture
Respect the existing layered architecture. Keep business logic out of the user
interface, and keep workspace, context-building, search, permissions, project, and
runtime responsibilities separated. Major components must remain replaceable without
forcing broad rewrites. If a change alters architecture, update `ARCHITECTURE.md` and
record the decision in `DECISIONS.md`.

### V. Minimal, Explicit Technology Choices
Prefer Rust for implementation, Markdown for knowledge, TOML for configuration, and
Podman plus Fedora-native workflows for runtime concerns. Keep runtime and UI choices
behind replaceable interfaces. Choose the simplest implementation that satisfies the
repository's local-first and modular goals.

## Workspace, Data, and Permission Constraints

- Projects must remain isolated from one another.
- Shared base knowledge is allowed; cross-project knowledge leakage is not.
- Resources are read-only by default unless the user grants a narrower write scope.
- Temporary permissions must expire with the session and must not become hidden
	persistent grants.
- Prefer portable, filesystem-backed layouts over proprietary or opaque storage.

## Development Workflow and Quality Gates

- Start with a spec when creating new capabilities.
- Update docs alongside behavior changes.
- Keep implementation slices small enough to review and revert independently.
- Plan and task artifacts must reflect the constitution before implementation begins.
- Any architecture change must include documentation updates and an explicit decision
	record.
- Questions that affect behavior, scope, permissions, or architecture must be asked,
	not guessed.

## Governance

This constitution supersedes conflicting workflow habits, task templates, and ad hoc
preferences. Amendments require a written rationale, a semantic version bump, and any
needed updates to dependent templates or guidance documents.

All plans, specs, tasks, and reviews must check for compliance with this constitution.
When the work touches behavior, scope, permissions, or architecture, unresolved
decisions must be marked `NEEDS CLARIFICATION` or asked explicitly before proceeding.

Versioning policy:
- MAJOR: incompatible principle removals or redefinitions.
- MINOR: new principles or materially expanded governance.
- PATCH: clarifications, wording updates, or non-semantic refinements.

**Version**: 1.0.0 | **Ratified**: 2026-07-20 | **Last Amended**: 2026-07-20
