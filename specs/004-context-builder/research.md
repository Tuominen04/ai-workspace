# Research: v0.4 Context Builder

## Decision 1: Use the existing workspace layout as the source of truth

- Decision: Assemble context from the shared base knowledge area and the selected project's knowledge area using the existing `Base` and `Projects/<project>` layout.
- Rationale: This matches the repository's filesystem-first architecture and avoids introducing hidden state or external services.
- Alternatives considered: Loading all files from the workspace root (rejected because it creates prompt noise) and introducing a database or vector store (rejected for v0.4 scope).

## Decision 2: Keep prompt assembly inside the context layer

- Decision: Extend the existing context module so it owns selection, ordering, and summarization of context sources rather than moving that logic into UI or runtime code.
- Rationale: This preserves the layered architecture and keeps the logic reusable for future interfaces such as TUI or GTK.
- Alternatives considered: Building prompt strings directly in the session flow (rejected because it would mix responsibilities) and pushing the behavior into the runtime layer (rejected because the runtime should remain abstracted).

## Decision 3: Keep v0.4 bounded and deterministic

- Decision: Limit the number of included source files and total bytes, and order them deterministically by path so the results remain predictable.
- Rationale: A bounded prompt is easier to reason about, easier to test, and better aligned with the project's local-first goals.
- Alternatives considered: Unbounded loading (rejected because it can overwhelm the model) and ranking heuristics based on recency or size (deferred because they are beyond the scope of v0.4).
