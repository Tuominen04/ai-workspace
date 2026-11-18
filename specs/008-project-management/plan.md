# Implementation Plan: Project Management

**Branch**: `008-project-management` | **Date**: 2026-11-18 | **Spec**: [spec.md](./spec.md)

## Summary

Archive/restore, clone and delete projects from the desktop app (ADR-0015). Each is one core function in `workspace.rs`, one `Engine` method, one thin Tauri command and the buttons in the interface. They are built as three pull requests, in the spec's priority order, and released together as 0.9.0.

## Technical Context

- **Language**: Rust (core, desktop shell), TypeScript/React (interface). No change to the CLI crate.
- **New dependency**: `trash` (crates.io) in the core, for delete only. On Linux it implements the freedesktop.org Trash specification in Rust: it moves the folder into `~/.local/share/Trash/files` (or a filesystem's `.Trash-<uid>`) and writes the matching `.trashinfo`. Its default `freedesktop` feature pulls in only `libc`, `scopeguard` and `urlencoding`: no network, no D-Bus (`zbus` comes only with the `portal` feature, which stays off) and no background process.
- **Storage**: the filesystem only, as always. `Projects/Archive/` holds archived projects.
- **Testing**: core integration tests on temporary workspaces (`project_management_tests.rs`), desktop shell tests, browser checks with the stub engine. Delete's tests point the Trash at a temporary home (`XDG_DATA_HOME`), so they never touch a real Trash.

## Design

### Core (`workspace.rs`, `engine.rs`, `error.rs`)

- `ARCHIVE_DIR = "Archive"`. `discover_projects` skips it; `discover_archived_projects` lists its subfolders.
- `normalize_project_name` reserves `archive`.
- `archive_project(paths, name)` and `restore_project(paths, name)` use `fs::rename`, within the same filesystem, so the move is atomic. The destination is checked first.
- `clone_project(paths, config, name)` copies recursively into `<folder>-clone`. It skips `Session/permissions.toml`, rewrites `project.toml` (`project_id`, `display_name`, `created_at`), and refuses symlinks rather than following them. A failed copy removes the partial clone.
- `delete_project(paths, name, archived)` calls `trash::delete`.
- `AppError::AlreadyExists(PathBuf)`, displayed as "Already exists: <path>".
- `Engine::archive_project`, `restore_project`, `clone_project` (gated), `delete_project`, `archived_projects`.

### Desktop

- Commands: `project_archive`, `project_restore`, `project_clone`, `project_delete`, `projects_archived`. Archive, restore and delete close an open chat session in that project first, as `workspace_set_root` already does.
- Sidebar: an **Archived** list at the bottom, above Theme, collapsed when long. An archived project shows a read-only Overview with **Restore** and **Delete**.
- A project's Overview: **Archive**, **Clone** and **Delete**. Delete asks for confirmation, naming the project and saying it goes to the Trash.

## Pull requests

1. **Archive and restore**: core, the Archived sidebar list, and the reserved name.
2. **Clone**.
3. **Delete to Trash**: the `trash` dependency and the confirmation.
4. **Version 0.9.0**: after the maintainer has tested 1–3.

## Constitution check

- **Local-first**: no network. `trash` writes only into the local Trash.
- **No automatic writes**: every operation is a button press, and delete is also confirmed.
- **Filesystem as the source of truth**: archive state is where the folder is, with no index file.
- **Filesystem access stays in `workspace.rs`**.
