# CLI Contract: AI Workspace v0.2

## Command Surface

### ai

- Purpose: Start the interactive local workspace session.
- v0.2 impact:
  - Continues to use the configured workspace root.
  - May benefit from workspace configuration and project metadata produced by the
    workspace-manager commands.

### ai init

- Purpose: Create the expected top-level local workspace layout.
- v0.2 impact:
  - Ensures `Config/` exists so workspace configuration can be stored visibly.
  - Bootstraps `Config/workspace.toml` when it is missing.
  - Uses deterministic overwrite behavior: existing `workspace.toml` is preserved
    unless an explicit overwrite flow is introduced.
  - Leaves existing project data untouched when rerun.

### ai status

- Purpose: Print non-interactive status of root paths and discovered projects.
- v0.2 impact:
  - May include metadata-aware project summaries once project metadata exists.

### ai config show

- Purpose: Print the current workspace configuration.
- Behavior:
  - Reads `Config/workspace.toml`.
  - Displays configured workspace-managed locations and default project-creation
    structure.
- Exit behavior:
  - Exit 0 on successful read and display.
  - Non-zero when configuration is missing or invalid.

### ai project create --name NAME [--description TEXT]

- Purpose: Create a new isolated managed project workspace.
- Behavior:
  - Validates the requested project name.
  - Rejects duplicate or invalid project identifiers.
  - Creates the project directory, standard subdirectories, `README.md`, and
    `project.toml`.
- Exit behavior:
  - Exit 0 on successful project creation.
  - Non-zero on validation failure or filesystem errors.

### ai project show PROJECT

- Purpose: Display metadata and managed paths for one project.
- Behavior:
  - Resolves the named project under the configured projects directory.
  - Reads and prints the project's metadata file and standard directories.
- Exit behavior:
  - Exit 0 on success.
  - Non-zero when the project does not exist or its metadata is invalid.

### ai validate

- Purpose: Validate workspace structure, workspace configuration, and project
  metadata without modifying them.
- Behavior:
  - Checks required top-level directories.
  - Validates `Config/workspace.toml`.
  - Validates every discovered project's `project.toml` and required structure.
  - Prints pass/fail results with actionable findings.
- Exit behavior:
  - Exit 0 when the workspace is ready for use.
  - Non-zero when blocking validation errors exist or required files cannot be read.

## Managed Files

### Config/workspace.toml

- Purpose: User-visible workspace configuration.
- Minimum responsibilities:
  - Define workspace-managed directories.
  - Define the standard project subdirectory set.
  - Define the expected per-project metadata file name.

### Projects/PROJECT/project.toml

- Purpose: User-visible metadata for a single project.
- Minimum responsibilities:
  - Store stable project identity.
  - Store human-readable project name and description.
  - Store the relative paths of required standard project directories.

## Environment Variables

### AI_WORKSPACE_HOME

- Purpose: Override the default workspace root (default: `~/AI`).
- Constraints:
  - Path must resolve to a local filesystem path.
  - Commands that create or validate workspace definitions require appropriate read and
    write access to the target location.

## Validation Contract

- Validation is read-only.
- Validation reports concrete filesystem paths and actionable messages.
- Validation failure must not leave partially created project data.
- Enforcement-gate policy: commands that depend on healthy workspace definitions
  (at minimum `ai project create` and `ai project show`) MUST run a pre-action
  blocking check and refuse to continue when blocking validation issues exist.
- `ai validate` always runs and reports findings; it is the command used to
  diagnose and clear blocking issues before retrying gated actions.

## Exit Code Contract

- `0`: successful command execution and validation-ready state where applicable.
- Non-zero: invalid input, duplicate project, malformed configuration or metadata,
  unreadable required files, or unrecoverable filesystem failure.
