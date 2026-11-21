# Release checklist

Run before tagging a release, on real hardware: the current Fedora release, real Ollama and a
real model. The automated checks use a stub engine and a stand-in Podman; this is where the
real ones are tried. Required for 1.0 (ADR-0016): copy this file into the release pull
request, tick each step, and note anything that failed. A failure is filed as an issue
labelled `bug` and fixed before tagging.

Use a scratch workspace so your real notes are not touched, and write down:

- Fedora release and desktop: Fedora Linux 45 KDE Plasma (6.8.3) Desktop Edition
- Ollama version (`ollama --version`): 0.40.13
- Podman version (`podman --version`): 6.2.0
- Model used: `qwen3.6:latest`
- Graphics card used, if any: ______

## 1. Build and install the candidate

- [X] On `main` at the commit to be tagged, build the package as the release does:

  ```bash
  cd desktop && npm ci && npm run tauri build -- --ci -- --locked
  ```

- [X] Install it: `sudo dnf install ./target/release/bundle/rpm/*.rpm` (from the repository root).
- [X] **AI Workspace** is in the applications menu, and `ai-workspace` opens it.
- [X] Choose a scratch folder under **Configuration** → **Workspace folder** (create it first),
  then **Initialize** it. The Overview shows it and "Chosen by user config file".

## 2. With the installed Ollama

Ollama installed, the model pulled (`ollama pull <model>`), and Ollama **not** running **yet**
(`systemctl stop ollama` if it runs as a service).

- [X] **New project** "Release Test" with a description. It opens on **Chat**.
- [X] Put two `.md` notes in `Projects/release-test/Knowledge/`, one with a distinctive word.
  Add a file named `wifi-password.md` too.
- [X] **Overview** → **Change** the model: installed models are suggested; pick yours.
- [X] **Start session**: the app starts Ollama ("Started Ollama for this session"). The context
  list shows the notes, lists `wifi-password.md` under Skipped, and shows the budget.
- [ ] Ask about the distinctive word: the reply streams in and uses the note. Formatting
  (a list or a table) renders; a link in the reply is shown as text.
- [ ] Switch to another tab and to Whole workspace and back: the chat is still there.
- [ ] **Permissions**: grant a folder outside the workspace holding a `.md` note; the next
  message lists it under Granted and can use it. **Revoke** removes it.
- [ ] **Search** finds the distinctive word.
- [ ] Stop Ollama mid-reply (`pkill ollama`): the failure is shown with **Retry**. Start the
  session again and **Retry** works.
- [ ] **End session** → **Draft a note** → edit → **Save note and end**: a
  `session-note-…md` appears in `Session/`. A new session lists it in its context.
- [ ] Close the app: no `ollama serve` the app started is left running (`pgrep -a ollama`).

## 3. With Ollama in Podman

`ollama` stopped and, ideally, not installed. Podman installed.

- [ ] Add `runtime = "podman"` near the top of `Config/workspace.toml`. **Configuration** shows
  "Runs Ollama with: Podman container (docker.io/ollama/ollama:latest, CPU)".
- [ ] Without the image: **Start session** says the image is not downloaded and shows the
  `podman pull` command. `podman images` shows the app pulled nothing.
- [ ] Run the shown `podman pull` command, then **Start session**: "Started Ollama in the
  ai-workspace-ollama container". `podman ps` shows it, published on `127.0.0.1:11434` only.
- [ ] Pull the model into it: `podman exec ai-workspace-ollama ollama pull <model>`. It is
  suggested on the project's Overview; a chat with it works.
- [ ] **End session**: `podman ps -a` no longer shows the container. `podman volume ls` still
  shows `ai-workspace-ollama`; a new session finds the model without pulling again.
- [ ] Start a container yourself (`podman run -d --name my-ollama -p 127.0.0.1:11434:11434
  -v ai-workspace-ollama:/root/.ollama docker.io/ollama/ollama:latest`), then start and end a
  session: the app uses it and leaves it running. Remove it afterwards.
- [ ] If you have a graphics card: set `[podman]` `gpu = "nvidia"` or `"amd"` (and pull the
  `:rocm` image for AMD); replies come noticeably faster and `podman exec ai-workspace-ollama
  ollama ps` shows the GPU in use.

## 4. Project management

- [ ] **Clone** "Release Test": "Release Test (Clone)" opens, with the notes and the session
  note, and no temporary grants.
- [ ] **Rename…** the clone to "Release Copy": the dialog shows `Projects/release-copy`; after
  saving, the folder and name both changed.
- [ ] Rename it to "Release Test": refused with "Already exists"; the message can be dismissed.
- [ ] **Archive** it from the right-click menu: it moves to **Archived (1)** and to
  `Projects/Archive/`. **Restore** brings it back.
- [ ] **Delete…** it → **Move to Trash**: it is in the Files app's Trash and can be restored
  from there.

## 5. Workspace and app

- [ ] **Validation** is clean. Break `project.toml` of one project (add a stray `[`):
  Validation reports it, its Overview is refused with a pointer to Validation, and
  **Archive** still works on it. Undo the change.
- [ ] Set `AI_WORKSPACE_HOME` to another folder and start the app: Configuration says the
  variable decides and **Switch now** goes back to the saved folder.
- [ ] **Theme** System, Light and Dark all apply, and the choice survives a restart.
- [ ] Nothing reached the internet: with networking off (or watching
  `ss -tnp | grep ai-workspace`), everything above except `podman pull` works, and the app
  only connects to `127.0.0.1:11434`.
- [ ] Open an older workspace (one from 0.9, or a copy of your real one): it loads and
  validates, and its projects and notes are all there.

## 6. Upgrade and removal

- [ ] Installing over the previous release (`sudo dnf install ./…rpm`) upgrades it; the
  workspace and its settings are untouched.
- [ ] `sudo dnf remove ai-workspace` removes the app and leaves the workspace and
  `~/.config/ai-workspace/` untouched.

## Result

- [ ] Every step passed, or each failure has an issue labelled `bug` that is fixed.
- [ ] No open issues labelled `bug`.
