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

---

## ADR-0006

Release packaging, the `local-ai` command name, and the license. A release
is a Fedora `.rpm` built by GitHub Actions when a `v<version>` tag is pushed
and attached, with a `SHA256SUMS` file, to a draft release the maintainer
publishes by hand. The package is built with `cargo-generate-rpm` from
metadata in `Cargo.toml` and is installed in a Fedora container before the
draft is created. The command is renamed from `ai` to `local-ai`, and every
message that names it is built from one `COMMAND_NAME` constant in `lib.rs`.
The project is licensed `MIT OR Apache-2.0`.

Reason

The installed command had been a hand-copied build that fell months behind
the code. A tag-triggered package makes every version installable with
`dnf`, the same way the maintainer's other project is released. Build
automation in the cloud was explicitly requested and is build-time only;
the installed tool stays offline. `cargo-generate-rpm` needs no `.spec`
file or Fedora build container and keeps the version in one place. `ai` is
too generic a name for a system-wide command; a single constant keeps the
name from being scattered across string literals again. A package must
declare a license, and the dual license matches the maintainer's other
project (v0.6, `specs/007-release-packaging/`).

Status

Accepted
