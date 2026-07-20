# Feature Specification: AI CLI Foundation v0.1

**Feature Branch**: `001-ai-cli-foundation`

**Created**: 2026-07-20

**Status**: Draft

**Input**: User description: "Build first version v0.1 of AI Workspace runnable from CLI ai with workspace selection, context loading, runtime start/stop, and session shutdown"

## User Scenarios & Testing *(mandatory)*

<!--
  IMPORTANT: User stories should be PRIORITIZED as user journeys ordered by importance.
  Each user story/journey must be INDEPENDENTLY TESTABLE - meaning if you implement just ONE of them,
  you should still have a viable MVP (Minimum Viable Product) that delivers value.

  Assign priorities (P1, P2, P3, etc.) to each story, where P1 is the most critical.
  Think of each story as a standalone slice of functionality that can be:
  - Developed independently
  - Tested independently
  - Deployed independently
  - Demonstrated to users independently
-->

### User Story 1 - Start Local Workspace Session (Priority: P1)

As a user, I can run one command (`ai`) and start a local session by selecting a
workspace project.

**Why this priority**: This is the core value of v0.1 and the minimum usable product.

**Independent Test**: From a terminal with prepared workspace folders, run `ai`, select
a listed workspace, and verify the session starts and can be closed cleanly.

**Acceptance Scenarios**:

1. **Given** the workspace root exists with at least one project, **When** I run `ai`,
  **Then** I see available projects and can select one.
2. **Given** one project is selected, **When** session startup completes, **Then** I get
  a visible active-session state and can close it intentionally.

---

### User Story 2 - Enforce Base-Only AI Context (Priority: P2)

As a user, I can confirm that the AI context is restricted to shared Base knowledge
only.

**Why this priority**: Strict read boundaries are required by the project safety model.

**Independent Test**: Run `ai` with known Markdown files in Base and non-Base folders,
then verify that only Base files are listed as AI-readable context.

**Acceptance Scenarios**:

1. **Given** documentation files exist in Base and non-Base locations, **When** I start a session,
  **Then** the session summary reports only Base files as AI-readable context.
2. **Given** no matching Base files exist, **When** I start a session, **Then** the
  session still starts and clearly reports zero loaded Base context files.

---

### User Story 3 - Runtime Lifecycle and Visibility (Priority: P3)

As a user, I can run a session that checks runtime availability, starts it when
possible, and reports runtime state clearly.

**Why this priority**: v0.1 requires runtime readiness but must remain transparent and
recoverable if runtime is unavailable.

**Independent Test**: Start with runtime running and not running; in both cases verify
clear runtime status messaging and clean shutdown behavior.

**Acceptance Scenarios**:

1. **Given** runtime is already active, **When** I start `ai`, **Then** the session uses
  existing runtime and reports that state.
2. **Given** runtime is not active but can be started, **When** I start `ai`, **Then**
  runtime startup is attempted and status is shown.
3. **Given** runtime is unavailable, **When** I start `ai`, **Then** the session does not
  crash and reports that runtime could not be started.

---

### Edge Cases

- No projects exist under the workspace root.
- User provides an invalid workspace selection repeatedly.
- The command runs where the home directory cannot be resolved.
- Runtime binary is not installed.
- Runtime start is initiated but readiness is not reached in time.
- Base or project directories exist but contain no Markdown files.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST provide a CLI entry command `ai` that starts an
  interactive workspace session.
- **FR-002**: The system MUST initialize and validate the standard workspace directory
  layout before starting a session.
- **FR-003**: The system MUST discover available project workspaces from the configured
  projects directory.
- **FR-004**: The system MUST present discovered projects and allow the user to choose
  one for the active session.
- **FR-005**: The system MUST fail with a clear message when no projects are available.
- **FR-006**: The system MUST load AI-readable context inputs from shared Base knowledge
  only.
- **FR-007**: The system MUST NOT include selected project files in AI-readable context.
- **FR-008**: The system MUST display what Base context was loaded for the current
  session.
- **FR-009**: The system MUST check runtime availability for the session and report
  whether runtime was reused, started, or unavailable.
- **FR-010**: The system MUST support a clean session shutdown initiated by the user.
- **FR-011**: The system MUST ensure temporary session runtime processes started by the
  command are terminated when the session ends.
- **FR-012**: The system MUST provide a non-interactive status command that reports
  configured root paths and discovered projects.
- **FR-013**: The system MUST support an initialization command that creates the default
  workspace directory structure when it does not yet exist.

### Key Entities *(include if feature involves data)*

- **Workspace Root**: User-owned root directory containing Base, Projects, Cache, Logs,
  and Config.
- **Project Workspace**: Isolated project folder within Projects used as the selected
  session context.
- **Context Snapshot**: List of discovered Base knowledge files at session start.
- **Runtime Status**: Session-visible state describing runtime availability and startup
  behavior.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A first-time user can initialize the workspace root and start a session
  with one project selection in under 3 minutes.
- **SC-002**: In 95% of local runs with valid workspace inputs, the command reaches a
  usable session state in under 10 seconds.
- **SC-003**: In 100% of runs without available projects, the command exits with a
  clear, actionable error message.
- **SC-004**: In 100% of user-initiated exits, the session shutdown returns terminal
  control without leaving runtime processes started by the session.

## Assumptions

- Target users run on Fedora-compatible local environments.
- v0.1 scope is terminal CLI only; TUI and GUI are out of scope.
- Session context is read-only; automated write workflows are out of scope.
- AI-readable context is limited to `~/AI/Base/`.
- Runtime default is local and optional for startup continuity; missing runtime should
  degrade gracefully.
- Project discovery is folder-based within the workspace root project directory.
