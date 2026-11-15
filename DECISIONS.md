# Architecture Decisions

## ADR-0001

Filesystem is the source of truth.

Reason

Everything remains portable.

Status

Accepted

---

## ADR-0002

Rust is the primary language.

Reason

Long-term maintainability.

Status

Accepted

---

## ADR-0003

Podman instead of Docker.

Reason

Fedora native.

Status

Accepted

---

## ADR-0004

Runtime Manager sends chat requests to Ollama via a blocking HTTP client
(`ureq`), calling `/api/chat` with `stream: false`.

Reason

The codebase is fully synchronous today; a single localhost HTTP call does
not justify introducing an async runtime. Non-streaming keeps response
handling to one JSON object per turn instead of incremental NDJSON parsing.

Status

Accepted

---

## ADR-0005

Search-driven context selection. The Context Builder ranks Base/Project/Session
Markdown by relevance instead of a fixed priority order whenever a query is
available: the project's own description at session start, the user's
message on every chat turn thereafter. Ranking uses simple, dependency-free
keyword scoring in a new `search.rs` module; it never drops a source that
would otherwise fit under the existing size caps, so a small workspace still
gets everything.

Reason

As workspace documentation grows past the Context Builder's fixed
source/byte caps, a fixed priority order stops being "relevant" and starts
being arbitrary. Search relevance is what makes the selection actually
track the current task (v0.5, `specs/006-filesystem-search/`).

Status

Accepted

---

## ADR-0006

Release packaging, the `ai-workspace` command name, and the license. A release
is a Fedora `.rpm` built by GitHub Actions when a `v<version>` tag is pushed
and attached, with a `SHA256SUMS` file, to a draft release the maintainer
publishes by hand. The package is built with `cargo-generate-rpm` from
metadata in `Cargo.toml` and is installed in a Fedora container before the
draft is created. The command is renamed from `ai` to `ai-workspace`, matching
the crate, the repository and the product name (`local-ai` was considered
first and dropped before any release, partly because LocalAI ships a binary
of that name), and every message that names it is built from one
`COMMAND_NAME` constant in `lib.rs`. The project is licensed
`MIT OR Apache-2.0`.

Reason

The installed command had been a hand-copied build that fell months behind
the code. A tag-triggered package makes every version installable with
`dnf`, the same way the maintainer's other project is released. Build
automation in the cloud was explicitly requested and is build-time only;
the installed tool stays offline. `cargo-generate-rpm` needs no `.spec`
file or Fedora build container and keeps the version in one place. `ai` is
too generic a name for a system-wide command; a single constant keeps the
name from being scattered across string literals again. A package must
declare a license, and the dual license matches the maintainer's other
project (v0.6, `specs/007-release-packaging/`).

Status

Accepted

---

## ADR-0007

The workspace folder is a per-user setting. `ai-workspace config set-root PATH`
validates that PATH is an existing directory and saves its absolute path as
`workspace_root` in `~/.config/ai-workspace/config.toml` (under
`$XDG_CONFIG_HOME` when that is an absolute path). The folder is resolved in
this order: the `AI_WORKSPACE_HOME` environment variable, then that file, then
`~/AI`. `status` and `config show` report which one was used. `--help`,
`--version` and `config set-root` never need a workspace, so they work even
when the saved setting is missing or broken.

Reason

