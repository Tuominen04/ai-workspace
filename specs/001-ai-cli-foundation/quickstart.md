# Quickstart Validation: AI CLI Foundation v0.1

## Prerequisites
- Fedora-compatible Linux environment.
- Rust toolchain installed (cargo, rustc).
- Optional: Ollama installed for runtime-start validation.

## Setup
1. Build binary:
   - cargo build
2. Ensure workspace root exists:
   - ./target/debug/ai init
3. Create at least one project folder:
   - mkdir -p ~/AI/Projects/DemoProject
4. Add Base knowledge files:
   - mkdir -p ~/AI/Base/Common
   - echo "Base note" > ~/AI/Base/Common/intro.md

## Validation Scenarios

### Scenario 1: Non-interactive status command
1. Run:
   - ./target/debug/ai status
2. Expected:
   - Prints root paths and discovered projects.
   - Includes DemoProject in project list.

### Scenario 2: Interactive startup and clean shutdown
1. Run:
   - ./target/debug/ai
2. Select DemoProject when prompted.
3. Press Enter to close session.
4. Expected:
   - Session summary shown.
   - Command returns to shell after Enter.

### Scenario 3: Base-only context enforcement
1. Add non-Base markdown file:
   - echo "Project note" > ~/AI/Projects/DemoProject/notes.md
2. Run:
   - ./target/debug/ai
3. Expected:
   - Session output lists Base markdown files.
   - Project markdown file is not listed as AI-readable context.

### Scenario 4: No projects edge case
1. Temporarily move projects directory:
   - mv ~/AI/Projects ~/AI/Projects.bak
   - mkdir -p ~/AI/Projects
2. Run:
   - ./target/debug/ai
3. Expected:
   - Clear error or guidance that no workspaces were found.
4. Restore:
   - rm -rf ~/AI/Projects
   - mv ~/AI/Projects.bak ~/AI/Projects

### Scenario 5: Runtime visibility behavior
1. With Ollama running, start ai and verify reuse message.
2. With Ollama stopped and available, start ai and verify startup-attempt message.
3. Without Ollama installed, start ai and verify degraded runtime message without panic.

## References
- Specification: ./spec.md
- Plan: ./plan.md
- Data model: ./data-model.md
- CLI contract: ./contracts/cli.md
