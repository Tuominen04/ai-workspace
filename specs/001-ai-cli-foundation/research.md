# Research: AI CLI Foundation v0.1

## Decision 1: Build a single local CLI binary in Rust
- Decision: Use a single Rust CLI binary named ai as the v0.1 entrypoint.
- Rationale: The technology direction sets Rust as primary for maintainability and native execution on Fedora. A single binary minimizes operational overhead and supports local-first use.
- Alternatives considered:
  - Bash-only launcher scripts: Rejected due to weaker long-term structure and testing ergonomics.
  - Python CLI: Rejected because it diverges from the project's stated primary language.

## Decision 2: Keep AI-readable context restricted to Base only
- Decision: Limit AI-readable context to ~/AI/Base and do not include selected project files in AI context.
- Rationale: This enforces strict read boundaries and user-controlled visibility while still allowing workspace selection and session flow.
- Alternatives considered:
  - Load Base plus selected project context: Rejected by current policy requirement.
  - Full repository indexing: Rejected for v0.1 due to safety and scope.

## Decision 3: Use filesystem layout initialization at startup
- Decision: Ensure the expected root directories exist before interactive session startup.
- Rationale: v0.1 requires predictable local behavior and explicit filesystem-backed state.
- Alternatives considered:
  - Fail fast when folders are missing: Rejected due to poorer first-run experience.
  - Hidden internal DB for state bootstrap: Rejected by filesystem-first constitution.

## Decision 4: Runtime lifecycle based on local Ollama presence
- Decision: Check for existing local Ollama runtime, start when available, degrade gracefully when missing.
- Rationale: Keeps runtime local and transparent, supports practical first-run behavior, and aligns with runtime abstraction goals.
- Alternatives considered:
  - Hard fail if Ollama missing: Rejected because v0.1 should remain usable enough to show workspace and context state.
  - Integrate multiple runtimes in v0.1: Rejected as out of scope for this phase.

## Decision 5: Keep architecture as layered modules from day one
- Decision: Separate entrypoint, session orchestration, workspace discovery, context builder, runtime manager, and UI output.
- Rationale: Preserves replaceability and avoids business logic leaking into terminal presentation.
- Alternatives considered:
  - Single file prototype: Rejected because it would violate layering and reduce migration quality.

## Decision 6: Prefer std-only dependencies for v0.1
- Decision: Keep external dependencies at zero for this first version.
- Rationale: Minimal dependencies and local reliability are explicit stack goals.
- Alternatives considered:
  - Add CLI parser/logging crates immediately: Deferred to post-v0.1 once baseline behavior is validated.
