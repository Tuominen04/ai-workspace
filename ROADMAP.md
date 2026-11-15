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

Runtime abstraction

Support multiple runtimes.

---

## v0.9

Project management

Create (done in v0.7)

Delete

Archive

Clone

---

## v0.10

Documentation

Complete user documentation.

---

## v1.0

Stable release

Stable architecture

Workspace manager

Permission model

Context Builder

Podman integration

Fedora-first experience
