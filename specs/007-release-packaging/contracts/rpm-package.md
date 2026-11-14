# Contract: The `local-ai` Package

Covers FR-001, FR-002, FR-005 to FR-008, and FR-018 (see spec.md). Field values are listed in [data-model.md](../data-model.md).

## Identity

```text
Name        : local-ai
Version     : <project version>
Release     : 1
Architecture: x86_64
License     : MIT OR Apache-2.0
Summary     : Local-first AI workspace manager for Fedora
```

File name on a release: `local-ai-<version>.x86_64.rpm`.

## Contents

Exactly three files:

```text
/usr/bin/local-ai
/usr/share/licenses/local-ai/LICENSE-MIT
/usr/share/licenses/local-ai/LICENSE-APACHE
```

`cargo-generate-rpm` 0.21.1 has no way to flag a file as a license file, so `rpm -qL local-ai` lists nothing; the texts are installed at the standard location regardless.

## What the package never does

- No install, upgrade, or removal scripts.
- No file under `/home`, `/etc`, or `/var`.
- No service, timer, or autostart entry.
- No requirement on Ollama or any other AI runtime.
- No change to, creation of, or deletion of a workspace folder.

## Where the metadata lives

`Cargo.toml`, under `[package]` (`version`, `description`, `license`) and `[package.metadata.generate-rpm]` (`name`, `require-sh = false`, the three assets with their destinations and modes). Built-in dependency detection is selected on the command line (`--auto-req builtin`), because the metadata key only accepts `"no"`. The version is written once, in `[package] version`.

## Behavior under the package manager

| Action | Expected result |
| --- | --- |
| `sudo dnf install ./local-ai-<version>.x86_64.rpm` | `local-ai` is on the path in a new terminal; no prompts beyond the usual confirmation |
| Install a higher version over a lower one | handled as an upgrade; one version installed afterwards |
| `sudo dnf remove local-ai` | the three files are gone; the workspace folder is unchanged |
| `rpm -qi local-ai` | shows the identity fields above |
| `rpm -ql local-ai` | shows exactly the three files above |

## Local build (for verification, not for release)

```bash
cargo install --locked cargo-generate-rpm@0.21.1   # once
cargo build --release --locked
cargo generate-rpm --auto-req builtin
ls target/generate-rpm/
```

Releases are built only by the workflow in [release-workflow.md](./release-workflow.md).
