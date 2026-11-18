# Feature Specification: Project Management

**Feature Branch**: `[008-project-management]`

**Created**: 2026-11-18

**Status**: Draft

**Input**: Maintainer: "Deleting: does Fedora support an auto-cleanup (like 30 days) trash can? That would be great; if not, just hard delete. Archive: `/path/to/Projects/Archive/`. Clone: `<Cloned Project> (Clone)`, copy the old project's sessions too. Already exists: block with a notice. Archived projects at the bottom of the sidebar. Desktop only."

The roadmap's v0.9 entry ("Project management: Create, Delete, Archive, Clone"). Creating projects shipped in v0.7. This spec covers the other three. Decisions are recorded in ADR-0015.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Archive and restore a project (Priority: P1)

The maintainer has finished with a project but wants to keep it. They archive it from the desktop app, and it leaves the project list without being lost. Later they restore it.

**Why this priority**: The safest of the three operations, since nothing is lost, and the most common one.

**Independent Test**: Archive a project, check that its folder now sits in `Projects/Archive/` and that it appears only in the sidebar's Archived list. Restore it and check that it is back where it was, with all files intact.

**Acceptance Scenarios**:

1. **Given** a project `security`, **When** the maintainer archives it, **Then** its folder moves, whole and unchanged, from `Projects/security` to `Projects/Archive/security`.
2. **Given** an archived project, **When** the app lists projects, **Then** it is not among the projects. It is listed under **Archived**, at the bottom of the sidebar.
3. **Given** an archived project is selected, **When** the maintainer looks at it, **Then** they see a read-only overview with **Restore** and **Delete**. There is no chat, search or permissions for it.
4. **Given** an archived project, **When** the maintainer restores it, **Then** its folder moves back to `Projects/<name>` and it is a normal project again.
5. **Given** `Projects/Archive/security` already exists, **When** the maintainer archives another project named `security`, **Then** nothing moves and the app says "Already exists". Restoring is refused the same way when `Projects/<name>` exists.
6. **Given** a chat session is open in the project, **When** it is archived, **Then** the session is ended first, which revokes its temporary grants.

---

### User Story 2 - Clone a project (Priority: P2)

The maintainer wants to start new work from an existing project, keeping its knowledge, specs, resources and past sessions.

**Why this priority**: Useful, and it writes only new files. It never changes the original.

**Independent Test**: Clone a project. Check that the clone has every file of the original, including its session notes, under a new folder with a "(Clone)" name, and that the original is unchanged.

**Acceptance Scenarios**:

1. **Given** a project whose display name is `Security` (folder `security`), **When** the maintainer clones it, **Then** a new project is created with display name `Security (Clone)` and folder `security-clone`.
2. **Given** the clone, **When** its files are compared with the original's, **Then** everything is copied, including `Session/` and its notes, except the original's permission session state (`Session/permissions.toml`). The clone starts with no temporary grants.
3. **Given** the clone, **When** its `project.toml` is read, **Then** it has its own project id and creation time, and keeps the original's description and model.
4. **Given** `Projects/security-clone` already exists, **When** the maintainer clones `security`, **Then** nothing is written and the app says "Already exists".
5. **Given** a clone was made, **When** the app lists projects, **Then** the clone is listed and selected, opening on its chat.

---

### User Story 3 - Delete a project (Priority: P3)

The maintainer no longer needs a project at all and deletes it, with a way back if it was a mistake.

**Why this priority**: The only operation that removes something, so it comes last and is guarded.

**Independent Test**: Delete a project, confirm it is gone from the workspace and present in the desktop Trash, where the Files app can restore it.

**Acceptance Scenarios**:

1. **Given** a project or an archived project, **When** the maintainer chooses **Delete**, **Then** the app first asks for confirmation, naming the project. Nothing happens until they confirm.
2. **Given** the maintainer confirms, **When** the project is deleted, **Then** its folder is moved to the user's Trash (the freedesktop.org Trash that GNOME Files uses), not erased. It can be restored from there.
3. **Given** GNOME's "Automatically Delete Trash Content" setting is on, **When** the configured age passes (30 days by default), **Then** the desktop removes it from the Trash. The app itself never empties the Trash.
4. **Given** the folder cannot be moved to the Trash, **When** deletion is attempted, **Then** nothing is deleted yet. A second warning says why and offers to erase it permanently; only confirming that erases the folder, and Cancel keeps it.
5. **Given** a chat session is open in the project, **When** it is deleted, **Then** the session is ended first.

