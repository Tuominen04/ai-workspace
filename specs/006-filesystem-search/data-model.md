# Phase 1 Data Model: v0.5 Filesystem Search

All entities below are in-memory/request-scoped Rust types — none are persisted to disk. The filesystem (Markdown files under `Base/`, `Projects/<name>/Knowledge/`, `Projects/<name>/Session/`) remains the sole source of truth; search recomputes results fresh on every call.

## SearchQuery

Represents the user's search request.

| Field | Type | Notes |
| --- | --- | --- |
| `text` | `String` | Freeform query text; split into whitespace-separated terms for scoring. Never empty (rejected before reaching `search.rs`). |
| `project` | `Option<Project>` | The active project to include Knowledge/Session content from, if any. `None` scopes search to Base only. |
| `limit` | `usize` | Maximum results to return. Default `10` for the standalone command; equal to `DEFAULT_MAX_CONTEXT_SOURCES` (8) when driven by the Context Builder. |

**Validation rules**: `text` must be non-empty after trimming (FR-004's "no matches" is a *result*, not a validation failure — an empty query is a *usage* error, consistent with how `ai permission grant` rejects a missing `--path`).

## SearchCandidate

An internal, pre-permission-filtered handle to a searchable file, produced by the caller (`session.rs`) from existing `workspace.rs` APIs before being passed into `search.rs`. Not a public-facing type.

| Field | Type | Notes |
| --- | --- | --- |
| `source_kind` | `SourceKind` | `Base` \| `Project` \| `Session` — mirrors the existing `"base"`/`"project"`/`"session"` tags already used in `context.rs`. |
| `path` | `PathBuf` | Canonical/resolved path to the Markdown file. Already known to be within a Default or Temporary permission grant. |

## SearchResult

A single ranked match, returned to both the standalone command and the Context Builder.

| Field | Type | Notes |
| --- | --- | --- |
| `source_kind` | `SourceKind` | Where the match came from (Base/Project/Session) — surfaced to the user per FR-005 and reused as the Context Builder's existing priority tie-breaker. |
| `path` | `PathBuf` | File location, for display (`ai search` output) and for reuse as a Context Builder source. |
| `relevance_score` | `f64` | Non-negative; higher is more relevant. `0.0` never appears in results (files scoring zero are matches of nothing and are excluded, satisfying "no results" per FR-004). |
| `snippet` | `String` | Bounded (~160 char) plain-text excerpt showing why the file matched (Decision 6 in research.md). |

**Relationships**: `SearchResult` is produced from a `SearchCandidate` plus a `SearchQuery`; a `Vec<SearchResult>` sorted by `relevance_score` descending (ties broken by `source_kind` priority, then path) is the complete output of one search.

## SearchOutcome

The top-level return value of running a search — what both the CLI renderer and the Context Builder consume.

| Field | Type | Notes |
| --- | --- | --- |
| `query_text` | `String` | Echoed back for display. |
| `results` | `Vec<SearchResult>` | Empty when nothing matched — the "no results" case (FR-004), rendered distinctly from an error. |

## ContextSnapshot (modified)

Existing type in `context.rs`; unchanged in shape, but how its fields are populated changes:

| Field | Type | Change |
| --- | --- | --- |
| `base_files`, `project_files`, `session_files` | `Vec<PathBuf>` | Now populated from relevance-ranked `SearchResult`s when a query is supplied, instead of always being the first N in fixed priority order. |
| `assembled_content`, `source_count`, `total_bytes` | as before | Unchanged in meaning; still bounded by `DEFAULT_MAX_CONTEXT_SOURCES`/`DEFAULT_MAX_CONTEXT_BYTES`. |

**New input**: `build_context_snapshot(paths, project, query: Option<&str>)` — see research.md Decision 4. `query = None` reproduces v0.4 behavior exactly; `query = Some(text)` runs the same selection through `search.rs` ranking first.

## State / Lifecycle

Search has no state machine — every call is a pure function of (current filesystem content, current permission grants, query) to `SearchOutcome`. There is no caching, no persisted index, and no result set that outlives a single command invocation or chat turn, consistent with the constitution's Filesystem First principle already governing this project (filesystem remains sole source of truth; nothing here becomes hidden state).
