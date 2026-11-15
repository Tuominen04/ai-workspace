# Desktop app

The desktop app is a window onto the same workspace the `ai-workspace` command uses. It is a
Tauri 2 app on the core library (ADR-0008, ADR-0009): the Rust shell in `desktop/src-tauri`
calls the core, and the React interface in `desktop/src` shows what comes back.

## What it does

The first version is read-only plus chat:

- **Sidebar** — the workspace folder, where that choice came from, and the projects. Choosing
  a project scopes the other tabs to it.
- **Overview** — the workspace's folders, or the chosen project's description, model and folders.
- **Validation** — a validation run, with how to clear each issue.
- **Search** — ranked results from Base knowledge and the chosen project's Knowledge and
  Session folders.
- **Configuration** — the workspace configuration (`Config/workspace.toml`): the folder in
  use and why, the configuration file, the default model, the project file name and the
  folders new projects get. Read-only; edit the file or use the command line to change it.
- **Chat** — a session in the chosen project, like `ai-workspace` with no arguments: opening it
  grants the project's default permissions and builds the first context; each message rebuilds
  the context from what was asked; ending it revokes temporary permissions. Quitting the app
  ends an open session too.
- **Theme** — System, Light or Dark, at the bottom of the sidebar.

Creating projects and granting permissions are still done with the command line.

The app finds the workspace exactly as the command does: `AI_WORKSPACE_HOME`, else the folder
saved by `ai-workspace config set-root`, else `~/AI`.

## Installing

Each release has a second package, `ai-workspace-desktop-<version>.x86_64.rpm`:

```bash
sudo dnf install ./ai-workspace-desktop-<version>.x86_64.rpm
```

It installs `/usr/bin/ai-workspace-desktop`, an **AI Workspace Desktop** entry in the
applications menu, an icon and the two license texts. `dnf` pulls in WebKitGTK and GTK.
Removing it (`sudo dnf remove ai-workspace-desktop`) leaves the workspace untouched.

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
