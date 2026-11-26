# Releases

A release of AI Workspace is three Linux files of the desktop app (`docs/desktop.md`),
installed as the `ai-workspace` command (ADR-0010, ADR-0017), attached to a GitHub release
together with a `SHA256SUMS` file and their signed build provenance:

| File | For |
|---|---|
| `ai-workspace-<version>.x86_64.rpm` | Fedora |
| `ai-workspace_<version>_amd64.deb` | Ubuntu 24.04+, Debian 13+, Linux Mint 22+ |
| `ai-workspace-<version>-x86_64.AppImage` | Other x86_64 distributions with glibc 2.39+ and FUSE 2 |

The command-line front end is not released; it is run from source. The files are built by
`.github/workflows/release.yml`, which calls `.github/workflows/packages.yml`; nobody builds
a release by hand.

Installing a release is covered in the [README](../README.md#install).

## Verifying a download

Download your file and `SHA256SUMS` into the same folder, then:

```bash
sha256sum -c --ignore-missing SHA256SUMS
```

`--ignore-missing` skips the files you did not download. Expected output, for example:

```text
ai-workspace_<version>_amd64.deb: OK
```

`FAILED` means the file is not the one the release process produced. Do not
install it; download it again.

### Checking where it was built

From 1.0.1, each package comes with signed build provenance: a statement, signed through
Sigstore, that the file was built by this repository's release workflow from the tag's commit.
With the GitHub command-line tool (`sudo dnf install gh`, then `gh auth login`):

```bash
gh attestation verify <file> --repo Tuominen04/ai-workspace
```

with `<file>` the `.rpm`, `.deb` or AppImage you downloaded (install `gh` with `sudo apt
install gh` on Ubuntu and Debian).

It ends with `Verification succeeded!` for a genuine package. The same signed statement is
attached to the release: as `ai-workspace-<version>.x86_64.rpm.sigstore.json` up to 1.0.1, and
from 1.1 as `ai-workspace-<version>.sigstore.json`, one statement covering the `.rpm`, the
`.deb` and the AppImage. Pass it with `--bundle` to check against that file instead of
GitHub's copy.

## Cutting a release

0. Run the [release checklist](checklist/release-checklist.md) against the commit to be tagged,
   with real Ollama on the current Fedora, and its short Linux section on the test machines
   (Ubuntu 24.04 and Debian 13 with the `.deb`, openSUSE with the AppImage; spec 010).
   For 1.0 and later this is required (ADR-0016):
   the filled-in copy is kept as `docs/checklist/v<version>.md` and linked from the version
   bump pull request, and no issue labelled `bug` may be open.
1. Set the new `version` under `[workspace.package]` in the root `Cargo.toml`,
   run `cargo build` so `Cargo.lock`
   follows, and merge that to `main`.
2. Tag the merged commit and push the tag:

   ```bash
   git checkout main && git pull
   git tag v<version>
   git push origin v<version>
   ```

3. Wait for the **Release** workflow (`gh run watch`). It ends with a **draft**
   release holding the three packages, `SHA256SUMS` and the provenance.
4. Read the draft. Download the packages from it and try them if you want to.
5. Publish it:

   ```bash
   gh release edit v<version> --draft=false
   ```

The workflow never publishes on its own. Until step 5 the release is a draft
and is not shown as the latest release.

## What stops a release

The workflow ends without creating anything when:

- a release for the tag already exists (the existing one is left unchanged);
- the tagged commit is not on `main`;
- the tag does not equal `version` in the root `Cargo.toml` (the message states both);
- any test fails, or `Cargo.lock` is out of date;
- the `.rpm` does not install on Fedora, or the `.deb` on Ubuntu 24.04 or Debian 13,
  or either is not `ai-workspace <version>`, does not provide the `ai-workspace`
  command or its menu entry, or the binary is missing a library;
- the AppImage does not extract, or its binary is missing a library that it should
  carry itself.

To retry after fixing the cause, delete the tag
(`git push --delete origin v<version>` and `git tag -d v<version>`) and push
it again from the fixed `main`.

## What the workflow does

Three jobs:

- **checks** (read-only access): the release does not exist yet, the tag is on
  `main` and equals the version.
- **build** (read-only access, `packages.yml`): runs
  `cargo test --locked --workspace` (the core, the command line and the
  desktop shell), builds the three packages with `npm ci && npm run tauri build`
  in `desktop/`, checks the `.rpm` in a Fedora container, the `.deb` in Ubuntu
  24.04 and Debian 13 containers and the extracted AppImage in a Debian
  container, then writes `SHA256SUMS`. The same workflow runs on pull requests
  that change the packaging, so the files can be downloaded from that run and
  tried before a release.
- **draft** (write access): takes the finished files, attests their build
  provenance and creates the draft release. It checks out no code and runs no
  project code.

## Building the package locally

For checking the package, not for releasing it (needs the packages listed in
`docs/desktop.md`):

```bash
cd desktop
npm ci
npm run tauri build
rpm -qip "../target/release/bundle/rpm/AI Workspace-"*.rpm   # name, version, license
rpm -qlp "../target/release/bundle/rpm/AI Workspace-"*.rpm   # the installed files
dpkg-deb -I ../target/release/bundle/deb/*.deb                # name, version, dependencies
```

The AppImage step needs FUSE, or `APPIMAGE_EXTRACT_AND_RUN=1` in the environment as the
workflow sets it.

The package is described by the `bundle` section of
`desktop/src-tauri/tauri.conf.json`. Its name `ai-workspace` comes from
`productName` ("AI Workspace") and the installed binary's from `mainBinaryName`;
Tauri names the files after the product, and the workflow renames them to the names in
the table above. The version comes from the workspace.
