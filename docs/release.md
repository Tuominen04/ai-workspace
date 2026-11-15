# Releases

A release of AI Workspace is two Fedora packages, attached to a GitHub release together
with a `SHA256SUMS` file:

- `ai-workspace-<version>.x86_64.rpm` — the `ai-workspace` command;
- `ai-workspace-desktop-<version>.x86_64.rpm` — the desktop app (`docs/desktop.md`).

Both carry the same version, and either can be installed without the other. It is built by
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
ai-workspace-desktop-<version>.x86_64.rpm: OK
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
- the command's package does not install on Fedora, or the installed command
  does not report `ai-workspace <version>`;
- the desktop package does not install on Fedora, is not version `<version>`,
  has no menu entry, or its binary is missing a library.

To retry after fixing the cause, delete the tag
(`git push --delete origin v<version>` and `git tag -d v<version>`) and push
it again from the fixed `main`.

## What the workflow does

Two jobs:

- **build** (read-only access): checks the tag, runs
  `cargo test --locked --workspace`, builds with
  `cargo build --release --locked -p ai-workspace-cli`, packages with
  `cargo generate-rpm -p crates/ai-workspace-cli --auto-req builtin`, installs the package in a Fedora
  container and runs `ai-workspace --version`; then builds the desktop app
  with `npm run tauri build` (Tauri's bundler makes its rpm), installs that in
  Fedora too and checks it links; then writes `SHA256SUMS`.
- **draft** (write access): takes the finished files and creates the draft
  release. It checks out no code and runs no project code.

## Building the package locally

For checking the package, not for releasing it:

```bash
cargo install --locked cargo-generate-rpm@0.21.1   # once
cargo build --release --locked -p ai-workspace-cli
cargo generate-rpm -p crates/ai-workspace-cli --auto-req builtin
rpm -qip target/generate-rpm/ai-workspace-*.rpm   # name, version, license
rpm -qlp target/generate-rpm/ai-workspace-*.rpm   # the three installed files
```

The desktop package is built from `desktop/` with `npm ci && npm run tauri build`; it lands
in `target/release/bundle/rpm/`, named after the product (`AI Workspace Desktop-…rpm`).
Its metadata is the `bundle` section of `desktop/src-tauri/tauri.conf.json`; the package
name `ai-workspace-desktop` comes from `productName`.

The command's package is described under `[package.metadata.generate-rpm]` in
`crates/ai-workspace-cli/Cargo.toml`; its version, license and summary come
from that crate, which takes version and license from the workspace.

## Not included yet

The repository is private, so releases carry no build provenance attestation
and tags are not protected by a ruleset. Both can be added if the repository
becomes public.
