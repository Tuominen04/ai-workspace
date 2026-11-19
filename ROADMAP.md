# Roadmap

---

## v0.1

Foundation

Goal

Run one command.

Select workspace.

Open AI.

Features

- CLI
- Workspace discovery
- Start Ollama
- Start UI
- Shutdown

---

## v0.2

Workspace Manager

Features

- Workspace metadata
- Configuration
- Validation
- Project creation

---

## v0.3

Permissions

Features

- Temporary permissions
- Session permissions
- Permission viewer

---

## v0.4

Context Builder

Features

- Base loading
- Project loading
- Prompt assembly

---

## v0.5

Search

Filesystem search

Relevant document loading

---

## v0.6

Release packaging

Fedora `.rpm` built from a version tag.

Command renamed to `ai-workspace`.

The next user-interface step is a Tauri desktop app (ADR-0009), started
after the engine was split from the CLI (ADR-0008) and ahead of project
agents.

---

## v0.7

Desktop app

A Tauri desktop app on the core library (ADR-0008, ADR-0009): projects,
validation, search, chat, permissions, configuration, workspace folder.

It is the only thing released, as the `ai-workspace` package and command;
the command line stays in the repository as a developer tool (ADR-0010,
ADR-0011).

---

## v0.8

Project agents

A project's agent is an Ollama model, chosen per project; the model's own
instructions are kept (ADR-0012). The context is sized from the model's
window and long files are cut with a visible marker; names that look like
secrets are never read. A session note is offered when a chat ends and saved
only on request. Replies stream, a failed reply can be retried (ADR-0013),
and the model reads Markdown from Base, the whole project and active
temporary grants (ADR-0014).

---

## v0.9

Project management

Create (done in v0.7)

Delete (to the desktop Trash)

Archive and restore (`Projects/Archive/`)

Clone

Rename (folder and display name)

All from a project's right-click menu or Overview. The desktop app also gained tabs per
selection (a project opens on its chat) and an Archived view in the sidebar.

See ADR-0015 and `specs/008-project-management/`.

---

## v0.10

Documentation

Complete user documentation: a short README for someone who downloads a release (what it is,
Ollama, quick start) and a user guide (`docs/user-guide.md`) covering every feature, where
things are kept and what to do when something goes wrong. The vision text moves to
`docs/vision.md`.

---

## v1.0

Stable release

Stable architecture

Workspace manager

Permission model

Context Builder

Podman integration

Fedora-first experience

---

## v1.1

Runtime abstraction

Support multiple runtimes. Moved after the stable release (ADR-0015): Ollama is the
only runtime in use, so it is not urgent.
