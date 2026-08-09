# Data Model: v0.4.1 Runtime Chat Loop

## Entity: ChatMessage

- **role**: One of `system`, `user`, or `assistant`, matching Ollama's `/api/chat` message roles.
- **content**: The plain-text message body.

## Entity: ChatSession (in-memory only)

- **history**: Ordered list of `ChatMessage` for the current session, seeded with one `system` message from the Context Builder's `assembled_content`, followed by alternating `user`/`assistant` turns.
- **model**: The resolved model name used for every request in this session.

`ChatSession` is never written to disk — it exists only for the lifetime of one interactive session, consistent with the constitution's "temporary session state must remain temporary."

## Entity: Model Configuration (extends existing entities)

- `WorkspaceConfig.default_model` (new field, `workspace.toml`): the workspace-wide default model name, set on `ai init` and editable like other workspace settings.
- `ProjectMetadata.model` (new optional field, `project.toml`): a per-project override; when present, takes priority over `WorkspaceConfig.default_model` for that project's sessions.

## Relationships

- One active session produces one `ChatSession`.
- A `ChatSession` is seeded from exactly one `ContextSnapshot` (v0.4 Context Builder) via `ContextSnapshot::as_system_prompt()`.
- A `ChatSession`'s `model` is resolved from exactly one `ProjectMetadata` and one `WorkspaceConfig` (project override wins, else workspace default).

## Validation Rules

- The system message (context) is sent exactly once per session, as the first entry in `history`.
- A failed chat request does not corrupt `history`: the user's turn that triggered the failure is not silently duplicated on retry, and no assistant turn is appended for a failed request.
- `WorkspaceConfig.default_model` must always resolve to a non-empty string; existing `workspace.toml` files without the field deserialize with a built-in default rather than failing to load.
- `ProjectMetadata.model` is optional; existing `project.toml` files without the field deserialize with no override (falls back to workspace default) rather than failing to load.
