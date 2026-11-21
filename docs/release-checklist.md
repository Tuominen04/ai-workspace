# Release checklist

Run before tagging a release, on real hardware: the current Fedora release, real Ollama and a real model. The automated checks use a stub engine and a stand-in Podman; this is where the real ones are tried. Required for 1.0 (ADR-0016): copy this file into the release pull request, tick each step, and note anything that failed. A failure is filed as an issue labelled `bug` and fixed before tagging.

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

- [X] Install it, from the repository root:

  ```bash
  sudo dnf reinstall ./target/release/bundle/rpm/*.rpm
  ```

  Use `reinstall` while the candidate has the same version as the installed release (the version is only bumped in the release pull request, after this checklist): `dnf install` skips a package of a version that is already installed and leaves the old build in place. Use `sudo dnf install` instead if AI Workspace is not installed at all.
- [X] The candidate is what runs: `rpm -qi ai-workspace` shows today's **Build Date**.
- [X] **AI Workspace** is in the applications menu, and `ai-workspace` opens it.
- [X] Choose a scratch folder under **Configuration** → **Workspace folder** (create it first), then **Initialize** it. The Overview shows it and "Chosen by user config file".

## 2. With the installed Ollama

Ollama installed and running as its service, as its installer sets it up (`systemctl status ollama` shows it active), and the model pulled (`ollama pull <model>`).

- [X] **New project** "Release Test" with a description. It opens on **Chat**.
- [X] Put two `.md` notes in `Projects/release-test/Knowledge/`, one with a distinctive word. Add a file named `wifi-password.md` too.
- [X] **Overview** → **Change** the model: installed models are suggested; pick yours (the workspace default may be a model you have not pulled).
- [X] **Start session**: the app uses the running service ("Ollama is already running on 127.0.0.1:11434"). The context list shows the notes, lists `wifi-password.md` under. Skipped, and shows the budget.
- [X] Ask about the distinctive word: the reply streams in and uses the note. Formatting (a list or a table) renders; a link in the reply is shown as text.
- [X] Switch to another tab and to Whole workspace and back: the chat is still there.
- [X] **Permissions**: grant a folder outside the workspace holding a `.md` note with two facts in it. Ask about the first: the next reply uses it, without restarting the session, and the context list shows the note under Granted. **Revoke** it, then ask about the second fact, one the model has not said yet: it no longer knows it, and the list leaves the note out. (What the model already said stays in the conversation, so it can repeat the first fact.)
- [X] **Search** finds the distinctive word.
- [X] Stop the service mid-reply (`sudo systemctl stop ollama`): the failure is shown with **Retry**.
- [X] With the service still stopped, **End session** and **Start session**: the app starts its own Ollama ("Started Ollama for this session"). It runs as you and keeps models in `~/.ollama/models`, not the service's: **Overview** → **Change** lists no models, and the **?** at the end of its hint explains why. Run `ollama pull <model>` while the session is open: **Retry** now works.
- [X] **End session** → **Draft a note** → edit → **Save note and end**: a
  `session-note-…md` appears in `Session/`. A new session lists it in its context.
- [X] Close the app: no `ollama serve` the app started is left running (`pgrep -a ollama`). Start the service again (`sudo systemctl start ollama`).

## 3. With Ollama in Podman

Podman installed, and no other Ollama answering on the port: stop the service
(`sudo systemctl stop ollama`) and any `ollama serve` (`pkill ollama`), then check that
`ss -tlnp | grep 11434` prints nothing. Otherwise the app uses that Ollama and says so ("Using
an Ollama already running on 127.0.0.1:11434, not the app's Podman container").

- [X] Add `runtime = "podman"` near the top of `Config/workspace.toml`. **Configuration** shows "Runs Ollama with: Podman container (docker.io/ollama/ollama:latest, CPU)".
- [X] Without the image (remove it first: **End session**, then `podman rmi docker.io/ollama/ollama:latest`; the models stay in the volume): **Start session** says the image is not downloaded and shows the app pulled nothing.
- [X] Run the shown `podman pull docker.io/ollama/ollama:latest` command, then **Start session**: "Started Ollama in the ai-workspace-ollama container". `podman ps` shows it, published on `127.0.0.1:11434` only.
- [X] Pull the model into it: `podman exec ai-workspace-ollama ollama pull <model>`. It is suggested on the project's Overview; a chat with it works.
- [X] **End session**: `podman ps -a` no longer shows the container. `podman volume ls` still shows `ai-workspace-ollama`; a new session finds the model without pulling again.
- [X] Start a container yourself (`podman run -d --name my-ollama -p 127.0.0.1:11434:11434 -v ai-workspace-ollama:/root/.ollama docker.io/ollama/ollama:latest`), then start and end a session: the app uses it and leaves it running. Remove it afterwards.
- [ ] If you have a graphics card: set `[podman]` `gpu = "nvidia"` or `"amd"` (and pull the `:rocm` image for AMD); replies come noticeably faster and `podman exec ai-workspace-ollama ollama ps` shows the GPU in use.

## 4. Project management

- [ ] **Clone** "Release Test": "Release Test (Clone)" opens, with the notes and the session note, and no temporary grants.
- [ ] **Rename…** the clone to "Release Copy": the dialog shows `Projects/release-copy`; after saving, the folder and name both changed.
- [ ] Rename it to "Release Test": refused with "Already exists"; the message can be dismissed.
- [ ] **Archive** it from the right-click menu: it moves to **Archived (1)** and to `Projects/Archive/`. **Restore** brings it back.
- [ ] **Delete…** it → **Move to Trash**: it is in the Files app's Trash and can be restored from there.

## 5. Workspace and app

- [ ] **Validation** is clean. Break `project.toml` of one project (add a stray `[`): Validation reports it, its Overview is refused with a pointer to Validation, and **Archive** still works on it. Undo the change.
- [ ] Set `AI_WORKSPACE_HOME` to another folder and start the app: Configuration says the variable decides and **Switch now** goes back to the saved folder.
- [ ] **Theme** System, Light and Dark all apply, and the choice survives a restart.
- [ ] Nothing reached the internet: with networking off (or watching `ss -tnp | grep ai-workspace`), everything above except `podman pull` works, and the app only connects to `127.0.0.1:11434`.
- [ ] Open an older workspace (one from 0.9, or a copy of your real one): it loads and validates, and its projects and notes are all there.

## 6. Upgrade and removal

- [ ] Installing over the previous release (`sudo dnf install ./…rpm`) upgrades it; the workspace and its settings are untouched.
- [ ] `sudo dnf remove ai-workspace` removes the app and leaves the workspace and `~/.config/ai-workspace/` untouched.

## Result

- [ ] Every step passed, or each failure has an issue labelled `bug` that is fixed.
- [ ] No open issues labelled `bug`.
