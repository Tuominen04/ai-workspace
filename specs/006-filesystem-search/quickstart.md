# Quickstart: v0.5 Filesystem Search

Validation scenarios for the feature described in [spec.md](./spec.md), using the contracts in [contracts/](./contracts/) and entities in [data-model.md](./data-model.md).

## Prerequisites

```bash
cargo build
export AI_WORKSPACE_HOME=/tmp/ai-search-quickstart
rm -rf "$AI_WORKSPACE_HOME"
cargo run -- init
```

Create a project with enough Markdown content that search has something to differentiate:

```bash
cargo run -- project create --name demo --description "search quickstart"
mkdir -p "$AI_WORKSPACE_HOME/Projects/demo/Knowledge"
cat > "$AI_WORKSPACE_HOME/Projects/demo/Knowledge/ollama-setup.md" <<'EOF'
# Ollama Setup
Install Ollama locally and start the runtime before running ai.
EOF
cat > "$AI_WORKSPACE_HOME/Projects/demo/Knowledge/permissions.md" <<'EOF'
# Permissions
Temporary permissions are granted per session and revoked on exit.
EOF
```

## Scenario 1 — Standalone search finds relevant content (User Story 1 & 3)

```bash
cargo run -- search --query "ollama runtime" --project demo
```

**Expected**: `ollama-setup.md` is returned (and ranked above `permissions.md`, which doesn't mention Ollama), with a snippet showing the matched text — per `contracts/cli-search.md`'s success output. Confirms FR-001, FR-003, FR-005.

## Scenario 2 — No matches reported clearly (edge case)

```bash
cargo run -- search --query "kubernetes cluster autoscaling" --project demo
```

**Expected**: Exit code `0`, "No documents matched" message — not an error. Confirms FR-004.

## Scenario 3 — Permission boundary is respected (SC-004)

```bash
mkdir -p /tmp/ai-search-outside
echo "# Secret\nunrelated ollama content" > /tmp/ai-search-outside/secret.md
cargo run -- search --query "ollama" --project demo
```

**Expected**: `secret.md` never appears in results — it was never collected as a candidate because it isn't under Base or the `demo` project's granted paths. Confirms FR-002/FR-007. This scenario is also covered by an automated test (see `tests/search_tests.rs`'s permission-boundary case, added to close a coverage gap found during `/speckit-analyze`), not just this manual pass.

Now grant it and confirm it becomes visible:

```bash
# (interactive session required to grant; see ai permission grant --path /tmp/ai-search-outside)
```

## Scenario 4a — Session start is already relevance-selected (User Story 2, AC1)

Populate enough Knowledge files to exceed `DEFAULT_MAX_CONTEXT_SOURCES` (8), set the `demo` project's `--description` to something Ollama-specific (e.g. `"local Ollama runtime setup and troubleshooting"`), then start an interactive session but stop *before* typing anything at the chat prompt:

```bash
cargo run -- 
# select "demo" project, then check the session summary before sending any chat message
```

**Expected**: The very first system prompt (built from the project's description as the implicit query, per `contracts/context-builder-integration.md`) already includes `ollama-setup.md`, even though no question has been asked yet and it wasn't among the first 8 files in priority order. This is the "session starts" half of FR-006/AC1 — distinct from Scenario 4b below, which covers the "question is asked" half.

## Scenario 4b — Context assembly tracks the current question (User Story 2, AC1)

Continuing the same session, ask a targeted question:

```bash
# at the chat prompt: "How do I set up Ollama?"
```

**Expected**: The system prompt sent to the runtime for that turn (visible via `ai`'s session summary / chat context, or by temporarily logging `context.assembled_content` during manual verification) includes `ollama-setup.md`, ranked ahead of unrelated files. Confirms FR-006/FR-007 and User Story 2's acceptance scenarios.

## Scenario 5 — Small workspace still gets everything (User Story 2, AC2)

In a workspace with only 2-3 Knowledge files (well under the caps), start a session and confirm the initial context still includes all of them, regardless of what the project's description happens to rank highest — relevance selection only matters once the cap would otherwise truncate something.

## Scenario 6 — Search stays fast as the workspace grows (SC-003)

Generate ~200 small Markdown fixture files under `Projects/demo/Knowledge/` (e.g. a short loop writing `doc-001.md` .. `doc-200.md`, with a handful containing "ollama"), then time a search:

```bash
time cargo run -- search --query "ollama" --project demo
```

**Expected**: Completes in well under 2 seconds. This scenario exists specifically to give SC-003 at least one concrete verification pass, since it's a performance criterion not otherwise exercised by the smaller fixture sets in Scenarios 1-5.

## Automated coverage

These scenarios map to `tests/search_tests.rs` (Scenarios 1–3) and `tests/context_tests.rs` (Scenarios 4a–5), run via:

```bash
cargo test --test search_tests
cargo test --test context_tests
```

Scenario 6 (SC-003) is validated manually per this quickstart rather than as an automated test, to avoid a timing-based test that could flake in CI; re-run it by hand after any change to `search.rs`'s scoring logic.
