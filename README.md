# AI Workspace

> **Vision:** A local-first AI workspace manager for Fedora that provides isolated, project-aware AI workspaces while keeping the user in complete control of data, permissions, and execution.

---

## Why this project exists

Current local AI solutions focus on running language models or providing a chat interface.

This project has a different goal.

The AI should not be "another chatbot".

Instead, it should behave like a knowledgeable assistant that understands a specific workspace, has access only to the information explicitly granted by the user, and helps with planning, learning, documenting and reasoning.

The AI never becomes the source of truth.

The filesystem is the source of truth.

---

## Core Principles

- Local-first
- Fedora-first
- User-controlled
- Project isolation
- Explicit permissions
- Simplicity over features
- Documentation before implementation
- Modular architecture
- Replaceable components

---

## Long-Term Vision

The final system should behave like this:

```text
User -> AI Workspace -> Select Workspace -> Context Builder -> LLM -> User Interface
```

The user launches a single application.

The application asks which workspace should be opened.

The system automatically:

- loads the Base knowledge
- loads the selected project knowledge
- verifies available resources
- applies temporary permissions
- starts the required services
- opens the preferred user interface

When the session ends, temporary permissions disappear automatically.

---

## Functional Goals

The completed system should support:

- Multiple isolated projects
- Shared Base knowledge
- Local LLM execution
- Temporary resource permissions
- Read-only project understanding
- Optional future write workflows
- Multiple simultaneous workspaces
- Future multi-agent support
- Replaceable UI
- Replaceable LLM backend

No cloud services are required.

No internet connection is required.

Everything should work completely offline.

---

## Workspace Model

```text
Base
    +
Selected Project
    +
Temporary Resources
```

Every workspace is built from these three layers.

Projects never have access to each other.

---

## Base

Base contains permanent knowledge shared across every project.

Examples:

- Fedora
- Linux
- Development
- Common principles
- User identity
- User preferences

Base should change rarely.

---

## Project

Every project owns its own knowledge.

Example:

```text
Weather/

Knowledge/
Specs/
Resources/
```

The project is responsible for its own documentation and history.

Projects must remain completely isolated.

---

## Resources

Projects may reference external resources.

Examples:

- Git repository
- Documentation
- PDFs
- API specifications

Resources are read-only by default.

---

## Session Permissions

Users may temporarily grant additional access.

Examples:

```text
~/Downloads/api.pdf

~/Projects/weather-app/docs

~/Projects/weather-app/src
```

These permissions only exist while the session is active.

After closing the workspace they disappear automatically.

---

## User Interaction Philosophy

The AI should:

- explain
- reason
- review
- teach
- plan
- document

The AI should not automatically:

- edit files
- execute commands
- modify Git repositories
- install software

Future versions may support these actions only after explicit user approval.

---

## Source of Truth

The source of truth is always the filesystem.

Never:

- internal databases
- hidden memory
- proprietary storage

Project knowledge lives inside project directories.

Base knowledge lives inside Base.

Everything should remain portable.

---

## Architecture Goals

Every major component must be replaceable.

Examples:

LLM

```text
Ollama
LM Studio
vLLM
future backend
```

User Interface

```text
CLI
TUI
GTK
Web UI
```

Search

```text
Simple file search
RAG
Vector database
```

Nothing outside the abstraction layer should depend on a specific implementation.

---

## Development Philosophy

The project should evolve in small, testable iterations.

Every feature should:

1. solve one problem
2. remain understandable
3. avoid unnecessary abstraction
4. avoid premature optimization

The architecture should support future growth without forcing early complexity.

---

## Project Phases

### Phase 1 - Foundation

Goal:

Create a working local workspace launcher.

Features:

- CLI
- Workspace selection
- Start Ollama
- Start UI
- Stop services
- Base knowledge
- Project knowledge

This phase intentionally avoids:

- RAG
- agents
- indexing
- automation
- editing files

---

### Phase 2 - Workspace Manager

Add:

- Project creation
- Resource management
- Session permissions
- Configuration
- Logging

---

### Phase 3 - Context System

Introduce:

- Intelligent document loading
- Context builder
- Search
- Efficient prompt construction

The AI should no longer load everything.

Instead it should load only what is relevant.

---

### Phase 4 - User Experience

Improve usability.

Possible interfaces:

- CLI
- TUI
- GTK desktop application

