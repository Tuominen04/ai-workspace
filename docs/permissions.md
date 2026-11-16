# Permissions

## Scope Model

Permission scope is explicit and session-bound.

- Default scope:
  - `Base/`
  - selected project directory
- Temporary scope:
  - explicit local filesystem paths granted with `ai-workspace permission grant`

Temporary permissions are removed when the session ends.

## Commands

Grant access:

```text
ai-workspace permission grant [--project PROJECT] --path PATH
```

View effective access scope:

```text
ai-workspace permission view [--project PROJECT] [--filter TEXT] [--page N] [--page-size N]
```

Revoke temporary access:

```text
ai-workspace permission revoke [--project PROJECT] --path PATH
```

If `--project` is omitted and more than one project exists, the command returns a
project-context error.

## Behavior

- Grants are idempotent on canonical path.
- Non-existent or invalid paths are denied with an actionable error.
- Unreachable granted paths remain visible with `unavailable` status.
- Access checks against unavailable paths are blocked until the path is reachable.
- Permission view output is deterministic and supports filtering and pagination.

## Session Artifact and Logs

- Active session permission state is stored in:
  - `Projects/<project>/Session/permissions.toml`
- Structured permission events are appended to:
  - `Logs/permissions-events.log`

Event types:

- `grant`
- `deny`
- `revoke`
- `status-change`

## What a grant lets the model read (ADR-0014)

The default grants (Base and the selected project) and the session's active
temporary grants are exactly what a chat's context and search read: Markdown in
all of Base and the whole project folder, plus a granted Markdown file or the
Markdown files in a granted folder (two levels deep). Names that look like
secrets are never read. A grant made during a chat counts from the next
message; revoking it, or ending the session, removes it from the context.