---

### User Story 4 - Rename a project (Priority: P4)

The maintainer wants a project's name to match what it has become.

**Why this priority**: Added after the first three; a convenience that changes no content.

**Independent Test**: Rename a project, then check that its folder and display name both changed, its files are intact, and the old folder is gone.

**Acceptance Scenarios**:

1. **Given** a project `security`, **When** the maintainer renames it to "Home Security", **Then** its folder moves to `Projects/home-security`, its display name becomes "Home Security", its project id follows the folder, and every other file is unchanged.
2. **Given** `Projects/home-security` exists, **When** the maintainer renames another project to "Home Security", **Then** nothing changes and the app says "Already exists".
3. **Given** a new name that normalizes to the same folder (such as "SECURITY" for `security`), **When** it is saved, **Then** only the display name changes.
4. **Given** a name that is not allowed (empty, reserved such as "Archive", or with characters other than letters, digits, spaces, `-` and `_`), **When** it is saved, **Then** nothing changes and the app says why.
5. **Given** the project has temporary grants or an open chat session, **When** it is renamed, **Then** the session is ended and the temporary grants are revoked and logged first.
6. **Given** an archived project, **When** the maintainer looks at it, **Then** Rename is not offered.

---

### Edge Cases

- A project folder named `Archive` would collide with the archive folder. `archive` becomes a reserved project name, like `base` or `config`.
- An existing workspace may already have a project folder named `Archive`. It is then treated as the archive folder: its subfolders are listed as archived projects. The release notes mention this.
- A deleted or archived project was selected: the app selects the whole workspace afterwards.
- The archive folder is created when the first project is archived. It is never created otherwise.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Project discovery MUST skip `Projects/Archive/`. A separate listing MUST return the projects inside it.
- **FR-002**: Archiving MUST move the project folder unchanged to `Projects/Archive/<folder>`. Restoring MUST move it back. Both MUST refuse, writing nothing, when the destination exists.
- **FR-003**: Cloning MUST copy the project folder to `<folder>-clone` with display name `<display name> (Clone)`, a new project id and creation time, and without `Session/permissions.toml`. It MUST refuse, writing nothing, when the destination exists.
- **FR-004**: Deleting MUST move the folder to the user's Trash, following the freedesktop.org Trash specification. When the Trash cannot take it, it MUST erase the folder only after a second, separate confirmation that says it cannot be undone.
- **FR-005**: Every refusal because a destination exists MUST be one typed error whose message reads "Already exists: <path>".
- **FR-006**: Archive, restore and delete MUST end an open chat session in that project first.
- **FR-007**: Clone MUST run the validation gate, since it reads and writes `project.toml`. Archive, restore and delete move whole folders without reading them and MUST NOT be gated, so a project with broken metadata can still be put away or removed.
- **FR-008**: All filesystem work MUST stay in the core's `workspace.rs` (per CLAUDE.md). The desktop calls one `Engine` method per operation.
- **FR-009**: The desktop app MUST offer these operations: **Archive**, **Clone** and **Delete** on a project's Overview; **Restore** and **Delete** on an archived project. The command line is unchanged.
- **FR-010**: Nothing MUST be deleted, moved or written without an explicit action by the person. Delete also requires a confirmation.
- **FR-011**: Renaming MUST move the folder to the normalized new name and set the display name, refusing a taken folder with "Already exists". It MUST end the project's permission session first (temporary grants revoked and logged) and run the validation gate. Archived projects MUST NOT be renamed.

### Key Entities

- **Archived project**: a project folder under `Projects/Archive/`. It keeps its `project.toml` and all files, and is listed, restored or deleted, never chatted with.

## Success Criteria *(mandatory)*

- **SC-001**: Archiving then restoring a project leaves every file byte-identical.
- **SC-002**: A clone contains every file of the original except `Session/permissions.toml`, and the original is unchanged.
- **SC-003**: A project deleted to the Trash can be restored with the Files app's Trash.
- **SC-004**: No operation overwrites an existing folder.

## Assumptions

- Fedora Workstation (GNOME). Auto-emptying the Trash is a GNOME setting (Settings → Privacy & Security → File History & Trash), off by default. Other desktops have their own equivalents.
- A Trash is available for the workspace's filesystem: the home Trash, or a `.Trash-<uid>` folder at the top of another mounted filesystem, as the specification allows.
