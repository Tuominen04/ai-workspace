# Quickstart: v0.4 Context Builder

## Prerequisites

- Fedora-compatible Linux environment.
- Rust toolchain installed (`cargo`, `rustc`).
- Repository checkout with the current v0.4 spec and plan artifacts.

## Setup

1. Build the binary:
   - `cargo build`
2. Initialize the workspace if needed:
   - `./target/debug/ai init`
3. Create or select a project:
   - `./target/debug/ai project create --name demo-context`

## Validation Scenarios

### Scenario 1: Base context appears by default

1. Start a session for a project.
2. Observe the startup summary.
3. Expected: base knowledge files are listed and the summary clearly shows shared base context.

### Scenario 2: Project context appears for the selected project

1. Create a project with project-specific markdown files.
2. Start a session for that project.
3. Expected: the context summary includes the project's knowledge sources.

### Scenario 3: Switching projects changes the context

1. Start a session for project A.
2. Start a new session for project B.
3. Expected: the context summary reflects project B rather than project A.

### Scenario 4: Context stays bounded

1. Add enough knowledge files to exceed the planned size limit.
2. Start a session.
3. Expected: the assembled context remains bounded and deterministic.

## Verification Commands

- `cargo test`
- `./target/debug/ai validate`
- `./target/debug/ai`
