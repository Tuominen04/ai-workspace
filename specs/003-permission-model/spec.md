# Feature Specification: v0.3 Permission Model

**Feature Branch**: `[003-permission-model]`

**Created**: 2026-07-22

**Status**: Draft

**Input**: User description: "v0.3 permissions scope from roadmap and README"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Grant Temporary Access (Priority: P1)

As a workspace user, I can grant temporary access to one or more external paths for the current session so the AI can use only the resources I explicitly allow.

**Why this priority**: Explicit, temporary permission grants are the core safety and usability capability of v0.3.

**Independent Test**: Can be fully tested by starting a session, granting a valid external path, and confirming the granted path appears as active only for that session.

**Acceptance Scenarios**:

1. **Given** an active workspace session, **When** the user grants a valid external path, **Then** the path is added to active session permissions.
2. **Given** an active workspace session, **When** the user grants multiple valid external paths, **Then** each path is recorded as a separate active session permission.
3. **Given** an active workspace session, **When** the user attempts to grant a path that does not exist, **Then** the request is rejected with a clear corrective message.

---

### User Story 2 - Inspect Active Permissions (Priority: P2)

As a workspace user, I can view current permissions so I always know exactly what data the AI is allowed to access.

**Why this priority**: Visibility is required for trust, auditability, and safe operation in a local-first environment.

**Independent Test**: Can be fully tested by granting at least one path and verifying the permission viewer shows default permissions and temporary session permissions with clear status.

**Acceptance Scenarios**:

1. **Given** an active session with no temporary permissions, **When** the user opens the permission viewer, **Then** the viewer shows default permissions and an empty temporary-permission state.
2. **Given** an active session with temporary permissions, **When** the user opens the permission viewer, **Then** the viewer lists all active temporary permissions.
3. **Given** a permission has expired after session end, **When** the user starts a new session and opens the viewer, **Then** the expired permission is not shown as active.

---

### User Story 3 - Automatic Permission Expiry (Priority: P3)

As a workspace user, I can end a session knowing temporary permissions are removed automatically and not carried into later sessions.

**Why this priority**: Automatic cleanup protects privacy and enforces least-privilege behavior without extra user actions.

**Independent Test**: Can be fully tested by granting temporary permissions, ending the session, starting a new session, and confirming prior temporary permissions are absent.

**Acceptance Scenarios**:

1. **Given** a session with active temporary permissions, **When** the session ends, **Then** those temporary permissions are revoked automatically.
2. **Given** temporary permissions were granted in a previous session, **When** a new session starts, **Then** none of the previous session's temporary permissions are active.

### Edge Cases

- What happens when the same path is granted more than once in a session?
- How does the system handle a granted path that becomes unavailable during an active session?
- What happens when a user attempts to grant access outside allowed local filesystem boundaries?
- How does the permission viewer behave when the active permission set is very large?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST maintain two permission categories for each session: default permissions and temporary session permissions.
- **FR-002**: The system MUST include Base and the selected project in default permissions for every active session.
- **FR-003**: The system MUST allow users to grant temporary access to explicit filesystem paths for the current session.
- **FR-004**: The system MUST validate that each requested temporary path is local and exists before granting access.
- **FR-005**: The system MUST reject invalid permission requests and provide an actionable error message describing why the request was blocked.
- **FR-006**: The system MUST provide a permission viewer that displays all active permissions and clearly distinguishes default versus temporary grants.
- **FR-007**: The system MUST reflect permission state changes in the viewer within the same user interaction flow.
- **FR-008**: The system MUST revoke all temporary session permissions automatically when the session ends.
- **FR-009**: The system MUST start each new session with no temporary permissions carried over from prior sessions.
- **FR-010**: The system MUST preserve project isolation by preventing permissions granted for one project session from becoming active in another project session.
- **FR-011**: The system MUST present permission information in user-readable terms that make current AI access scope explicit.

### Key Entities *(include if feature involves data)*

- **Permission Grant**: A record of allowed access with attributes for path, permission category (default or temporary), grant time, and active state.
- **Session Permission Set**: The complete set of active permissions for one session, including immutable defaults and revocable temporary grants.
- **Permission View**: A user-facing representation of active permissions grouped by category and current validity status.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 95% of users can grant a valid temporary permission in under 30 seconds during usability validation.
- **SC-002**: 100% of ended sessions remove all temporary permissions before the next session begins.
- **SC-003**: 100% of invalid permission requests are rejected with a clear reason and suggested correction.
- **SC-004**: 95% of users can correctly identify current AI access scope from the permission viewer on first attempt.
- **SC-005**: 100% of session handoffs between different projects preserve project isolation with no cross-project temporary permission leakage.

## Assumptions

- Permission grants are scoped to local filesystem paths and do not include remote or cloud resources in v0.3.
- Default permissions remain limited to Base and the selected project for the active session.
- Write access behavior remains unchanged from earlier phases; this feature focuses on visibility and control of access scope.
- Permission management is initiated by explicit user action and never auto-expands scope without a user request.
