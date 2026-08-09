# Contract: `ai search` Command

Standalone entry point for User Story 3 (see spec.md) — lets a user search accessible workspace content without starting a full session.

## Usage

```text
ai search --query TEXT [--project PROJECT] [--limit N]
```

| Flag | Required | Default | Notes |
| --- | --- | --- | --- |
| `--query TEXT` | yes | — | Freeform search text. Missing/empty → usage error, same style as `ai permission grant`'s missing `--path`. |
| `--project PROJECT` | no | none (Base-only scope) | Project name/id to include Knowledge + Session content from, in addition to Base. Unknown project → the same `ProjectNotFound`-style error other project-scoped commands already produce. |
| `--limit N` | no | `10` | Maximum results returned. Must be a positive integer (same parsing style as `--page`/`--page-size` in `ai permission view`). |

## Gating

Calls `enforce_validation_gate` before running, matching `ai project show` and all `ai permission *` commands — search depends on a structurally valid workspace/project to resolve `--project` and to know where Knowledge/Session content lives. Blocked (Error-severity) validation issues cause the command to refuse to run, with the same actionable error path already used elsewhere.

## Output (success, matches found)

Renders each `SearchResult` (see data-model.md) in relevance order:

```text
Search: "<query>"  (N result(s))

1. [project] Projects/<name>/Knowledge/setup.md   score: 4.2
   "...install Ollama and configure the local runtime before..."

2. [base] Base/glossary.md   score: 1.0
   "...runtime refers to the local model-serving process..."
```

Each entry shows: rank, `source_kind` tag, display path, relevance score, and the snippet. This is presentation logic and belongs entirely in `ui.rs` (per `CLAUDE.md`'s "UI never contains business logic" — `ui.rs` renders a `SearchOutcome` it's handed, it does not compute one).

## Output (success, no matches)

```text
Search: "<query>"  (0 results)
No documents matched. Try a different term, or check `ai permission view` if you expected a path to be included.
```

Distinct, non-error output — satisfies FR-004 (clearly report no matches rather than erroring).

## Errors

| Condition | Behavior |
| --- | --- |
| Missing/empty `--query` | Usage error printed to stderr, non-zero exit, before any workspace access (same pattern as other commands' missing required flags). |
| Unknown `--project` | `AppError::ProjectNotFound`-equivalent, non-zero exit. |
| Blocked validation gate | `AppError::ValidationGateBlocked`-equivalent, non-zero exit, same message style as `ai project show`/`ai permission *`. |
| Zero matches | **Not an error** — exit code `0`, "no results" message as above. |

## Non-goals for this contract

- No pagination flag beyond `--limit` (unlike `ai permission view`'s `--page`/`--page-size`) — search result sets are expected to be small (bounded by relevance, not by an exhaustive listing).
- No `--source` filter (base-only vs. project-only) beyond the presence/absence of `--project` — narrower scoping is a possible future addition, not required by the spec.
