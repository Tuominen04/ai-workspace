# Technology Stack

## Philosophy

Every technology must satisfy at least one of these goals:

- Native Fedora integration
- Long-term maintainability
- Replaceability
- Minimal dependencies
- Local-first
- Open source

Technology should never be selected because it is trendy.

---

## Programming Language

### Rust

Reason

- Native binary
- Excellent error handling
- Memory safety
- Strong ecosystem
- Excellent system programming support
- GTK support
- TUI support
- TOML ecosystem
- Async support when needed

Rust is expected to remain the primary language for the project.

---

## Runtime

Current

- Ollama

Future

- LM Studio
- vLLM
- llama.cpp
- Remote runtimes (optional)

Runtime must be abstracted behind a common interface.

---

## Containers

Container Runtime

- Podman

Reason

- Fedora native
- Rootless support
- Quadlet integration
- No daemon
- Excellent systemd integration

Docker is intentionally not a project dependency.

---

## Service Management

systemd User Services

Quadlet

Reason

- Native Fedora
- Reliable lifecycle management
- User-level services
- Automatic cleanup

---

## Configuration

TOML

Reason

- Human readable
- Strong Rust ecosystem
- Excellent for configuration
- Version control friendly

---

## Knowledge Format

Markdown

Reason

- Human readable
- Git friendly
- AI friendly
- Portable

---

## Logging

Structured logging

Future support

JSON logs

Human-readable logs

Debug mode

---

## UI Evolution

Phase 1

CLI

↓

Phase 2

TUI

↓

Phase 3

GTK

Backend should remain unchanged.
