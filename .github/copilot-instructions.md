# AI Workspace Instructions

## Project Intent

This repository is a local-first AI workspace manager for Fedora.
Treat the filesystem as the source of truth.
Keep the system project-aware, isolated, and user-controlled.

## Default Working Rules

- Prefer documentation before implementation. This is a strict rule.
- Keep changes small, focused, and architecture-friendly.
- Ask a clarifying question instead of guessing when a decision affects behavior, scope, permissions, or architecture. This is a hard rule.
- Do not introduce cloud dependencies unless the user explicitly asks.
- Preserve replaceability of major components.

## Architecture Rules

- Respect the existing layered architecture.
- Keep business logic out of the user interface.
- Keep workspace, context-building, and runtime responsibilities separated.
- If a change alters architecture, update `ARCHITECTURE.md` and record the decision in `DECISIONS.md`.

## Technology Defaults

- Prefer Rust for implementation.
- Prefer Markdown for knowledge and documentation.
- Prefer TOML for configuration.
- Prefer Podman and Fedora-native workflows.
- Keep runtime and UI choices abstracted behind replaceable interfaces.

## Workspace Rules

- Projects must remain isolated from each other.
- Base knowledge is shared; project knowledge stays within the project.
- Temporary permissions should stay temporary.
- Avoid hidden state outside the filesystem unless external software requires it.

## Change Process

- Start new capabilities with a spec or clear written intent.
- Update documentation when behavior changes.
- Prefer one meaningful feature per pull request. This is a strict rule.
- If a request is underspecified, stop and ask what should apply, where it should apply, and whether it is a hard rule or a preference.
