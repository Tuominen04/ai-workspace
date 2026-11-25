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

Superseded by ADR-0013 (replies are streamed)

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

Accepted; the gaps listed under "What users lose for now" are closed by
ADR-0011. Supersedes the packaging part of ADR-0006 (the `cargo-generate-rpm`
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
- **Switch now.** When `AI_WORKSPACE_HOME` is set, saving a folder does not
  change the folder in use, because the variable keeps its priority
  (ADR-0007) and an app cannot unset its parent shell's variable. The app then
  offers **Switch now**: until it restarts, it resolves the workspace as if
  the variable were not set (saved folder, else `~/AI`) and says so wherever
  it names the folder's source. The choice is held in memory only, so a
  restart honours the variable again. The core gains `Engine::open_saved()`
  for this; the variable's priority everywhere else is unchanged. Switching
  also closes an open chat session first.
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

---

## ADR-0012

Project agents (Step C of `docs/plans/core-split-and-agents.md`): a project
can run an Ollama agent, such as the maintainer's `sec` security advisor,
over its own files, and keep a record of what was done. Decided with the
maintainer; built as separate pull requests and released together as 0.8.0.

1. **The agent is an Ollama model, named in the project.** `model` in
   `project.toml` (already read by `resolve_model`) names it; Ollama stays the
   only owner of agents and their Modelfiles. When a session opens, the core
   asks Ollama for the model's own `SYSTEM` instructions (`POST /api/show`)
   and the system message becomes those instructions **followed by** the
   workspace context. Before this, the context alone was sent as the system
   message, and Ollama used it in place of the Modelfile's `SYSTEM`, so the
   agent's rules were lost. If Ollama cannot say, the session goes on with the
   context alone and says so. The desktop app shows which model a session
   uses and whether its own instructions are included, and lets a project's
   model be set from the Overview tab.
2. **Markdown only.** Context keeps coming from `.md` files in Base and the
   project's Knowledge and Session folders; raw configs, logs and scripts are
   not read. Material the agent should see goes into notes. Widening this is a
   later decision.
3. **A built-in block list.** Files whose names look like secrets (`*.key`,
   `*.pem`, `id_rsa*`, `id_ed25519*`, `*.p12`, `*.kdbx`, `.env*`, `*secret*`,
   `*password*`, `*token*`) are never read into context or search, even inside
   granted folders. A session lists what it skipped.
4. **Fit the model's window.** The context's byte budget follows the model's
   `num_ctx` (from `/api/show`) instead of the fixed 12,000 bytes, keeping
   room for the conversation. A file that does not fit whole is cut, with a
   visible `[truncated: N of M bytes shown]` marker, so both the model and the
   person know only part was seen.
5. **Session notes, offered, never automatic.** Ending a session in the
   desktop app offers to save a short Markdown note, drafted by the model and
   editable first, into `Projects/<name>/Session/`, where later sessions pick
   it up as context. Nothing is written without the person's OK, in line with
   the no-automatic-writes rule. Closing the window with an answered chat open asks the same question
   before the window closes, since the conversation itself is not kept (amended
   with the maintainer).

Reason

The maintainer's goal is an advisor that knows the history of their security
work and keeps its own rules. Each point removes one of the four obstacles the
plan found: replaced instructions, unreadable or unsafe files, a context too
small and cut silently, and no memory between sessions. Ollama keeps owning
models, the filesystem keeps being the only store, and every write stays
explicit.

Status

Accepted

---

## ADR-0013

Replies are streamed. A chat request asks Ollama for `stream: true` and reads
its answer piece by piece (newline-delimited JSON), so the desktop app shows
the reply as it is written. This supersedes ADR-0004's one-reply-per-turn,
non-streaming choice.

- **No total time limit while the model writes.** The request waits up to 10
  seconds to connect and up to 5 minutes for each next piece, which covers a
  large model being loaded into memory before its first word, but there is no
  limit on the whole answer. Before, one 120-second limit covered loading and
  the entire answer, and a 27B model on a desktop machine ran past it.
- **One code path.** `RuntimeManager` streams always; a caller that does not
  show progress passes a no-op callback. `Session::send` streams too, so the
  command line and session-note drafts get the same timeouts.
- **To the window, a Tauri channel.** `chat_send` takes a `Channel<String>`
  and sends each piece as it arrives, then returns the full reply. The
  interface shows the growing text (rendered as Markdown) in place of
  "Thinking…".
- **Errors mid-stream.** A line carrying Ollama's `error` field, or a stream
  that ends without `done`, fails the request; the half-written text is
  discarded and the message is dropped from the history, as before, so the
  person can retry it.

Reason

The maintainer's 27B security model timed out on its first answer, and the
app showed nothing for two minutes before failing. Streaming removes the
whole-answer limit and shows progress at once, which is what local models need
most; it is the norm in chat interfaces. It changes only how replies are read,
not what is sent or where it goes: still only `127.0.0.1:11434`.

Status

Accepted. Supersedes ADR-0004.

---

## ADR-0014

What the model reads. A chat's context, and search, read Markdown from:

- **all of `Base/`** and **the whole selected project folder** (`Knowledge/`,
  `Specs/`, `Resources/`, `Session/` and the project's top level), three
  levels deep: what the two default grants cover;
- **the session's temporary grants**, while they are granted: a granted
  Markdown file, or the Markdown files in a granted folder, two levels deep.
  They appear as a "granted" group. A grant inside Base or the project adds
  nothing new; a grant whose path has disappeared is skipped.

The rest of ADR-0012 applies to all of it: Markdown only, the secret block
list, the context budget with visible truncation. Because the context is
rebuilt for every message, a grant made during a session counts from the next
message; revoking it, or ending the session (which revokes temporary grants,
as since v0.3), removes it again.

One function, `context::readable_sources`, defines this scope, and both the
Context Builder and search use it, so they cannot drift apart. Grants are read
without side effects (`workspace::active_temporary_paths`): looking does not
create a permission session.

Reason

Before this, context read only the project's `Knowledge/` and `Session/`, and
temporary grants were recorded and logged but never read, while the
Permissions tab said the model reads granted folders. The maintainer granted a
router-hardening note and the agent, correctly, saw nothing. The maintainer
chose the default grants plus temporary grants as the scope, which makes the
permission model and what the model actually reads the same thing.

Status

Accepted. Amends ADR-0012 point 2 (which folders are read).

---

## ADR-0015

Project management: archive, restore, clone and delete projects from the desktop app
(spec `specs/008-project-management/`). Decided with the maintainer; built as separate
pull requests and released as 0.9.0. Runtime abstraction, the old v0.9, moves after the
stable release, to v1.1.

1. **Archive is a folder.** Archiving moves `Projects/<name>` whole to
   `Projects/Archive/<name>`; restoring moves it back. Project discovery skips
   `Archive/`, which lists separately, at the bottom of the desktop sidebar, as
   read-only projects that can be restored or deleted. `archive` becomes a reserved
   project name.
2. **Clone copies everything but the permission state.** The clone is
   `<folder>-clone` with display name `<display name> (Clone)`, a new project id
   and creation time. It copies all files, past sessions and notes included, except
   `Session/permissions.toml`, so it starts with no temporary grants.
3. **Delete goes to the Trash.** The folder moves to the freedesktop.org Trash, the
   one GNOME Files uses, through the `trash` crate; it can be restored from there,
   and GNOME can empty it after 30 days (a desktop setting, off by default). Delete
   asks for confirmation first. If the Trash cannot take the folder (some other
   drives have none), nothing is deleted and a second warning says why and offers
   to erase it permanently; only that second confirmation erases (amended with the
   maintainer, who chose this hard-delete fallback).
4. **Never overwrite.** When a destination exists, every operation refuses with
   "Already exists", writing nothing; the person can rename or clone by hand.
5. **Gate only what reads metadata.** Clone runs the validation gate. Archive,
   restore and delete move whole folders without reading them and are not gated,
   so a project with broken metadata can still be put away. An open chat session in
   the project is ended first.
6. **Desktop only.** The command line, a developer tool since ADR-0010, gets no new
   commands.
7. **Rename moves the folder too** (added with the maintainer). Renaming a project to
   "Home Security" sets its display name and moves its folder to the normalized name,
   `Projects/home-security`, keeping the two in step; a taken folder is refused with
   "Already exists". Only active projects are renamed; an archived one is restored
   first. The project's permission state names its old folder, so renaming ends it,
   as ending a chat session does: temporary grants are revoked and logged. An open
   chat session in the project is ended first. Rename is gated, like clone, since it
   reads and writes `project.toml`.

Reason

The maintainer wants to tidy the workspace without losing work: archive what is
finished, start new work from an old project, and remove what is unneeded with a
way back. Keeping archive state in the folder layout and deletes in the system Trash
keeps the filesystem the only store and every removal reversible. The `trash` crate
is local only: no network, no D-Bus, no background process.

Status

Accepted

---

## ADR-0016

What 1.0 means (spec `specs/009-stable-release/`). Decided with the maintainer.

1. **Ollama in Podman.** `workspace.toml` gains `runtime = "ollama" | "podman"` (default
   `"ollama"`) and a `[podman]` table (`image`, `gpu = "none" | "nvidia" | "amd"`). With
   `"podman"`, a session that finds nothing on `127.0.0.1:11434` starts the Ollama image as
   the container `ai-workspace-ollama`, published only on `127.0.0.1:11434`, with models in
   the named volume `ai-workspace-ollama`, started with `--pull=never`. Whatever already
   answers on that port is used instead, whatever the setting. The app never downloads the
   image or models: a missing image is reported with the `podman pull` command. CPU by
   default; `nvidia` adds the CDI device, `amd` the ROCm image and its devices, and either
   turns SELinux labelling off for that container (`--security-opt=label=disable`, as NVIDIA
   documents for Podman; amended after the 1.0 test pass found that Fedora's SELinux keeps
   Ollama from using the card otherwise, and it runs on the CPU without saying so). A container
   the app started is stopped (and removed) with the session or the app, as `ollama serve`
   is today; a container the person started is never touched. This is still Ollama, started
   differently; several runtimes remain v1.1.
2. **The workspace format is frozen for 1.x.** The layout and every file the app reads or
   writes are documented; every 1.x reads a 1.0 workspace unchanged, unknown keys are
   ignored, and an incompatible change waits for 2.0 or comes with a migration the person
   starts that keeps a copy. A 1.0 workspace is kept as a test fixture.
3. **Release gate.** 1.0 is tagged only after a written checklist passes on the current
   Fedora with real Ollama and a real model, for both runtimes, and with no issue labelled
   `bug` open.
4. **Fedora-first needs nothing new** beyond confirming the RPM in that test pass, and the
   developer command line stays as it is (ADR-0010).

Reason

The maintainer prefers containers and wants 1.0 to be a promise: notes written today stay
readable, and the release is tested against the real runtime, not only the stub. Starting
the container only on demand, binding it to localhost and never pulling keeps the
local-first and no-background-service rules.

Status

Accepted

---

## ADR-0017

Linux packages beyond Fedora (spec `specs/010-linux-packages/`). Decided with the maintainer:
start with Linux; Windows and macOS follow with their own decisions.

1. **Three Linux files per release.** Besides the Fedora `.rpm` (ADR-0010), each release has a
   `.deb` (`ai-workspace_<version>_amd64.deb`) for Ubuntu, Debian and their derivatives, and an
   AppImage (`ai-workspace-<version>-x86_64.AppImage`) for other x86_64 distributions. All three
   come from one build of one tag by Tauri's bundler, with the same package name, command and
   menu entry. `SHA256SUMS` and the signed build provenance cover each file.
2. **Supported systems.** Built on `ubuntu-24.04`, so glibc 2.39 is the floor: Fedora (current),
   Ubuntu 24.04 LTS and later, Debian 13 and later, Linux Mint 22 and later; the AppImage on
   other distributions with glibc 2.39 and FUSE 2. x86_64 only.
3. **No Flatpak, Snap or distribution repositories.** Their sandboxes would keep the app from
   reading granted folders, starting `ollama` or `podman`, and using the host's Trash; a
   repository would mean hosting and signing keys. Releases stay files on the release page.
4. **One app.** No code path depends on the distribution; the packages differ only in how they
   install.
5. **Release gate per package.** The release workflow smoke-tests each file (the `.deb` in
   Ubuntu 24.04 and Debian 13 containers, the AppImage extracted); every release's checklist
   adds a short section run on three virtual machines (Ubuntu 24.04, Debian 13 with Ollama in
   Podman, openSUSE Tumbleweed with the AppImage), next to the full checklist on Fedora.

ADR-0010's "only the desktop app is released" stands; its "one Fedora `.rpm`" becomes these
three Linux files.

Reason

The app's code is already distribution-neutral (Tauri, freedesktop Trash, Ollama on
localhost); only the packaging was Fedora-only. Native `.deb` packages reach the largest group
of Linux desktop users, and the AppImage covers the rest with one file, without new hosting or
a sandbox that would break the permission model.

Status

Proposed
