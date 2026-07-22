# CLI Contract: AI Workspace v0.3 Permission Model

## Command Surface

### ai permission grant --path PATH

- Purpose: Grant temporary session access to one local filesystem path.
- Behavior:
  - Resolves and canonicalizes the requested path.
  - Validates that path exists and is within approved local roots.
  - Applies idempotent behavior: duplicate canonical path grants keep one active entry.
  - Emits a local structured event for grant or deny outcomes.
- Exit behavior:
  - Exit 0 when the grant is accepted (including idempotent duplicate acceptance).
  - Non-zero when path validation or boundary checks fail.

### ai permission view [--filter TEXT] [--page N] [--page-size N]

- Purpose: Display active session permissions to the user.
- Behavior:
  - Lists default and temporary permission entries with status.
  - Uses deterministic sorting.
  - Supports path filtering and paged output for large result sets.
  - Includes unavailable grants in output.
- Exit behavior:
  - Exit 0 on successful rendering of the requested view.
  - Non-zero if session context cannot be resolved.

### ai permission revoke --path PATH

- Purpose: Revoke a temporary permission during the active session.
- Behavior:
  - Resolves requested path to canonical form.
  - Removes matching temporary grant from active state.
  - Leaves default grants unchanged.
  - Emits a local structured revoke event.
- Exit behavior:
  - Exit 0 on successful revocation.
  - Non-zero if matching temporary grant is not found or session context is invalid.

## Session Lifecycle Contract

- Temporary grants are session-scoped and are removed automatically at session end.
- New sessions start with default grants only.
- Grants from one project session cannot become active in another project session.

## Permission Validation Contract

- Allowed grant scope is limited to approved local roots.
- Non-existent paths are denied with actionable messaging.
- Paths that become unreachable transition to unavailable status and are blocked for
  access attempts until reachable again.

## Logging Contract

- The system writes local structured log events for:
  - grant
  - deny
  - revoke
  - unavailable-path status changes
- Logging is local-first and does not require network connectivity.

## Exit Code Contract

- `0`: command succeeded and permission state is valid for the operation.
- Non-zero: validation failure, out-of-bound request, missing session context,
  unsupported permission transition, or filesystem access error.
