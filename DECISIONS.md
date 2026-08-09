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
