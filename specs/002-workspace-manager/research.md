# Research: Workspace Manager v0.2

## Decision 1: Use TOML for workspace configuration

- Decision: Store workspace-level configuration in `Config/workspace.toml`.
- Rationale: The repository's stack guidance explicitly prefers TOML for
  human-readable configuration with strong Rust ecosystem support. Keeping the config
  in the workspace filesystem preserves portability and user control.
- Alternatives considered:
  - JSON config: Rejected because TOML is the repository default for configuration.
  - Environment variables only: Rejected because they are harder to inspect and do not
    provide a durable workspace definition.

## Decision 2: Store metadata inside each project workspace

- Decision: Store per-project metadata in `Projects/<project>/project.toml` rather than
  in a centralized registry.
- Rationale: This keeps each project self-describing and isolated, matches the
  filesystem-first architecture, and avoids hidden or cross-project state.
- Alternatives considered:
  - Central workspace registry file: Rejected because it creates coupling between
    projects and makes recovery harder when one project is moved independently.
  - README-only metadata: Rejected because it is less reliable for validation and
    command-driven inspection.

## Decision 3: Create projects with a standard minimal structure

- Decision: New projects should start with `Knowledge/`, `Specs/`, `Resources/`,
  `Session/`, `README.md`, and `project.toml`.
- Rationale: The architecture documentation already defines these project-level
  concerns. Creating them up front gives users a predictable layout and supports later
  context, permissions, and project-management phases.
- Alternatives considered:
  - Empty project directories: Rejected because they push structural errors into later
    commands.
  - Multiple templates or archetypes: Rejected as unnecessary complexity for v0.2.

## Decision 4: Keep validation read-only and explicit

- Decision: Validation should inspect workspace layout, workspace config, and project
  metadata without modifying them.
- Rationale: Non-mutating validation preserves trust, avoids hidden repair logic, and
  aligns with the requirement that failures must not change existing project data.
- Alternatives considered:
  - Auto-fix missing files during validation: Rejected because it blurs validation and
    mutation and can create unexpected state changes.
  - Validate only at creation time: Rejected because existing workspaces must remain
    checkable without creating new projects.

## Decision 5: Extend the CLI with explicit workspace-manager commands

- Decision: Expose v0.2 behavior through explicit commands such as `ai project create`,
  `ai project show`, `ai config show`, and `ai validate`.
- Rationale: The application is currently a CLI, and explicit commands keep the
  workspace-management surface discoverable, scriptable, and separate from the
  interactive session flow.
- Alternatives considered:
  - Fold everything into `ai status`: Rejected because it mixes inspection, mutation,
    and validation concerns.
  - Interactive creation prompts only: Rejected because they reduce repeatability and
    scriptability.

## Decision 6: Preserve the current layered Rust module shape

- Decision: Extend the current flat `src/` module layout by adding a configuration
  module while keeping workspace logic in the workspace layer and presentation in the
  UI layer.
- Rationale: Existing v0.1 modules already separate paths, workspace discovery,
  runtime, context, session, and UI concerns. v0.2 can build on that without
  requiring an architecture rewrite.
- Alternatives considered:
  - Large refactor into many nested module trees before v0.2: Rejected because it
    would broaden scope without delivering workspace-manager behavior.
  - Put config and validation logic in the UI or session layer: Rejected because it
    violates architectural boundaries.
