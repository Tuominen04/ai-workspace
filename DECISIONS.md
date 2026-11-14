# Architecture Decisions

## ADR-0001

Filesystem is the source of truth.

Reason

Everything remains portable.

Status

Accepted

---

## ADR-0002

Rust is the primary language.

Reason

Long-term maintainability.

Status

Accepted

---

## ADR-0003

Podman instead of Docker.

Reason

Fedora native.

Status

Accepted

---

## ADR-0004

Runtime Manager sends chat requests to Ollama via a blocking HTTP client
(`ureq`), calling `/api/chat` with `stream: false`.

Reason

The codebase is fully synchronous today; a single localhost HTTP call does
not justify introducing an async runtime. Non-streaming keeps response
handling to one JSON object per turn instead of incremental NDJSON parsing.

Status

Accepted

---

## ADR-0005

Search-driven context selection. The Context Builder ranks Base/Project/Session
Markdown by relevance instead of a fixed priority order whenever a query is
available: the project's own description at session start, the user's
message on every chat turn thereafter. Ranking uses simple, dependency-free
keyword scoring in a new `search.rs` module; it never drops a source that
would otherwise fit under the existing size caps, so a small workspace still
gets everything.

Reason

As workspace documentation grows past the Context Builder's fixed
source/byte caps, a fixed priority order stops being "relevant" and starts
being arbitrary. Search relevance is what makes the selection actually
track the current task (v0.5, `specs/006-filesystem-search/`).

Status

Accepted
