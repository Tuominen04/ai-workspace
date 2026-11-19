# Feature Specification: Stable Release (1.0)

**Feature Branch**: `[009-stable-release]`

**Created**: 2026-11-19

**Status**: Draft

**Input**: Maintainer's answers for v1.0: Ollama can run in Podman; "stable" means the workspace format is frozen, a real-Ollama test pass is done and no known bugs remain; Fedora-first needs nothing beyond confirming the RPM on the current Fedora; the developer command line stays as it is. Podman details: chosen by a setting, the image is downloaded by the person (never by the app), CPU by default with an optional GPU setting, and a container the app started is stopped when the app closes. Decisions are recorded in ADR-0016.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Run Ollama in Podman (Priority: P1)

The maintainer prefers containers to installing software on the host. They download the Ollama image once with Podman, set the workspace to use it, and chat as before; the app starts the container when a session needs it and stops it when the app closes.

**Why this priority**: The only new capability in 1.0; everything else is a promise or a check.

**Independent Test**: With no `ollama` command installed, pull the Ollama image, set `runtime = "podman"`, start a chat session, get a reply, close the app, and see that the container is gone.

**Acceptance Scenarios**:

1. **Given** `runtime = "podman"` and the image downloaded, **When** a chat session starts and nothing answers on `127.0.0.1:11434`, **Then** the app starts the Ollama container, published only on `127.0.0.1:11434`, with models kept in a named volume, and the session works as with the installed `ollama`.
2. **Given** something already answers on `127.0.0.1:11434` (an `ollama` you started, or a container you started yourself), **When** a session starts, **Then** the app uses it and starts nothing, whatever the setting.
3. **Given** the image is not downloaded, **When** a session starts, **Then** the app downloads nothing; the chat says the image is missing and shows the one `podman pull` command to run.
4. **Given** Podman is not installed, **When** a session starts with `runtime = "podman"`, **Then** the chat says so, as it does today when `ollama` is missing.
5. **Given** the app started the container, **When** the session ends or the app closes, **Then** the container is stopped and removed; the volume, and so the downloaded models, stay. A container the app did not start is never stopped.
6. **Given** a GPU setting of `nvidia` or `amd`, **When** the container starts, **Then** it gets that card (NVIDIA through CDI, AMD through the ROCm image and its devices); the default, `none`, runs on the CPU.
7. **Given** the Podman runtime, **When** the maintainer looks at Configuration, **Then** it shows which runtime is set, the image and the GPU setting.
8. **Given** models are pulled into the container with `podman exec`, **When** a project's model is changed on its Overview, **Then** the installed models are suggested as with the installed `ollama`.

---

### User Story 2 - A workspace that later versions keep reading (Priority: P2)

The maintainer's notes will outlive many versions. Everything 1.0 writes must still be read, unchanged, by every 1.x.

**Why this priority**: The core of "stable": the filesystem is the source of truth, so its format is the contract.

**Independent Test**: A workspace written by 1.0, kept as a test fixture, loads and validates in every later build; the format is written down.

**Acceptance Scenarios**:

1. **Given** the documented 1.0 format (folder layout, `project.toml`, `workspace.toml`, the user settings file, `Session/permissions.toml`, session note names, `Projects/Archive/`), **When** a later 1.x reads a 1.0 workspace, **Then** it loads without change and validates clean.
2. **Given** a 1.x adds a setting, **When** an older 1.x reads a file with that key, **Then** it ignores the unknown key and keeps working.
3. **Given** a future change that cannot stay compatible, **Then** it waits for 2.0, or comes with a migration the person starts and that keeps a copy.

---

### User Story 3 - Released only after a real test pass and with no known bugs (Priority: P3)

Before tagging 1.0 the maintainer runs a written checklist on Fedora with real Ollama and a real model, covering every feature and both runtimes; any bug found is fixed first.

**Why this priority**: It decides when 1.0 is tagged, not what it does.

**Independent Test**: The checklist exists, a filled-in run against the release candidate is attached to the release PR, and no issue labelled `bug` is open.

**Acceptance Scenarios**:

1. **Given** the checklist, **When** it is run on the current Fedora release with the RPM built from the candidate, **Then** every step passes for both the installed `ollama` and the Podman runtime.
2. **Given** a bug is found, by the checklist or otherwise, **Then** it is filed as an issue labelled `bug` and fixed before 1.0 is tagged.

### Edge Cases

- Port 11434 taken by something that is not Ollama: the session reports that Ollama did not answer; the app does not guess.
- A container named `ai-workspace-ollama` already exists but is stopped (for example after a crash): the app removes that stopped container first. It only ever removes the container with its own name, and only when that container is not running.
- AMD needs the `rocm` image: with `gpu = "amd"` and no image set, the default image is the ROCm tag.
- SELinux on Fedora: the volume is a named Podman volume, not a host folder, so no relabelling is needed.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: `workspace.toml` MUST accept `runtime = "ollama" | "podman"` (default `"ollama"`) and a `[podman]` table with `image` and `gpu = "none" | "nvidia" | "amd"` (default `"none"`); files without them MUST behave as today.
- **FR-002**: If `127.0.0.1:11434` answers, the app MUST use it and start nothing.
- **FR-003**: With the Podman runtime the app MUST start the container with `--pull=never`, publish only on `127.0.0.1:11434`, keep models in the named volume `ai-workspace-ollama`, name the container `ai-workspace-ollama`, and stop and remove it when it started it and the session ends or the app closes.
- **FR-004**: The app MUST NOT download images or models. A missing image MUST be reported with the `podman pull` command.
- **FR-005**: All process work MUST stay in `runtime.rs`; the setting is read in `config.rs`; no file access moves elsewhere.
- **FR-006**: The 1.0 workspace format MUST be documented, and a 1.0 workspace fixture MUST load and validate in the test suite.
- **FR-007**: Reading MUST ignore unknown keys in every TOML file the app reads.
- **FR-008**: A release checklist for real Ollama on Fedora, covering both runtimes and every user-visible feature, MUST exist and be run before tagging 1.0.
- **FR-009**: 1.0 MUST NOT be tagged while an issue labelled `bug` is open.
- **FR-010**: The developer command line MUST keep working and stay unreleased (ADR-0010).

## Success Criteria *(mandatory)*

- **SC-001**: A chat works with only Podman and the Ollama image installed, and no `ollama` command.
- **SC-002**: After the app closes, `podman ps -a` shows no container it started.
- **SC-003**: The 1.0 fixture workspace loads and validates in every later build.
- **SC-004**: A completed checklist run is attached to the 1.0 release PR, with no open `bug` issues.

## Assumptions

- Podman 4 or newer, as on current Fedora; rootless.
- NVIDIA users have the NVIDIA Container Toolkit's CDI specification set up; AMD users have ROCm-capable hardware. The guide links to Ollama's and Podman's own instructions.
- Runtime abstraction (several runtimes, ADR-0015) stays in v1.1: the Podman runtime is still Ollama, only started differently.
