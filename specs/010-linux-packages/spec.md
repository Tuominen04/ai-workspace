# Feature Specification: Linux Packages (1.1)

**Feature Branch**: `[010-linux-packages]`

**Created**: 2026-11-24

**Status**: Draft

**Input**: The maintainer wants AI Workspace on more systems, starting with Linux beyond Fedora; Windows and macOS come later, each with its own spec. The PR should say which virtual machines the maintainer installs and what to test on each. Decisions are recorded in ADR-0017.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Install on Ubuntu or Debian (Priority: P1)

Someone on Ubuntu, Debian or Linux Mint downloads one `.deb` from the release page, installs it with `apt`, and finds **AI Workspace** in the applications menu, as on Fedora.

**Why this priority**: Ubuntu and its derivatives are the largest group of Linux desktop users; a native package installs, upgrades and uninstalls the way they expect.

**Independent Test**: On a fresh Ubuntu 24.04 LTS, `sudo apt install ./ai-workspace_<version>_amd64.deb`, open it from the menu, initialize a workspace, create a project and chat with a model in Ollama.

**Acceptance Scenarios**:

1. **Given** Ubuntu 24.04 LTS or later, Debian 13 or later, or Linux Mint 22 or later on x86_64, **When** the `.deb` is installed with `apt`, **Then** `apt` pulls in the libraries it needs (WebKitGTK 4.1 and GTK 3), the menu lists **AI Workspace**, and `ai-workspace` opens it.
2. **Given** an older release is installed from a `.deb`, **When** a newer `.deb` is installed, **Then** it replaces the older one and the workspace is read as before.
3. **Given** the package is installed, **When** it is removed (`sudo apt remove ai-workspace`), **Then** the workspace folder and the per-user settings stay untouched.
4. **Given** an older system (Ubuntu 22.04, Debian 12), **When** someone tries the `.deb`, **Then** `apt` refuses it for its library versions rather than installing something that does not start; the README lists the supported versions.

---

### User Story 2 - Run it on any other Linux with an AppImage (Priority: P2)

Someone on a distribution with neither a `.deb` nor the Fedora `.rpm` (openSUSE, Arch, …) downloads one AppImage, makes it executable and runs it, without installing anything system-wide.

**Why this priority**: Reaches every other x86_64 distribution with one file; less integrated than a package (no menu entry by itself, updates by downloading again).

**Independent Test**: On openSUSE Tumbleweed, `chmod +x ai-workspace-<version>-x86_64.AppImage && ./ai-workspace-<version>-x86_64.AppImage`, then initialize, create a project and chat.

**Acceptance Scenarios**:

1. **Given** an x86_64 distribution with glibc 2.39 or later and FUSE 2 (`libfuse2`), **When** the AppImage is made executable and run, **Then** the app opens and works as the packaged one.
2. **Given** FUSE 2 is missing (Ubuntu 24.04 does not install it), **When** the AppImage is run, **Then** it fails with the AppImage's own message, and the README says which package to install (`libfuse2t64` on Ubuntu 24.04).
3. **Given** a newer release, **When** the person downloads the new AppImage, **Then** it reads the same workspace; the app never updates itself or downloads anything.

---

### User Story 3 - The same app everywhere (Priority: P1)

Whichever package is installed, the app behaves the same: same workspace, Ollama or Ollama in Podman, Trash, local-only.

**Why this priority**: A package that installs but behaves differently is worse than none.

**Independent Test**: The release checklist's short Linux section (below) passes on each test machine.

**Acceptance Scenarios**:

1. **Given** any supported distribution, **When** a chat session starts, **Then** the app starts or finds Ollama exactly as on Fedora (installed `ollama`, or `runtime = "podman"`), and connects only to `127.0.0.1:11434`.
2. **Given** GNOME or KDE, **When** a project is deleted, **Then** it goes to that desktop's Trash (freedesktop Trash, already used on Fedora).
3. **Given** Podman on Ubuntu or Debian (AppArmor, not SELinux), **When** the Podman runtime starts the container, **Then** it works; the SELinux option the app adds with a GPU is accepted and has no effect there.

### Edge Cases

