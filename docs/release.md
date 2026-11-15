# Releases

A release of AI Workspace is one Fedora package, `ai-workspace-<version>.x86_64.rpm`,
attached to a GitHub release together with a `SHA256SUMS` file. The package is the
desktop app (`docs/desktop.md`), installed as the `ai-workspace` command (ADR-0010). The
command-line front end is not released; it is run from source. It is built by
`.github/workflows/release.yml`; nobody builds a release by hand.

Installing a release is covered in the [README](../README.md#install).

## Verifying a download

Download the package and `SHA256SUMS` into the same folder, then:

```bash
sha256sum -c SHA256SUMS
```

Expected output:

```text
ai-workspace-<version>.x86_64.rpm: OK
```

`FAILED` means the file is not the one the release process produced. Do not
install it; download it again.

## Cutting a release

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
   release holding the package and `SHA256SUMS`.
4. Read the draft. Download the package from it and try it if you want to.
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
- the package does not install on Fedora, is not `ai-workspace <version>`,
  does not provide the `ai-workspace` command or its menu entry, or the binary
  is missing a library.

To retry after fixing the cause, delete the tag
(`git push --delete origin v<version>` and `git tag -d v<version>`) and push
it again from the fixed `main`.

## What the workflow does

Two jobs:

- **build** (read-only access): checks the tag, runs
  `cargo test --locked --workspace` (the core, the command line and the
  desktop shell), builds the package with `npm ci && npm run tauri build` in
  `desktop/`, installs it in a Fedora container and checks it there, then
  writes `SHA256SUMS`.
- **draft** (write access): takes the finished files and creates the draft
  release. It checks out no code and runs no project code.

## Building the package locally

For checking the package, not for releasing it (needs the packages listed in
`docs/desktop.md`):

```bash
cd desktop
npm ci
npm run tauri build
rpm -qip "../target/release/bundle/rpm/AI Workspace-"*.rpm   # name, version, license
rpm -qlp "../target/release/bundle/rpm/AI Workspace-"*.rpm   # the installed files
```

The package is described by the `bundle` section of
`desktop/src-tauri/tauri.conf.json`. Its name `ai-workspace` comes from
`productName` ("AI Workspace") and the installed binary's from `mainBinaryName`;
Tauri names the file after the product, and the workflow renames it to
`ai-workspace-<version>.x86_64.rpm`. The version comes from the workspace.

## Not included yet

The repository is private, so releases carry no build provenance attestation
and tags are not protected by a ruleset. Both can be added if the repository
becomes public.