An installed package cannot know where the user keeps the workspace, and an
environment variable has to be exported from a shell profile. The setting
cannot live inside the workspace, because the tool has to find it first, so it
is a small TOML file the user can read and edit, in line with the
filesystem-first rule. The environment variable stays on top so tests and
one-off runs can still point at a scratch folder (`specs/` was skipped for this
change at the maintainer's request).

Status

Accepted

---

## ADR-0008

The engine is split from the command-line front end into two crates of one
Cargo workspace (resolver 3) in this repository:

- `crates/ai-workspace-core` (library `ai_workspace_core`): all engine
  behavior. It returns data and never prints, reads the terminal, or knows the
  command's name.
- `crates/ai-workspace-cli` (package `ai-workspace-cli`, binary
  `ai-workspace`): argument parsing, rendering, prompts, usage text and
  `COMMAND_NAME`. The installed command and the RPM stay `ai-workspace`.

Cargo enforces the direction: the core cannot depend on the CLI. Version,
edition and license live once, in `[workspace.package]`.

The core's front-end API:

```rust
pub struct Engine { /* WorkspacePaths + RootSource */ }
impl Engine {
    pub fn open() -> Result<Engine>;                    // resolves the root (ADR-0007)
    pub fn at(paths: WorkspacePaths, source: RootSource) -> Engine;  // an already resolved root
    pub fn paths(&self) -> &WorkspacePaths;
    pub fn root_source(&self) -> &RootSource;
    pub fn init(&self) -> Result<InitReport>;
    pub fn projects(&self) -> Result<Vec<Project>>;     // creates the layout and config if missing
    pub fn config(&self) -> Result<WorkspaceConfig>;
    pub fn validate(&self) -> Result<ValidationResult>;
    pub fn create_project(&self, name: &str, description: Option<&str>) -> Result<CreatedProject>;
    pub fn project_details(&self, name: &str) -> Result<(Project, ProjectMetadata)>;
    pub fn grant(&self, project: Option<&str>, path: &str) -> Result<(Project, PermissionGrantResult)>;
    pub fn revoke(&self, project: Option<&str>, path: &str) -> Result<(Project, PermissionRevokeResult)>;
    pub fn permissions(&self, project: Option<&str>, options: PermissionViewOptions)
        -> Result<(Project, PermissionViewSnapshot)>;
    pub fn search(&self, query: &str, project: Option<&str>, limit: usize) -> Result<SearchOutcome>;
}
pub fn set_workspace_root(path: &str) -> Result<SetRootReport>;   // needs no workspace

pub struct Session { /* project, model, runtime, chat history */ }
impl Session {
    pub fn open(engine: &Engine, project: &Project) -> Result<Session>;
    pub fn summary(&self) -> SessionSummary<'_>;        // project, first context, runtime status
    pub fn can_chat(&self) -> bool;                     // a runtime is available
    pub fn send(&mut self, message: &str) -> Result<String>;
    pub fn close(self) -> Result<usize>;                // temporary grants revoked
}
```

How the design points were settled:

- **Validation gate.** `AppError::ValidationGateBlocked` carries the
  `GatedOperation` that was refused and the full `ValidationResult`, so any
  front end can render the blocking issues. The CLI prints the same two lines
  as before and then the error.
- **Hints that name the command.** `ValidationIssue::suggested_fix` is a
  `SuggestedFix` value (run an `Action` such as `Init` or `Validate`, fix
  something by hand and rerun one, or a plain manual step) instead of text
  containing "run `ai-workspace init`". The CLI turns an `Action` into a
  command line. Errors whose wording is CLI vocabulary (the gate, the missing
  `--project`) are worded by the CLI; the core's `Display` stays neutral.
  Usage errors, including an empty search query, belong to the CLI's argument
  parser and leave the core. One exception stays: the per-user config file's
  first-line comment names `ai-workspace config set-root`, because that file
  and its folder are named after the product, not after a front end.
- **Who owns the runtime.** `Session` owns the `RuntimeManager`, so closing or
  dropping a session stops an Ollama the session started. Dropping a session
  that was not closed also closes its permission session, so an error or a
  panic in a front end no longer leaves temporary grants behind.
- **Streaming.** Unchanged: one non-streaming reply per turn (ADR-0004).
- **Platform differences.** The home and per-user config folder lookups sit in
  one small `platform` module inside `paths.rs`. Only Linux behavior exists;
  macOS and Windows are not added.
- **Public surface.** `Engine`, `Session`, their report types and the data
  types they return are the API front ends use. The lower modules (`paths`,
  `config`, `workspace`, `context`, `search`, `runtime`) stay public for now,
  because the existing integration tests drive them directly and are the
  regression oracle for this split; narrowing them is follow-up work once
  those tests go through `Engine`.

Tests that only use the library live in the core crate; tests that run the
binary live in the CLI crate. They pass unchanged apart from where they live
and the crate name in their `use` lines. The CLI's output stays byte-for-byte
the same.

Reason

`session.rs` did the work and printed it in the same functions, and its chat
loop and project selection read stdin. A second front end, most likely a
Tauri desktop app (not decided), could only get printed text back. A crate
boundary makes "business logic never depends on UI" (`ARCHITECTURE.md` rule
4) something the compiler checks. One repository is chosen over a separate
engine repository because the core's API is about to change repeatedly
(sessions, project agents), one tag should release every artifact, and a
solo maintainer would pay for every contract change three times across two
repositories. A crate can be lifted into its own repository later, mostly
mechanically.

Revisit if another project needs to depend on the core, a front end is seen
reaching around the core's public API, or core changes start being batched or
avoided because of the boundary.

Status

Accepted

---

## ADR-0009

A desktop app is the next front end, ahead of project agents, so visual work
can be seen and tested as it grows. It is a Tauri 2 app on the core library
from ADR-0008, built with the same stack as the maintainer's Learning Corner:
React 19 and TypeScript, bundled by Vite, with the wire types generated from
Rust by `ts-rs`.

Layout, in this repository:

```txt
desktop/                    the interface (npm package, private): index.html, src/, e2e/
desktop/src/lib/engine.ts   the only file that calls Tauri's `invoke`
desktop/src/lib/bindings/   TypeScript generated from the shell's wire types
desktop/src-tauri/          crate ai-workspace-desktop: window, commands, wire types
```

`desktop/src-tauri` is a member of the Cargo workspace but not a default
member, so `cargo build`, `cargo run` and `cargo test` at the root still build
only the core and the CLI and need no WebKitGTK. The desktop crate is built
with `--workspace`, `-p ai-workspace-desktop`, or the Tauri CLI. Its version
comes from `[workspace.package]` (`tauri.conf.json` and `package.json` carry
none of their own), so one tag still versions every artifact.

The first version is **read-only plus chat**: workspace status and projects,
a project's details, the workspace configuration, validation results, search,
and a chat window driving `Session`. Creating projects and changing permissions stay in the CLI for now.

- **Commands are thin.** Each Tauri command calls one `Engine` or `Session`
  method and converts the result to a wire type in `dto.rs`. The core's own
  types are not sent across, so the interface's contract is the generated
  bindings, and CI fails when they are stale.
- **One session at a time.** The open `Session` lives in the shell's state.
  Opening another closes the first, and the shell closes it when the app exits,
  so temporary grants never outlive the window.
- **No blocking in the window.** Opening a session (which may start Ollama)
  and sending a message run off the main thread.
- **No network from the webview.** The content security policy allows only
  the app's own files, and the capability list is `core:default`. Ollama is
  reached from Rust, as in the CLI.
- **The theme is the interface's only state.** System, Light or Dark is kept in
  the webview's local storage: a display preference of the computer, not
  workspace state, so it stays out of the workspace folder.
- **Validation fixes stay structured.** `SuggestedFix` crosses as data (kind,
  action, text); the interface words it. The desktop names no CLI command.
- **Tests.** Shell tests call the command functions against temporary
  workspaces and regenerate the bindings. Interface logic has `vitest` tests.
  Browser checks run the interface in Chromium with a stub engine behind
  `invoke`, as Learning Corner does. Ollama is never needed.
- **Packaging** is a separate pull request, and ADR-0010 settles it: the
  desktop app is the only package released.

Reason

A window makes progress visible and gives the maintainer something to try
without reading terminal output. Reusing Learning Corner's stack means one set
of tools, conventions and test patterns across both projects; JavaScript
tooling is a build-time cost only, and the installed app stays offline. The
core split made this possible without moving engine logic: the shell only
converts and renders. Read-only plus chat is the smallest version that is
useful every day and exercises every part of the core's API that returns data.

Status

Accepted

---

## ADR-0010

Only the desktop app is released. It is the `ai-workspace` package and the
`ai-workspace` command: `/usr/bin/ai-workspace` opens the window, and the
applications menu lists it as **AI Workspace**. The command-line front end is
no longer packaged; its code stays in `crates/ai-workspace-cli`, CI keeps
testing it, and it is run from source (`cargo run -- <args>`) as a developer
tool.

- **Package.** Built by Tauri's bundler from the `bundle` section of
  `desktop/src-tauri/tauri.conf.json`. Tauri derives the RPM's name from
  `productName` ("AI Workspace") and the installed binary's from
  `mainBinaryName` ("ai-workspace"); neither may change without changing the
  package. The cargo binary is still `ai-workspace-desktop`, so `cargo run -p
  ai-workspace-desktop` and the CLI's `cargo run` do not clash; `npm run tauri
  build` writes `target/release/ai-workspace`, the path a release build of the
  CLI also uses, so locally the last build wins.
- **Upgrade.** Because the package keeps the name `ai-workspace`, a newer
  release replaces an installed command-line package on `dnf upgrade`. The
  release must carry a higher version than the installed 0.6.0.
- **Release workflow.** Builds and smoke-tests only this package: it installs
  in Fedora, `ai-workspace` resolves to `/usr/bin/ai-workspace`, the menu entry
  exists, and the binary links. `cargo-generate-rpm` is no longer used; the
  CLI crate's `[package.metadata.generate-rpm]` stays in place, unused, should
  the command line be released again.
- **What users lose for now.** The installed app has no way yet to create
  projects, grant or revoke permissions, or choose the workspace folder; those
  were command-line features. Until the desktop app gains them, the folder is
  chosen with `AI_WORKSPACE_HOME` or by editing
  `~/.config/ai-workspace/config.toml`, and the rest is done from source.

Reason

The maintainer wants one thing to install and use, and the window is that
thing. Keeping the command line as code costs little (CI already tests it) and
keeps a scriptable front end on the core. Reusing the `ai-workspace` name
for the package and the command means an installed system upgrades in place
instead of collecting two packages.

Status

Accepted. Supersedes the packaging part of ADR-0006 (the `cargo-generate-rpm`
package of the command line); the tag-triggered draft release, the command
name and the license stand.

---

## ADR-0011

The desktop app gains the workspace's write operations, so the installed app
(ADR-0010) can do what the command line did:

- **Create a project** (name and optional description), from the sidebar.
- **Permissions**, a tab per project: the default and temporary grants with
  their status, filtered and paged; **grant** temporary access to a path;
  **revoke** a temporary grant. Grants live in the project's permission
  session exactly as with the command line, and ending a chat session in that
  project revokes them, as before.
- **Choose the workspace folder**, in the Configuration tab: it is saved in
  `~/.config/ai-workspace/config.toml` (ADR-0007). When `AI_WORKSPACE_HOME` is
  set, the app says that the variable still wins.
- **Initialize the workspace**: create the standard layout and default
  configuration, from the Configuration tab when the file is missing and from
  validation issues whose fix is to initialize.

How:

- **One core call per command, as before.** Each command calls the existing
  `Engine` method (`create_project`, `grant`, `revoke`, `permissions`, `init`)
  or `set_workspace_root`, so the validation gate, idempotent grants, denied
  paths, audit events and canonical paths behave exactly as on the command
  line. The project is always named explicitly: the window always has one
  selected.
- **Changing the workspace folder closes an open chat session first.** The
  session belongs to the old workspace; closing it revokes its temporary grants
  there instead of leaving them behind.
- **Paths are typed or pasted.** No file dialog yet, so the app's permissions
  stay at `core:default`; a folder picker (Tauri's dialog plugin) can come
  later and would be its own small decision.
- **Errors are shown where the action was taken** (a denied path under the
  grant field, a refused gate with a pointer to Validation), using the core's
  messages.

Reason

ADR-0010 made the desktop app the only thing installed, which left creating
projects, permissions and the workspace folder to a command line that is no
longer installed. These are the operations the core already exposes; the app
only needs to call them and show the results, so this extends ADR-0009's shell
without new engine behavior.

Status

Accepted