- **ARM computers** (Raspberry Pi, ARM laptops): out of scope; packages are x86_64 only, as today.
- **Flatpak and Snap**: out of scope. Their sandboxes keep the app from reading folders the person grants, starting `ollama` or `podman` on the host, and reaching the host's Trash, which the app needs; making that work means wide sandbox holes that defeat the point.
- **Distribution repositories** (an apt repository, Flathub, the AUR): out of scope; releases stay files on the GitHub release page, so nothing new needs hosting or signing keys.
- **Both a package and an AppImage**: they read the same workspace and settings; running both at once is like running the app twice.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Each release MUST contain, besides the Fedora `.rpm`, a `.deb` (`ai-workspace_<version>_amd64.deb`) and an AppImage (`ai-workspace-<version>-x86_64.AppImage`), all built from the same tag by the release workflow.
- **FR-002**: The `.deb` MUST install as the package `ai-workspace` with the command `ai-workspace` and the menu entry **AI Workspace**, and declare its library dependencies so `apt` installs them.
- **FR-003**: The release workflow MUST smoke-test each package before drafting the release: the `.rpm` in Fedora (as today), the `.deb` in Ubuntu 24.04 and Debian 13 containers (installs with `apt`, the command resolves, the menu entry exists, the binary links), and the AppImage (it extracts, and its binary links).
- **FR-004**: `SHA256SUMS` and the signed build provenance MUST cover every package.
- **FR-005**: The supported systems MUST be written down: Fedora (current), Ubuntu 24.04 LTS and later, Debian 13 and later, Linux Mint 22 and later; AppImage on other x86_64 distributions with glibc 2.39 or later and FUSE 2.
- **FR-006**: The app's behaviour MUST NOT change by package; no code paths depend on the distribution.
- **FR-007**: README, user guide and `docs/release.md` MUST give install, upgrade, removal and verification steps for each package.
- **FR-008**: The release checklist MUST gain a short section per package, run on the test machines below for every release.

### Key Entities

- **Package**: one of `.rpm`, `.deb`, AppImage; all from one build of one tag.
- **Test machine**: a virtual machine with one distribution and desktop, used for that package's checklist section.

## Test machines

Installed by the maintainer as virtual machines (for example GNOME Boxes or virt-manager on the Fedora host). Each needs about 4 virtual CPUs, 8 GB of memory and 40 GB of disk; a small model such as `qwen3:1.7b` answers on the CPU in reasonable time.

| VM | Distribution and desktop | Tests | Why this one |
|---|---|---|---|
| 1 | **Ubuntu 24.04 LTS** Desktop (GNOME) | `.deb` with installed Ollama; then the AppImage on the same VM | The oldest supported system: if the build's library versions are too new, it shows here. Also the most common desktop Linux. |
| 2 | **Debian 13 "trixie"** (GNOME) | `.deb` with **Ollama in Podman** | Debian proper, not Ubuntu; Podman under AppArmor instead of SELinux. |
| 3 | **openSUSE Tumbleweed** (KDE Plasma) | AppImage with installed Ollama | Neither Debian- nor Fedora-based, rolling with the newest libraries, and KDE instead of GNOME. |
| (host) | Fedora (current) | `.rpm`: the full release checklist, as for 1.0 | Unchanged. |

Optional, when time allows: **Ubuntu 26.04 LTS** (the newest LTS) with the `.deb`.

### The short checklist per VM

Copied into the release checklist (FR-008), one copy per VM:

1. Install the package (or make the AppImage executable); **AI Workspace** opens from the menu (`.deb`) or from the file (AppImage).
2. The app says there is no workspace; **Initialize the workspace** creates it.
3. **New project**, add a `.md` note, set the model on **Overview**, **Start session**, ask about the note: the reply uses it.
4. Grant a folder on **Permissions**; the context list shows it.
5. **Delete…** a test project: it lands in the desktop's Trash.
6. Close the app with an answered chat: it asks about a session note.
7. Remove the package (`.deb`) or delete the AppImage: the workspace folder stays.
8. VM 2 only: with `runtime = "podman"`, the container starts and stops with the session.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: The next release page has three Linux files plus `SHA256SUMS` and provenance for each, and the release workflow's smoke tests pass for all three.
- **SC-002**: The short checklist passes on VMs 1 to 3, and the full checklist on Fedora.
- **SC-003**: README tells someone on Ubuntu, Debian or another distribution which file to download and how to install it, in under a minute's reading.

## Assumptions

- Building on GitHub's `ubuntu-24.04` machines (as today) sets the oldest supported glibc at 2.39, which Ubuntu 24.04, Debian 13, Linux Mint 22 and current Fedora meet.
- Tauri's bundler builds `.deb` and AppImage from the same `bundle` configuration as the `.rpm`; the AppImage bundles WebKitGTK, so it is much larger than the packages.
- Ollama's Linux installer and Podman work the same on these distributions; the app already finds whichever answers on `127.0.0.1:11434`.
- Windows and macOS are separate later specs.
