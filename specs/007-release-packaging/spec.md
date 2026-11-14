# Feature Specification: Release Packaging

**Feature Branch**: `[007-release-packaging]`

**Created**: 2026-11-14

**Status**: Draft

**Input**: User description: "Release packaging: like Learning Corner, pushing a version tag makes GitHub Actions build an .rpm that can be installed on Fedora with dnf. The installed command could be `ai-workspace` instead of `ai`."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Install a released version on Fedora (Priority: P1)

The maintainer wants the current version of the tool on their Fedora machine without building it by hand and copying a file. They download one package file from the project's releases, install it with the system package manager, and the command is available in every terminal.

**Why this priority**: This is the whole point of the feature. Today the installed command is a hand-copied build from months ago that lacks chat and search; nothing else in this spec matters unless installing a release works.

**Independent Test**: On a Fedora machine with no copy of the tool, install the package file with the system package manager, open a new terminal, and run the command's version and help options.

**Acceptance Scenarios**:

1. **Given** a Fedora machine without the tool installed, **When** the user installs the release package with the system package manager, **Then** the `ai-workspace` command is available on the standard path without any further step.
2. **Given** the package is installed, **When** the user runs the command's version option, **Then** it reports the same version as the release the package came from.
3. **Given** an older release is installed, **When** the user installs a newer release package, **Then** the system package manager treats it as an upgrade and the newer version replaces the older one.
4. **Given** the package is installed, **When** the user removes it with the system package manager, **Then** the command is gone and the user's workspace folder and its contents are untouched.
5. **Given** the package is installed, **When** the user asks the system package manager about it, **Then** it shows the package name, version, summary, and license.

---

### User Story 2 - Cut a release by pushing a version tag (Priority: P2)

The maintainer decides a version is ready. They push one version tag. Without any further manual step, the package is built and attached to a draft release, which the maintainer reads and then publishes.

**Why this priority**: This is what makes Story 1 repeatable for every later version. A package could be built by hand once without it, so it ranks second.

**Independent Test**: Push a version tag for the current version on the main branch and verify a draft release appears with the package file and a checksum file attached, and that nothing is publicly released until the maintainer publishes it.

**Acceptance Scenarios**:

1. **Given** the main branch declares version X, **When** the maintainer pushes tag `vX`, **Then** a draft release titled for version X appears with the package file and a checksum file attached.
2. **Given** a draft release exists, **When** the maintainer has not published it, **Then** it is not shown as the latest release.
3. **Given** the pushed tag names a different version than the project declares, **When** the release process starts, **Then** it stops before building anything and states both versions.
4. **Given** the pushed tag points at a commit that is not on the main branch, **When** the release process starts, **Then** it stops before building anything and says so.
5. **Given** the project's automated tests fail at the tagged commit, **When** the release process runs, **Then** no package is produced and no draft release is created.

---

### User Story 3 - Check that a download is intact (Priority: P3)

Before installing, the maintainer checks that the package file they downloaded is the one the release process produced.

**Why this priority**: A safety check on top of a working install. Useful, but the install works without it.

**Independent Test**: Download the package file and the checksum file from a release, run the standard checksum verification, and confirm it passes; alter the package file and confirm it fails.

**Acceptance Scenarios**:

1. **Given** a release with its package file and checksum file, **When** the user verifies the download against the checksum file, **Then** the verification passes.
2. **Given** a package file that was altered after download, **When** the user verifies it, **Then** the verification fails.

---

### Edge Cases