The backend should remain unchanged.

---

### Phase 5 - Advanced Features

Possible future additions:

- Multi-agent workflows
- Knowledge suggestions
- Automatic documentation drafts
- Git integration
- Code review
- MCP integrations
- Local automation

These are optional and should never complicate the earlier phases.

---

## Non-Goals

This project is not intended to become:

- another chat client
- another IDE
- another code editor
- another cloud AI service

Its responsibility is managing local AI workspaces.

---

## Migration Strategy

Every early design decision should support future migration.

Examples:

- CLI can later become a TUI.
- TUI can later become GTK.
- Open WebUI can later be replaced.
- Ollama can later be replaced.
- Search implementation can later evolve into RAG.

Core business logic should never depend on a specific frontend or AI provider.

---

## Success Criteria

The project is successful when a user can:

- launch one command
- select a workspace
- immediately begin working with a project-aware local AI
- know exactly what information the AI can access
- close the session knowing no unnecessary services remain running

The system should feel like a natural extension of Fedora rather than another web application.

---

## Install

AI Workspace is installed from a Fedora package as the `ai-workspace` command, which
opens the desktop app.

1. Download `ai-workspace-<version>.x86_64.rpm` and `SHA256SUMS` from the project's
   [releases](https://github.com/Tuominen04/ai-workspace/releases).
2. Optionally [verify the download](docs/release.md#verifying-a-download).
3. Install it (`dnf` pulls in WebKitGTK and GTK):

   ```bash
   sudo dnf install ./ai-workspace-<version>.x86_64.rpm
   ```

4. Open **AI Workspace** from the applications menu, or run `ai-workspace`.

The package installs the app, its menu entry and icon, and the two license
texts. It does not create, move, or delete a workspace, and removing it
(`sudo dnf remove ai-workspace`) leaves your workspace folder untouched.
Releases up to 0.6 installed a command-line `ai-workspace` instead; upgrading
replaces it with the app. See [docs/desktop.md](docs/desktop.md) for what the
app does.

### Choosing the workspace folder

AI Workspace keeps its workspace in `~/AI` unless you say otherwise. To use another folder, create it and name it in `~/.config/ai-workspace/config.toml` (or under `$XDG_CONFIG_HOME`), a small file you can write by hand:

```toml
workspace_root = "/home/you/ai-local/workspaces"
```

The folder is looked up in this order, and the app's sidebar and Configuration tab show which one was used:

1. the `AI_WORKSPACE_HOME` environment variable, if set (handy for testing against a scratch folder);
2. `workspace_root` in the config file above;
3. `~/AI`.

To go back to `~/AI`, delete the config file. Installing, upgrading or removing the package never touches it or your workspace. (From source, `cargo run -- config set-root PATH` writes the file for you.)

How a release is made is described in [docs/release.md](docs/release.md).

---

## Command line (from source)

The command-line front end is kept in the repository and tested, but no longer
released (ADR-0010). Run it from a checkout with `cargo run -- <args>`; the
commands below are written as the binary is named, `ai-workspace`. It can still
do what the app cannot yet: create projects, grant and revoke permissions, and
save the workspace folder.

## Workspace Manager Commands (v0.2)

The v0.2 workspace-manager layer adds explicit commands for configuration,
project creation, project inspection, and validation:

```text
ai-workspace config show
ai-workspace project create --name NAME [--description TEXT]
ai-workspace project show PROJECT
ai-workspace validate
```

Validation behavior is safety-first:

- `ai-workspace validate` reports actionable workspace and project issues.
- `ai-workspace project create` and `ai-workspace project show` are enforcement-gated and refuse to
    continue while blocking validation issues exist.

## Permission Commands (v0.3)

The v0.3 permission layer adds explicit commands for temporary session grants,
permission inspection, and manual revoke:

```text
ai-workspace permission grant [--project PROJECT] --path PATH
ai-workspace permission view [--project PROJECT] [--filter TEXT] [--page N] [--page-size N]
ai-workspace permission revoke [--project PROJECT] --path PATH
```

Permission behavior is explicit and session-scoped:

- grants are idempotent by canonical path
- invalid paths are denied with actionable errors
- unreachable granted paths remain visible as unavailable
- temporary grants are removed automatically at session end

## License

Dual licensed under [MIT](LICENSE-MIT) or [Apache 2.0](LICENSE-APACHE), at your option.
