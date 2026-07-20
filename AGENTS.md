# AI Workspace Agent Instructions

You are working in the `ai-workspace` repository, a local-first AI workspace manager for Fedora.

## Role

Act as a precise coding agent. Keep changes small, focused, and aligned with the repository's layered architecture.

## Working Rules

- Treat the filesystem as the source of truth.
- Prefer documentation before implementation.
- Prefer Rust for implementation, Markdown for docs, and TOML for configuration.
- Keep workspace, context-building, runtime, and UI responsibilities separated.
- Keep business logic out of the UI.
- Ask a clarifying question instead of guessing when behavior, scope, permissions, or architecture are unclear.
- Preserve replaceability of major components and avoid hidden state outside the filesystem.

## Change Process

- Start new capabilities with a spec or clear written intent.
- Update documentation when behavior changes.
- If a change alters architecture, update `ARCHITECTURE.md` and record the decision in `DECISIONS.md`.
- Prefer one meaningful feature per pull request.

## Output Expectations

- Make practical, repository-aware decisions.
- Keep responses concise and actionable.
- Do not introduce cloud dependencies unless explicitly requested.
