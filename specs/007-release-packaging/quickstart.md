# Quickstart: Release Packaging

Validation scenarios for the feature described in [spec.md](./spec.md), using the contracts in [contracts/](./contracts/) and entities in [data-model.md](./data-model.md).

## Prerequisites

```bash
cargo install --locked cargo-generate-rpm@0.21.1
export AI_WORKSPACE_HOME=/tmp/local-ai-quickstart
rm -rf "$AI_WORKSPACE_HOME"
```

Scenarios 1 to 4 run locally. Scenarios 5 to 7 push tags to GitHub and are run once, when the feature is merged.

## Scenario 1 — The command is called `local-ai` (FR-003, FR-004)

```bash
cargo build
ls target/debug/local-ai          # exists
ls target/debug/ai                # must not exist after a clean build
cargo run -- --version            # local-ai 0.6.0
cargo run -- --help | grep -c '^  local-ai'   # every usage line
cargo run -- --help | grep -c '^  ai '        # 0
cargo run -- project show         # usage error names local-ai
```

**Expected**: the version line is `local-ai 0.6.0`; no output refers to the command as `ai`.

## Scenario 2 — Tests pass without a local AI runtime (FR-012, research.md Decision 8)

```bash
command -v ollama                  # note the directory it is in, if any
PATH=/usr/bin:/bin cargo test --locked
```

**Expected**: every test passes. If `ollama` lives in `/usr/bin` or `/bin`, stop the Ollama service and move the check to a container or a machine without it; the point is that no test needs the runtime.

## Scenario 3 — The package has the right identity and contents (FR-001, FR-005, FR-006)

```bash
cargo build --release --locked
cargo generate-rpm --auto-req builtin
RPM=$(ls target/generate-rpm/local-ai-*.rpm)
rpm -qip "$RPM"     # Name: local-ai, Version: 0.6.0, License: MIT OR Apache-2.0
rpm -qlp "$RPM"     # exactly the three files in contracts/rpm-package.md
rpm -qp --scripts "$RPM"   # no scripts
```

**Expected**: identity and file list match [contracts/rpm-package.md](./contracts/rpm-package.md); no install scripts.

## Scenario 4 — Install, upgrade, remove; the workspace is untouched (FR-002, FR-007, FR-008, SC-006)

```bash
ws="$HOME/ai-local/workspaces"          # or any real workspace folder
before=$(find "$ws" -type f -exec sha256sum {} + | sort | sha256sum)

sudo dnf install "./$RPM"
# open a new terminal
command -v local-ai            # /usr/bin/local-ai
local-ai --version             # local-ai 0.6.0
rpm -ql local-ai               # the command and the two license texts

sudo dnf remove local-ai
command -v local-ai            # nothing

after=$(find "$ws" -type f -exec sha256sum {} + | sort | sha256sum)
[ "$before" = "$after" ] && echo "workspace unchanged"
```

Upgrade check: build a second package with the version temporarily set one patch higher, install it over the first, and confirm `rpm -q local-ai` reports only the higher version. Revert the version afterwards.

**Expected**: the command appears and disappears with the package; `workspace unchanged` is printed.

## Scenario 5 — A wrong tag produces no release (FR-010, FR-011, SC-004)

Version mismatch:

```bash
git tag v0.0.1 origin/main && git push origin v0.0.1
gh run watch                       # fails at "Tag matches the version", naming both versions
gh release view v0.0.1             # release not found
git push --delete origin v0.0.1 && git tag -d v0.0.1
```

Tag off `main`:

```bash
git checkout -b off-main && git commit --allow-empty -m "off main" && git push origin off-main
git tag v0.6.0-offmain && git push origin v0.6.0-offmain
gh run watch                       # fails at "Tag is on main"
git push --delete origin v0.6.0-offmain off-main
git tag -d v0.6.0-offmain && git checkout main && git branch -D off-main
```

**Expected**: both runs fail before any compilation; no release exists for either tag.

## Scenario 6 — A correct tag produces a draft with the package and checksums (FR-009, FR-014, FR-015, SC-002)

```bash
git checkout main && git pull
git tag v0.6.0 && git push origin v0.6.0
gh run watch
gh release view v0.6.0 --json isDraft,assets -q '.isDraft, .assets[].name'
```

**Expected**: within 15 minutes, `true`, then `SHA256SUMS` and `local-ai-0.6.0.x86_64.rpm`. `gh release list` does not mark it Latest until it is published.

Pushing the same tag again (delete and re-push) must fail at "Release must not exist" and leave the draft as it was (FR-016).

## Scenario 7 — A download verifies (FR-014, SC-005)

```bash
mkdir /tmp/local-ai-dl && cd /tmp/local-ai-dl
gh release download v0.6.0 -R Tuominen04/ai-workspace
sha256sum -c SHA256SUMS            # local-ai-0.6.0.x86_64.rpm: OK
printf 'x' >> local-ai-0.6.0.x86_64.rpm
sha256sum -c SHA256SUMS            # FAILED
```

**Expected**: `OK` for the untouched file, `FAILED` after altering it.

## After validation

```bash
gh release edit v0.6.0 --draft=false     # publish
rm ~/.local/bin/ai                        # remove the old hand-copied command
```
