# Quickstart Validation: Workspace Manager v0.2

## Prerequisites

- Fedora-compatible Linux environment.
- Rust toolchain installed (`cargo`, `rustc`).
- Existing v0.1 CLI baseline available in this repository.

## Setup

1. Build the binary:
   - `cargo build`
2. Ensure the workspace root exists:
   - `./target/debug/ai init`
3. Confirm the initial workspace layout:
   - `./target/debug/ai status`

## Validation Scenarios

### Scenario 1: Show workspace configuration

1. Run:
   - `./target/debug/ai config show`
2. Expected:
   - The command prints the active workspace-managed paths.
   - The command identifies the standard project structure and metadata file name.

### Scenario 2: Create a managed project

1. Run:
   - `./target/debug/ai project create --name demo-project --description "Demo workspace"`
2. Expected:
   - A new project directory appears under `~/AI/Projects/` or the overridden root.
   - The project contains `Knowledge/`, `Specs/`, `Resources/`, `Session/`,
     `README.md`, and `project.toml`.

### Scenario 3: Inspect project metadata

1. Run:
   - `./target/debug/ai project show demo-project`
2. Expected:
   - The command prints the project's metadata and standard managed paths.
   - The project description matches the value provided at creation time.

### Scenario 4: Validate a healthy workspace

1. Run:
   - `./target/debug/ai validate`
2. Expected:
   - The command reports the workspace as ready.
   - No project data is modified during the validation run.

### Scenario 5: Reject duplicate project creation

1. Run project creation twice with the same project name.
2. Expected:
   - The second command fails with a clear duplicate-name message.
   - The first project's data remains unchanged.

### Scenario 6: Detect malformed project metadata

1. Edit `project.toml` for a test project so a required field is missing or invalid.
2. Run:
   - `./target/debug/ai validate`
3. Expected:
   - The command reports a blocking validation error for the affected project.
   - Unaffected projects still appear in the validation report.

### Scenario 7: Detect missing workspace definitions

1. Temporarily remove or rename `Config/workspace.toml`.
2. Run:
   - `./target/debug/ai config show`
   - `./target/debug/ai validate`
3. Expected:
   - Both commands fail with actionable messages identifying the missing or invalid
     configuration.
4. Restore the configuration file after the test.

## References

- Specification: `./spec.md`
- Plan: `./plan.md`
- Research: `./research.md`
- Data model: `./data-model.md`
- CLI contract: `./contracts/cli.md`
