# Desktop app

The desktop app is what AI Workspace installs: the `ai-workspace` command opens it
(ADR-0010). It is a
Tauri 2 app on the core library (ADR-0008, ADR-0009): the Rust shell in `desktop/src-tauri`
calls the core, and the React interface in `desktop/src` shows what comes back.

## What it does

- **Sidebar** — **Whole workspace** at the top, then the projects; a dot marks the project with
  an open chat session. **New project** creates one (name and an optional description).
  Right-clicking a project (or the context-menu key) offers **Rename…**, **Clone**, **Archive**
  and **Delete…**. **Archived (N)** at the
  bottom switches the sidebar to the archived projects (`Projects/Archive/`), with **← Projects**
  to go back; an archived project shows only a read-only Overview with **Restore**, also in its
  right-click menu, which moves it back among the projects, and **Delete…** (ADR-0015).
- **Tabs** — the whole workspace shows **Overview**, **Validation**, **Search** and
  **Configuration**; a project shows **Chat** (opened first), **Overview**, **Search** and
  **Permissions**. Each tab's page is a centred column. An open chat session survives switching
  tabs and projects, including the whole workspace, until it is ended or the app closes.
- **Overview** — the workspace's folder and where that choice came from, or the chosen project's description, model and folders.
  **Change** next to the model picks the Ollama model the project chats with (installed models
  are suggested); **Use default** returns it to the workspace default. It is saved as `model`
  in the project's `project.toml`. **Archive** moves the project, unchanged, into
  `Projects/Archive/` (ending an open chat session in it first); it is offered even when the
  project's details cannot load. A name already taken there is refused with "Already exists".
  **Rename…** changes the project's name and moves its folder to match ("Home Security" becomes
  `Projects/home-security`); the dialog shows the folder before saving. An open chat session in
  the project is ended and its temporary grants end; a taken folder is refused with "Already
  exists". Archived projects are restored before renaming.
  **Clone** copies the project, files and past sessions included, into `<folder>-clone` with
  the display name "<name> (Clone)" and opens it; the clone starts with no temporary grants,
  and an existing `<folder>-clone` is refused with "Already exists". **Delete…** asks first,
  then moves the project folder to the desktop Trash, where the Files app can restore it (GNOME
  can empty the Trash after 30 days: Settings → Privacy & Security → File History & Trash). If
  the Trash cannot take the folder, a second warning offers to erase it permanently; Cancel
  keeps it. An open chat session in the project is ended first.
- **Validation** — a validation run, with how to clear each issue. Where the fix is to
  initialize the workspace, a button does it.
- **Search** — ranked results from Base knowledge and the chosen project's Knowledge and
  Session folders.
- **Chat** — a session in the chosen project: opening it grants the project's default
  permissions and builds the first context; each message rebuilds the context from what was
  asked; ending it revokes temporary permissions. Quitting the app ends an open session too.
  Replies appear as they are written (streamed from Ollama, so a long answer from a large model
  never times out, ADR-0013). If a reply fails, the question stays and a **Retry** button sends
  it again, so the model gets the real question rather than a follow-up like "continue". Replies are shown as formatted Markdown (headings, lists, emphasis, code, tables); raw HTML in
  a reply is never rendered, and links and images are shown as text, never followed or loaded,
  so the window stays on this computer. The chat names the model it talks to (the project's `model` in `project.toml`, else the
  workspace default) and whether the model's own instructions are kept: a model built from a
  Modelfile with a `SYSTEM` prompt, such as a security advisor, keeps those rules, and the
  workspace context follows them (ADR-0012). Files whose names look like secrets (such as
  `wifi-password.md` or anything in a `secrets/` folder) are never read; the context list shows
  them under "Skipped". The context list also shows how much text fits (from the model's
  context window) and any file that was cut to fit, which the model sees marked as truncated.
  **End session** after at least one answer offers a note for next time: **Draft a note** asks
  the model for a short Markdown summary, which you can edit, then **Save note and end** writes
  it as `session-note-YYYY-MM-DD-HHMMSS.md` (UTC) in the project's Session folder, where later
  sessions read it as context. **End without a note** writes nothing. Closing the window while a chat with answers is open asks the same question first ("AI Workspace
  is closing"): save a note and close, close without a note, or keep chatting.
- **Permissions** — what the model may read, beyond Base and the project folder, and the
  chosen project's grants: the defaults (Base and the project) and any
  temporary ones, with their status, a filter and pages. **Grant** gives temporary access to
  a path you type or paste; **Revoke** takes a temporary grant away. A granted Markdown file, or
  the Markdown files in a granted folder (two levels deep), join the chat's context from the next
  message, listed as "Granted" (ADR-0014). Ending the chat session revokes temporary grants. Paths that do not exist
  are refused, and every change is written to `Logs/permissions-events.log`.
- **Configuration** — the workspace configuration (`Config/workspace.toml`): the folder in
  use and why, the configuration file, the default model, the project file name and the
  folders new projects get. **Workspace folder** saves another folder in
  `~/.config/ai-workspace/config.toml` (an open chat session is ended first). If
  `AI_WORKSPACE_HOME` is set it still takes precedence; the app says so and offers
  **Switch now**, which ignores the variable until the app restarts. When the
  configuration file is missing, **Initialize** creates the standard layout and defaults.
- **Theme** — System, Light or Dark, at the bottom of the sidebar.

Project details, search and permission changes are refused while validation reports blocking
issues, as on the command line.

The app finds the workspace as follows: `AI_WORKSPACE_HOME`, else `workspace_root` in
`~/.config/ai-workspace/config.toml`, else `~/AI`.

## Installing

Releases ship it as the `ai-workspace` package; see the [README](../README.md#install).
It installs `/usr/bin/ai-workspace`, an **AI Workspace** entry in the applications menu,
an icon and the two license texts. `dnf` pulls in WebKitGTK and GTK.

## Running it from source

Once, on Fedora:

```bash
sudo dnf install webkit2gtk4.1-devel librsvg2-devel libappindicator-gtk3-devel nodejs
```

Then, from `desktop/`:

```bash
npm ci
npm run tauri dev
```

To try it against a scratch workspace instead of your own, set `AI_WORKSPACE_HOME` first.

## Checks

From `desktop/`:

```bash
npm run lint && npm run typecheck && npm test && npm run build   # the interface
npm run bindings                                                 # shell tests; regenerates src/lib/bindings/
npm run e2e                                                      # the interface in Chromium with a stub engine
```

`npm run e2e` needs a Chromium: `npx playwright install chromium` once, or point
`AIW_E2E_CHROMIUM` at one you have. A single check can be run against `npm run dev` with a
folder for screenshots: `URL=http://localhost:5174/ node e2e/checks/workspace.mjs /tmp/shots`.

None of the checks need Ollama or a real workspace. The shell's tests use temporary
workspaces; the browser checks answer every command from a stub.

## Where things live

| Path | What |
| --- | --- |
| `desktop/src-tauri/src/commands.rs` | The commands the interface can call, each one core call |
| `desktop/src-tauri/src/dto.rs` | The wire types; TypeScript is generated from them |
| `desktop/src/lib/engine.ts` | The interface's only bridge to the shell |
| `desktop/src/lib/bindings/` | Generated; never edit by hand |
| `desktop/src/components/` | The sidebar and the tabs |
| `desktop/e2e/` | Browser checks and their stub engine |

The interface keeps one thing of its own: the theme choice, in the webview's local storage.
It is a display preference of this computer, not workspace state.
