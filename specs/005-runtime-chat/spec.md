# Feature Specification: v0.4.1 Runtime Chat Loop

**Feature Branch**: `[005-runtime-chat]`

**Created**: 2026-08-05

**Status**: Draft

**Input**: User description: "wire the v0.4 context snapshot into an actual Ollama chat call so a session lets the user talk with a project-aware local AI, instead of assembling context that is never sent anywhere"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Ask a question grounded in workspace context (Priority: P1)

As a workspace user, I can start a session, ask a question, and receive an answer from the local AI that is grounded in my workspace's base and project knowledge, instead of only seeing a static summary of which files were loaded.

**Why this priority**: This is the core value of v0.4.1 because it is the first point where the AI actually responds to the user with assembled context, closing the gap between v0.4's context assembly and the project's stated goal of "immediately begin working with a project-aware local AI."

**Independent Test**: Can be fully tested by starting a session in a workspace with base and project knowledge, asking a question that only makes sense with that knowledge, and confirming the answer reflects it.

**Acceptance Scenarios**:

1. **Given** a workspace with base and project knowledge and Ollama running, **When** the user starts a session and asks a question, **Then** the AI responds using the assembled context.
2. **Given** the AI has answered once, **When** the user asks a follow-up question in the same session, **Then** the AI's response can take the earlier exchange into account.
3. **Given** the user is done asking questions, **When** the user types `exit` or `quit` (or closes input), **Then** the session ends and temporary permissions are revoked as before.

---

### User Story 2 - Continue working when the AI is unavailable (Priority: P2)

As a workspace user, I can still open and close a session normally when Ollama is not installed or not reachable, without the session crashing or leaving me stuck.

**Why this priority**: Local-first tooling must degrade gracefully; a user without Ollama installed, or with it temporarily down, must not lose the rest of the session experience that already works today.

**Independent Test**: Can be fully tested by starting a session with Ollama stopped/not installed and confirming the session still starts, shows the existing context summary, and closes cleanly.

**Acceptance Scenarios**:

1. **Given** Ollama is not installed, **When** the user starts a session, **Then** the session behaves as it did before this feature (summary shown, session closes on Enter) with no chat prompt offered.
2. **Given** Ollama is running when the session starts but becomes unreachable mid-session, **When** the user asks a question, **Then** the session reports the failure clearly and lets the user retry or exit, rather than terminating the session.

---

### User Story 3 - Know which model is answering (Priority: P3)

As a workspace user, I can rely on a sensible default model for chatting, and override it per project when a project needs a different model, without editing code.

**Why this priority**: Different projects may warrant different local models (size/speed/capability tradeoffs); this should be configuration, not a hardcoded choice.

**Independent Test**: Can be fully tested by setting a workspace-level default model, then overriding it for one project, and confirming each project's session uses the expected model.

**Acceptance Scenarios**:

1. **Given** no project-level override is set, **When** a session starts, **Then** the workspace's configured default model is used.
2. **Given** a project sets its own model override, **When** a session starts for that project, **Then** the project's override is used instead of the workspace default.

### Edge Cases

- The user presses Enter with no text at the chat prompt (should re-prompt, not exit or send an empty message).
- Ollama returns a malformed or unexpected response body.
- The configured model name does not exist in the local Ollama installation.
- The user ends input via EOF (e.g. Ctrl+D) instead of typing `exit`/`quit`.
- A very long conversation grows the message history; the session must remain usable for typical single-session workspace conversations.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST send the assembled context (from the v0.4 Context Builder) to the AI once per session, before the first user question is answered.
- **FR-002**: The system MUST let the user ask multiple questions in one session, carrying prior turns forward so follow-up questions can be understood in context.
- **FR-003**: The system MUST let the user end the chat explicitly (`exit`/`quit`) or via end-of-input, after which the session closes the same way it does today (temporary permissions revoked, confirmation shown).
- **FR-004**: The system MUST fall back to the pre-existing "press Enter to close" session behavior when the local AI runtime is not running or not installed, without offering a chat prompt.
- **FR-005**: The system MUST report a chat failure (network error, bad response) to the user without ending the session, so the user can retry or exit deliberately.
- **FR-006**: The system MUST resolve which model to use for a session from a per-project override when set, otherwise a workspace-wide default.
- **FR-007**: The system MUST NOT require the user to configure a model before the first `ai init` — a workspace-wide default must exist out of the box.
- **FR-008**: The system MUST NOT send anything to the AI runtime other than the assembled context and the conversation the user is having in the current session (no unrelated filesystem access, no data leaving the local machine).

### Key Entities *(include if feature involves data)*

- **Chat Turn**: One role-tagged message (system context, user question, or AI answer) contributing to the session's conversation.
- **Chat Session**: The ordered sequence of Chat Turns for one open workspace session, seeded with the assembled context and growing as the user asks questions.
- **Model Selection**: The resolved model name for a session, derived from project-level configuration if present, otherwise the workspace default.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A user starting a session with Ollama running can ask a question and receive a response grounded in workspace context without any manual setup beyond `ai init` and having Ollama running.
- **SC-002**: A user can ask at least one follow-up question in the same session and receive a coherent response that does not require repeating earlier context.
- **SC-003**: 100% of sessions started without Ollama available still start and close successfully, matching today's behavior.
- **SC-004**: A single failed chat request does not end the session; the user can continue the session after a failure.

## Assumptions

- Ollama, when used, runs locally on the default port already managed by the existing Runtime Manager (`127.0.0.1:11434`).
- The user has already pulled at least one model locally if they want to use chat; this feature does not manage model downloads.
- v0.4.1 targets a single local AI runtime response per user turn (no partial/streaming display) — streaming output is a possible future refinement, not required now.
- The chat loop is a CLI (stdin/stdout) experience, consistent with the project's current CLI-only interface; a future TUI/GTK interface can reuse the same underlying orchestration.
