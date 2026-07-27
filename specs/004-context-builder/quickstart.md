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

1. Ensure the workspace is initialized and a project exists:
   - `./target/debug/ai init`
   - `./target/debug/ai project create --name demo-context`
2. Start a session for that project:
   - `./target/debug/ai`
3. Observe the startup summary and look for the context section.
4. Expected: base knowledge files are listed and the summary clearly shows shared base context.

### Scenario 2: Project context appears for the selected project

1. Create or select the project and add a markdown file under its Knowledge directory:
   - `printf '# Demo notes\n\nProject-specific context.\n' > ~/AI/Projects/demo-context/Knowledge/notes.md`
   - `ls ~/AI/Projects/demo-context`
2. Start a session for that project:
   - `./target/debug/ai`
3. Expected: the startup summary includes the project knowledge file and clearly shows project context in addition to the shared base context.

### Scenario 3: Switching projects changes the context

1. Create or select two projects, for example `project-a` and `project-b`.
   - `./target/debug/ai project create --name project-a`
   - `./target/debug/ai project create --name project-b`
2. Add distinct markdown content to each project's Knowledge directory.
   - `printf '# Demo notes\n\nProject A -specific context.\n' > ~/AI/Projects/project-a/Knowledge/notes-a.md`
   - `printf '# Demo notes\n\nProject B -specific context.\n' > ~/AI/Projects/project-b/Knowledge/notes-b.md`
3. Start a session for project A:
   - `./target/debug/ai`
4. Start a new session for project B:
   - `./target/debug/ai`
5. Expected: the context summary reflects project B rather than project A.

### Scenario 4: Context stays bounded

1. Add 12 markdown files to a project's Knowledge directory so the total content exceeds the planned size limit.
   - `for i in $(seq 1 12); do printf '# Note %s\n\nThis is test content.\n' "$i" > ~/AI/Projects/demo-context/Knowledge/note-$i.md; done`
2. Start a session for that project:
   - `./target/debug/ai`
3. Expected: the assembled context remains bounded and deterministic, with only the selected subset of sources included.

## Verification Commands

- `cargo test`
- `./target/debug/ai validate`
- `./target/debug/ai`