- **Tag pushed twice or re-pushed** for a version that already has a release: the process fails with a clear message rather than overwriting or duplicating the existing release.
- **Old hand-copied `ai` binary still present** in the user's home directory: the package does not touch it. Both commands exist until the user deletes the old file; the documentation says to remove it.
- **Workspace location**: the installed command finds the workspace the same way as today (the `AI_WORKSPACE_HOME` setting, otherwise `~/AI`). Installing the package neither creates nor moves a workspace.
- **Local AI runtime not installed**: the package installs anyway. The command reports the missing runtime when a session is started, as it does today.
- **A tag that is not a version tag**: no release process runs.
- **Build failure part-way**: no draft release is created and no partial files are published.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The project MUST produce an installable Fedora package for 64-bit Intel/AMD machines containing the command-line tool.
- **FR-002**: The package MUST install the command on the system's standard command path so it is available in every terminal without any further user step.
- **FR-003**: The installed command MUST be named `ai-workspace`.
- **FR-004**: The rename from `ai` to `ai-workspace` MUST be delivered as part of this feature. Every place the command names itself (help text, usage and error messages, version output, and the project documentation) MUST say `ai-workspace`, and the `ai` name MUST NOT be provided by the package or by a build from source.
- **FR-005**: The package MUST carry a name, version, summary, and license, and its version MUST equal the version the project declares at the tagged commit.
- **FR-006**: The project MUST be licensed under MIT or Apache-2.0 at the user's choice, the same as Learning Corner. Both license texts MUST be added to the repository, the package MUST declare that license, and the package MUST install both license texts alongside the command.
- **FR-007**: Installing a newer package over an older one MUST be handled by the system package manager as an upgrade.
- **FR-008**: Installing, upgrading, or removing the package MUST NOT create, modify, or delete the user's workspace folder or anything in it.
- **FR-009**: Pushing a version tag (`v` followed by the version) MUST start the release process with no other manual step.
- **FR-010**: The release process MUST stop before building if the tagged commit is not on the main branch.
- **FR-011**: The release process MUST stop before building if the tag's version differs from the version the project declares, and MUST state both.
- **FR-012**: The release process MUST run the project's automated tests at the tagged commit and MUST NOT produce a release if any fail.
- **FR-013**: The release process MUST build from the committed dependency lock so a release contains exactly the dependency versions recorded in the repository.
- **FR-014**: The release process MUST attach to the release the package file, named with the product name and version, and a checksum file covering every attached file.
- **FR-015**: The release process MUST create the release as a draft; nothing is published until the maintainer publishes it.
- **FR-016**: The release process MUST fail with a clear message, without changing the existing release, when a release for that tag already exists.
- **FR-017**: The part of the release process that can write to the repository MUST NOT run any project or dependency code; building and publishing are kept separate.
- **FR-018**: The installed tool MUST behave identically to one built from the same commit by hand: no network access at run time, no telemetry, no background service, no automatic writes.
- **FR-019**: The project documentation MUST explain how to install a release, how to verify a download, how to cut a release, and that an old hand-copied command should be removed.

### Key Entities

- **Version tag**: A marker on a main-branch commit naming the version being released; the single trigger for a release.
- **Package file**: The installable Fedora package for one version, containing the command and its descriptive information (name, version, summary, license).
- **Checksum file**: A list of fingerprints for every file attached to a release, used to verify downloads.
- **Draft release**: The unpublished release holding the package file and checksum file until the maintainer publishes it.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: From a published release, a user can have a working command in a new terminal in under 2 minutes with a single install command.
- **SC-002**: Cutting a release takes the maintainer exactly one action (pushing the tag) before the draft appears, and the draft appears within 15 minutes.
- **SC-003**: 100% of releases report, from the installed command, the same version as their tag.
- **SC-004**: A tag that is off the main branch, mismatches the declared version, or has failing tests produces no release in 100% of cases.
- **SC-005**: Downloads verify against the published checksum file in 100% of cases for unaltered files and fail for altered ones.
- **SC-006**: Installing, upgrading, and removing the package leave the user's workspace folder byte-for-byte unchanged.

## Assumptions

- The release process runs on the project's existing code-hosting service's automation, as Learning Corner's does. This was explicitly requested and is build-time only; the tool itself stays offline (FR-018).
- Fedora on 64-bit Intel/AMD is the only target. Debian packages, portable bundles, and other architectures are out of scope; Learning Corner ships them because it is a public desktop app, and this tool is Fedora-first.
- The command was first drafted as `local-ai` and changed to `ai-workspace` by the maintainer on 2026-11-14, before any release, so that the command, crate, repository and product share one name.
- The package is named after the command, `ai-workspace`. The repository and the project's own name ("AI Workspace") do not change.
- The rename ships in the same pull request as packaging, by the maintainer's decision (2026-11-14), as an accepted exception to the one-feature-per-pull-request rule: the package and the command name it installs are released together.
- The repository stays private until the maintainer decides to publish it. Adding the license files does not change that.
- Releases are drafts first, mirroring Learning Corner.
- Build provenance attestation and tag-protection rules, which Learning Corner uses, are not available for a private repository on the current plan and are out of scope. Checksums are still provided. Both can be added if the repository becomes public.
- The package does not declare the local AI runtime as a dependency, because the runtime is not available from Fedora's own repositories; the existing "runtime not available" message covers its absence.
- Where the installed command looks for the workspace is unchanged by this feature. Making that a setting in a file, so no environment variable is needed, is a separate feature.
- A test-on-every-pull-request check is a separate feature; this one only requires tests to pass at release time.
- No package repository (for `dnf` to fetch updates automatically) is set up; the user downloads the package file from the release.
