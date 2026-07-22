# Data Model: v0.3 Permission Model

## Entity: PermissionGrant

- Description: One explicit permission entry describing AI access scope for a
  canonical filesystem path within a session.
- Fields:
  - grantId: stable identifier
  - sessionId: session identifier
  - canonicalPath: normalized absolute path
  - displayPath: user-visible path representation
  - category: `default` or `temporary`
  - status: `active` or `unavailable`
  - grantedAt: timestamp
  - source: `system-default` or `user-grant`
- Validation rules:
  - `canonicalPath` must resolve to a local path under approved local roots.
  - Only one `PermissionGrant` may exist per `sessionId + canonicalPath` pair.
  - `default` grants are immutable during session runtime.
  - `temporary` grants may transition between active and unavailable.
- State transitions:
  - requested -> active
  - requested -> denied
  - active -> unavailable
  - unavailable -> active
  - active -> revoked
  - unavailable -> revoked

## Entity: SessionPermissionSet

- Description: Aggregate permission state for one active workspace session.
- Fields:
  - sessionId: session identifier
  - workspaceId: selected workspace/project reference
  - defaultGrants: list of `PermissionGrant`
  - temporaryGrants: list of `PermissionGrant`
  - createdAt: timestamp
  - terminatedAt: optional timestamp
- Validation rules:
  - Default grants must include Base and selected project scope.
  - Temporary grants cannot outlive the session.
  - Grants from one project session must not appear in another project session.
- State transitions:
  - initialized -> active
  - active -> terminated

## Entity: PermissionViewSnapshot

- Description: User-facing view model for active permission inspection.
- Fields:
  - sessionId: session identifier
  - sortKey: deterministic sort descriptor
  - page: page index
  - pageSize: page size
  - filter: optional path filter term
  - totalCount: full result count
  - entries: current page of permission entries
- Validation rules:
  - View must include category and status for each entry.
  - Sorting must be deterministic for identical datasets.
  - Paging metadata must be consistent with `totalCount`.

## Entity: PermissionEvent

- Description: Structured local log event for permission lifecycle behavior.
- Fields:
  - eventId: stable event identifier
  - sessionId: session identifier
  - eventType: `grant`, `deny`, `revoke`, `status-change`
  - path: canonical path
  - category: default or temporary
  - previousStatus: optional status
  - newStatus: optional status
  - reasonCode: optional reason for deny or status change
  - occurredAt: timestamp
- Validation rules:
  - Every grant/deny/revoke/status-change action emits one corresponding event.
  - Events must contain enough context for human troubleshooting.
  - Events are stored locally only and do not depend on external services.
