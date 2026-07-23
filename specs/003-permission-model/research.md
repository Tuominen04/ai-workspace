# Research: v0.3 Permission Model

## Decision 1: Keep permission authority local and filesystem-scoped

- Decision: Restrict temporary permission grants to explicit local path grants and
  reject non-local or inaccessible paths.
- Rationale: This enforces least privilege, preserves project isolation, and aligns
  with the local-first constitution and permission constraints.
- Alternatives considered:
  - Allow any local path: Rejected due to excessive default scope.
  - Allow remote or network paths: Rejected for v0.3 due to local-first and offline
    design goals.

## Decision 2: Make duplicate grants idempotent on canonical path

- Decision: Normalize and canonicalize grant paths, then maintain exactly one active
  grant record per canonical path.
- Rationale: Idempotent behavior avoids noisy duplicate entries, makes user intent
  predictable, and simplifies validation and viewer rendering.
- Alternatives considered:
  - Reject duplicates as errors: Rejected because repeat grant actions are common and
    should not require error handling.
  - Keep duplicates: Rejected because it harms usability and complicates status
    semantics.

## Decision 3: Preserve unavailable grants as visible state

- Decision: When a granted path becomes unreachable, keep the grant record visible,
  mark it unavailable, and block access attempts until reachability returns.
- Rationale: This gives transparent permission state, avoids silent mutation, and
  preserves auditability while remaining safe.
- Alternatives considered:
  - Auto-remove unavailable grants: Rejected because users lose context unexpectedly.
  - Keep access allowed while unavailable: Rejected because it creates ambiguous and
    potentially unsafe behavior.

## Decision 4: Scale the permission viewer with deterministic paging and filtering

- Decision: Provide deterministic sorting with paging and path search/filter in the
  permission viewer.
- Rationale: Large permission sets remain navigable and reproducible for humans and
  scripts, matching requirement FR-016 and FR-017.
- Alternatives considered:
  - Render full list always: Rejected because it does not scale and degrades UX.
  - Truncate list without navigation: Rejected because it hides active scope.

## Decision 5: Capture permission lifecycle events in local structured logs

- Decision: Emit local structured log events for grant, deny, revoke, and
  unavailable-status transitions.
- Rationale: Provides traceability and operational visibility without introducing
  cloud dependencies.
- Alternatives considered:
  - Minimal fatal-only logging: Rejected due to insufficient audit visibility.
  - No logging: Rejected because it weakens explainability and supportability.

## Decision 6: Model temporary permissions as session-scoped state with explicit cleanup

- Decision: Keep temporary permission state bound to the active session lifecycle and
  ensure automatic revocation on session termination.
- Rationale: This is the core safety behavior for v0.3 and directly satisfies
  FR-008/FR-009 while honoring the temporary-permission principle.
- Alternatives considered:
  - Persist temporary grants across sessions: Rejected as a direct violation of
    temporary-access expectations.
  - Manual-only cleanup: Rejected because it increases risk of stale over-privileged
    state.
