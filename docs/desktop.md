# Desktop app

The desktop app is what AI Workspace installs: the `ai-workspace` command opens it
(ADR-0010). It is a
Tauri 2 app on the core library (ADR-0008, ADR-0009): the Rust shell in `desktop/src-tauri`
calls the core, and the React interface in `desktop/src` shows what comes back.

## What it does

- **Sidebar** — the workspace folder, where that choice came from, and the projects. Choosing
  a project scopes the other tabs to it. **New project** creates one (name and an optional
  description).
- **Overview** — the workspace's folders, or the chosen project's description, model and folders.
- **Validation** — a validation run, with how to clear each issue. Where the fix is to
  initialize the workspace, a button does it.
- **Search** — ranked results from Base knowledge and the chosen project's Knowledge and
  Session folders.
- **Chat** — a session in the chosen project: opening it grants the project's default
  permissions and builds the first context; each message rebuilds the context from what was
  asked; ending it revokes temporary permissions. Quitting the app ends an open session too.
- **Permissions** — the chosen project's grants: the defaults (Base and the project) and any
  temporary ones, with their status, a filter and pages. **Grant** gives temporary access to
  a path you type or paste; **Revoke** takes a temporary grant away. Paths that do not exist
  are refused, and every change is written to `Logs/permissions-events.log`.
- **Configuration** — the workspace configuration (`Config/workspace.toml`): the folder in
  use and why, the configuration file, the default model, the project file name and the
  folders new projects get. **Workspace folder** saves another folder in
  `~/.config/ai-workspace/config.toml` (an open chat session is ended first); if
  `AI_WORKSPACE_HOME` is set it still takes precedence, and the app says so. When the
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
