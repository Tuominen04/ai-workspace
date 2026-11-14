# ARCHITECTURE.md

> This document defines the technical architecture of AI Workspace.
>
> It intentionally focuses on responsibilities and interfaces instead of implementation details.
> Technologies may change over time, but the architecture should remain stable.

---

## Architecture Philosophy

The system should consist of small, independent components.

Each component should have a single responsibility.

Every major component must be replaceable without requiring changes to the rest of the application.

Dependencies should point inward.

Business logic must never depend on a specific frontend, LLM or search implementation.

---

## High-Level Architecture

```txt
                User
                  │
                  ▼
        ┌─────────────────┐
        │ User Interface  │
        └─────────────────┘
                  │
                  ▼
        ┌─────────────────┐
        │ Workspace Core  │
        └─────────────────┘
                  │
      ┌───────────┼────────────┐
      ▼           ▼            ▼
 Project     Context Builder   Runtime
 Manager          │            Manager
      │           ▼            │
      │      Search Engine     │
      │           │            │
      ▼           ▼            ▼
Filesystem      Knowledge      Ollama
```

---

## Code Layout

The layers below are split across two crates in one Cargo workspace (ADR-0008):

```txt
crates/ai-workspace-core     Layers 2–6 (library ai_workspace_core)
    engine.rs                Workspace Core: Engine, one method per workspace operation
    session.rs               Workspace Core: Session, one interactive project session
    paths.rs, config.rs      Configuration and filesystem layout (platform lookups in paths.rs)
    workspace.rs             Project Manager, validation, permission sessions
    context.rs               Context Builder
    search.rs                Search Engine
    runtime.rs               Runtime Manager
    error.rs                 AppError, shared by every layer

crates/ai-workspace-cli      Layer 1 (binary ai-workspace)
    app.rs                   Argument parsing, usage text, command dispatch
    ui.rs                    Rendering and terminal prompts
```

The core returns data and never prints, reads the terminal, or names the command; a front end
renders what it gets back. Cargo prevents the core from depending on a front end.

---

## Layer Mapping

The project is divided into logical layers.

---

### Layer 1 — User Interface

Responsibility

* User interaction
* Menus
* Session display
* Progress information

Possible implementations

* CLI
* TUI
* GTK
* Web UI

The UI should never contain business logic.

Implemented by `crates/ai-workspace-cli`. A second front end (for example a desktop app)
would be another crate that depends on the core.

---

### Layer 2 — Workspace Core

This is the heart of the application.

Responsibilities

* Workspace lifecycle
* Session lifecycle
* Context assembly
* Permission validation
* Configuration loading

Everything flows through this layer.

Implemented by `Engine` and `Session` in `crates/ai-workspace-core`. Both return data to the
front end; neither prints nor prompts.

---

### Layer 3 — Project Manager

Responsibilities

* Discover projects
* Create projects
* Load Base
* Load Project metadata
* Load Resources
* Validate directory structure

This layer knows the filesystem.

Nothing else should.

---

### Layer 4 — Context Builder

Responsibilities

* Determine which knowledge is relevant
* Assemble prompts
* Merge Base and Project knowledge
* Include temporary resources

The Context Builder decides **what** the AI sees.

The LLM never reads files directly.

---

### Layer 5 — Search Engine

Initial implementation

Simple filesystem search.

Future implementations

* RAG
* Embeddings
* Vector database

The rest of the application should not know which implementation is used.

---

### Layer 6 — Runtime Manager

Responsibilities

* Start Ollama
* Stop Ollama
* Start UI
* Stop UI
* Health checks
* Runtime status
* Send chat requests to the LLM

Later it may support multiple runtimes.

Examples

* Ollama
* LM Studio
* vLLM

---

## Workspace Lifecycle

```txt
Launch

↓

Select Workspace

↓

Load Configuration

↓

Load Base

↓

Load Project

↓

Load Resources

↓

Build Context

↓

Ensure Runtime

↓

Launch User Interface

↓

Interactive Session

↓

Shutdown

↓

Cleanup
```

---

## Filesystem Layout

```txt
~/AI/

Base/

Projects/

Cache/

Logs/

Config/
```

Everything user-created belongs here.

No hidden application state should exist elsewhere unless required by external software.

---

## Base Layout

```txt
Base/

Common/

Identity/

Linux/

Fedora/

Development/
```

Shared knowledge.

Loaded into every workspace.

---

## Project Layout

```txt
Project/

Knowledge/

Specs/

Resources/

Session/

README.md
```

Projects are isolated.

Projects never access each other.

---

## Session Model

A session contains

* Workspace
* Runtime
* Active permissions
* Temporary resources

A session never modifies permanent knowledge automatically.

---

## Permission Model

Permissions are explicit.

Default permissions

```txt
Base/

Selected Project/
```

Temporary permissions

```txt
Downloads/

Git repository/

PDFs/

External documentation/
```

Temporary permissions disappear when the session ends.

---

## Runtime Abstraction

Workspace Core should never know how an LLM works.

Instead it communicates with a Runtime interface.

Example

```txt
Workspace Core

↓

Runtime

↓

Ollama
```

This carries both control traffic (start, stop, health checks) and chat
messages: Workspace Core hands Runtime a resolved model name and the current
conversation, and Runtime is responsible for however that reaches the LLM
(HTTP call, local socket, or otherwise).

A future runtime should be replaceable without changing Workspace Core.

---

## Search Abstraction

Search should be replaceable.

Phase 1

Filesystem lookup

↓

Phase 2

Document search

↓

Phase 3

RAG

↓

Phase 4

Hybrid search

No other component should depend on the implementation.

---

## Configuration

Configuration should be human-readable.

Preferred format

```txt
TOML
```

Examples

```txt
config.toml

workspace.toml

runtime.toml
```

Configuration should never require recompiling the application.

---

## Logging

Logging should be structured.

Separate logs for

* Workspace
* Runtime
* Search
* Errors

Future versions may support debug logging.

---

## Error Handling

Errors should be descriptive.

The application should prefer

Explain

↓

Suggest

↓

Recover

Instead of immediately terminating whenever possible.

---

## Security

Principles

* Local-first
* Rootless by default
* Least privilege
* Explicit permissions
* No automatic write access
* No hidden background services

Every permission should be visible to the user.

---

## Technology Roadmap

The architecture intentionally avoids binding to a specific technology too early.

Current direction

Language

* Rust

Runtime

* Podman
* Quadlet
* Ollama

Configuration

* TOML

Knowledge

* Markdown

Platform

* Fedora Workstation

Future technology choices should support—not redefine—the architecture.

---

## Architectural Rules

The following rules should never be broken.

1. Filesystem is the source of truth.
2. Projects are isolated.
3. Base is shared.
4. Business logic never depends on UI.
5. Business logic never depends on a specific LLM.
6. Search implementation is replaceable.
7. Every component should have one responsibility.
8. New features should extend the architecture, not replace it.
9. The system should remain understandable after years of development.
10. Simplicity is preferred over cleverness.
