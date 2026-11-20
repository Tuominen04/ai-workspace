# User guide

Everything AI Workspace does, for the person using it. To install it, see the
[README](../README.md#quick-start).

- [The workspace](#the-workspace)
- [The window](#the-window)
- [Projects](#projects)
- [Models](#models)
- [Running Ollama in Podman](#running-ollama-in-podman)
- [Chat](#chat)
- [What the model reads](#what-the-model-reads)
- [Permissions](#permissions)
- [Search](#search)
- [Archive, clone, rename and delete](#archive-clone-rename-and-delete)
- [Validation](#validation)
- [Choosing the workspace folder](#choosing-the-workspace-folder)
- [When something goes wrong](#when-something-goes-wrong)
- [Privacy](#privacy)

## The workspace

Everything lives in one folder, `~/AI` unless you choose another:

```text
~/AI/
├── Base/        notes every project shares (about you, your systems, your habits)
├── Projects/    one folder per project
│   ├── home-lab/
│   │   ├── Knowledge/   notes about the project
│   │   ├── Specs/       plans and requirements
│   │   ├── Resources/   reference material
│   │   ├── Session/     notes saved after chats
│   │   ├── README.md    the project's description
│   │   └── project.toml its name, description and model
│   └── Archive/  archived projects
├── Config/      workspace.toml: the default model and new-project folders
├── Logs/        a record of every permission change
└── Cache/
```

These are ordinary folders and files. Edit notes with any editor, copy them, back them up;
the app picks up changes the next time it reads them. The app keeps nothing of its own
elsewhere, except which folder to use (see
[Choosing the workspace folder](#choosing-the-workspace-folder)) and your light or dark theme.

## The window

The sidebar lists **Whole workspace** and your projects. What you select decides the tabs:

| Selected | Tabs |
|---|---|
| Whole workspace | **Overview** (the folder and where it was chosen), **Validation**, **Search**, **Configuration** |
| A project | **Chat** (opened first), **Overview**, **Search**, **Permissions** |

Right-click a project for **Rename…**, **Clone**, **Archive** and **Delete…**. A green dot
marks the project with an open chat. **Archived (N)** at the bottom shows archived projects;
**Theme** switches between your system's colours, light and dark.

## Projects

**New project** in the sidebar asks for a name and an optional description. The name becomes
the folder: "Home Lab" is `Projects/home-lab`. Names may use letters, digits, spaces, `-` and
`_`; a few are reserved (`Base`, `Projects`, `Cache`, `Logs`, `Config`, `Archive`).

Put Markdown notes (`.md`) anywhere in the project folder. Subfolders are fine, up to three
levels deep.

## Models

The model is the AI that answers. It runs in [Ollama](https://ollama.com); AI Workspace never
downloads or changes models. Download one with Ollama first:

```bash
ollama pull qwen3:8b
```

- **A project's model:** on the project's **Overview**, **Change** and type or pick a model;
  the models Ollama has installed are suggested. **Use default** goes back to the workspace
  default. It is saved in the project's `project.toml`.
- **The workspace default:** shown in **Configuration**, set as `default_model` in
  `Config/workspace.toml`.

A model you built with your own instructions (an Ollama Modelfile with a `SYSTEM` prompt, such
as a security advisor) keeps them: the chat shows "the model's own instructions are kept", and
your notes are added after its rules. See Ollama's
[Modelfile reference](https://docs.ollama.com/modelfile) to make one.

## Running Ollama in Podman

Instead of installing Ollama, you can run it in a Podman container; AI Workspace then starts
the container when a chat needs it and stops it when the app closes. The app never downloads
anything itself, so you fetch the image once:

1. Get the image (Podman comes with Fedora Workstation):

   ```bash
   podman pull docker.io/ollama/ollama:latest
   ```

2. Open your workspace's configuration file: `Config/workspace.toml` inside your workspace
   folder (`~/AI/Config/workspace.toml` unless you chose another folder; **Configuration** in
   the app shows the exact path as "Configuration file"). Add this line near the top, above
   any `[table]`:

   ```toml
   runtime = "podman"
   ```

3. Start a chat session. **Configuration** shows "Runs Ollama with: Podman container".
4. While the session is open, download a model into the container:

   ```bash
   podman exec ai-workspace-ollama ollama pull qwen3:8b
   ```

   Models are kept in the Podman volume `ai-workspace-ollama`, so they survive the container
   being stopped. Then choose the model on the project's **Overview** (or as `default_model`).

The container has its own models: those of an installed Ollama are not in it, so download
them again into the container.

The container is reachable only from your own computer (`127.0.0.1:11434`). If an Ollama is
already running there (one you started yourself, in a container or not), the app uses it and
starts nothing. So stop an installed Ollama first (`sudo systemctl stop ollama`), and check
that nothing listens on the port (`ss -tlnp | grep 11434` prints nothing); otherwise the app
quietly uses that one instead.

**Your own models (Modelfiles):** `ollama create` run inside the container cannot see files in
your home folder. While a session has the container running, either use an installed `ollama`
command, which reads the Modelfile on your computer and sends it to the container:

```bash
ollama create sec -f ~/path/to/sec.Modelfile
```

or copy the file in first:

```bash
podman cp ~/path/to/sec.Modelfile ai-workspace-ollama:/tmp/sec.Modelfile
podman exec ai-workspace-ollama ollama create sec -f /tmp/sec.Modelfile
```

The Modelfile's `FROM` line must name a model the container already has, written exactly as
`ollama list` shows it (for example `qwen3:8b`, with a colon); otherwise Ollama tries to
download it and fails with "file does not exist". Check with
`podman exec ai-workspace-ollama ollama list`.

**Graphics card:** add a `[podman]` table at the end of `workspace.toml`:

```toml
[podman]
gpu = "nvidia"   # or "amd"
```

NVIDIA needs the NVIDIA Container Toolkit set up for Podman; AMD uses Ollama's ROCm image,
which you pull instead with `podman pull docker.io/ollama/ollama:rocm`. Ollama's
[container page](https://docs.ollama.com/docker) describes both. To use another image, set
`image = "…"` in the same table.

## Chat

On a project's **Chat**, **Start session**. AI Workspace starts Ollama if it is not running.

- **The context list** above the conversation shows which notes the model was given, which
  were left out because their names look like secrets, and any note cut short to fit (the model
  sees that it was cut).
- **Replies** appear as they are written and are formatted (headings, lists, tables, code).
  Links and images in a reply are shown as text and never opened, so nothing reaches the
  internet.
- **Retry:** if a reply fails, your question stays and **Retry** sends it again.
- **Switching** tabs or projects keeps the chat; it stays open until you end it or close the
  app.
- **End session:** after at least one answer, you can **Draft a note**. The model writes a
  short summary, you edit it, and **Save note and end** saves it in the project's `Session`
  folder, where the next chat reads it. **End without a note** saves nothing.

Ending a chat also ends any temporary permissions you gave during it.

## What the model reads

The model never opens files itself; AI Workspace gives it a selection with each message:

- Markdown notes in `Base` and in the selected project (its whole folder);
- notes you granted for this chat (see [Permissions](#permissions)).

Only `.md` files are read. Names that look like secrets are never read, even when granted:
anything with `password`, `secret` or `token` in its name, key files such as `id_rsa` or
`*.pem`, `.env` files, and anything in a folder with such a name. Rename a note if it is
skipped by mistake.

How much fits depends on the model's context window. The most relevant notes for your question
come first; a note that does not fit whole is cut and marked.

The conversation itself is sent with every message too. When a long chat no longer fits, the
model gets the latest messages and the oldest ones are left out; the chat says so when that
happens. For a fresh start with what matters kept, end the session with a note and start a new
one.

## Permissions

Beyond `Base` and the project, the model reads only what you grant. On a project's
**Permissions**, paste the path of a file or folder and **Grant**. Markdown in it joins the
chat from the next message (a folder's notes up to two levels deep). **Revoke** takes a grant
away, and ending the chat removes all temporary grants. Every change is recorded in
`Logs/permissions-events.log`.

## Search

**Search** finds notes by keywords: in the whole workspace's Base notes, or with a project
selected, also in that project and its granted notes. It searches exactly what the model may
read.

## Archive, clone, rename and delete

All four are in a project's right-click menu and on its **Overview**.

- **Archive** moves the project, unchanged, to `Projects/Archive/`. Find it under
  **Archived (N)** and **Restore** it when you need it again.
- **Clone** copies the project, notes and past sessions included, into a new project such as
  "Home Lab (Clone)". The copy starts without temporary permissions.
- **Rename…** changes the name and the folder together; the dialog shows the new folder before
  you save. Archived projects are restored before renaming.
- **Delete…** asks first, then moves the project to the Trash, where the Files app can restore
  it. GNOME can empty the Trash after 30 days: Settings → Privacy & Security → File History &
  Trash. If a drive has no Trash, a second warning offers to erase the project permanently.

An open chat in the project is ended first. When a name is already taken, the app says
"Already exists" and changes nothing.

## Validation

**Validation** (on Whole workspace) checks the workspace's folders and each project's
`project.toml`, and says how to fix each problem. While a serious problem remains, actions
that read project details, such as opening a project's Overview, searching, cloning or
renaming, are refused until it is fixed. Archiving and deleting still work, so a broken project
can be put away.

## Choosing the workspace folder

To use another folder than `~/AI`, create it and choose it under **Configuration** →
**Workspace folder**. An open chat is ended first. The choice is saved in
`~/.config/ai-workspace/config.toml`:

```toml
workspace_root = "/home/you/ai-local/workspaces"
```

If the `AI_WORKSPACE_HOME` environment variable is set, it decides instead; Configuration says
so and offers **Switch now** to use your saved folder until the app restarts. Delete the config
file to go back to `~/AI`. If the folder has no workspace yet, **Initialize** creates the
standard folders.

## When something goes wrong

| What you see | What to do |
|---|---|
| "No local runtime is running" | Install Ollama (see the [README](../README.md#quick-start)), or set up [Ollama in Podman](#running-ollama-in-podman), and start the session again. |
| "The Ollama image … is not downloaded" | Run the `podman pull` command it shows, then start the session again. |
| The chat uses an Ollama you did not expect | Something already answers on `127.0.0.1:11434` (an installed Ollama, or a container you started); stop it, then start the session again. |
| "The model '…' is not installed in Ollama" | Run the download command it shows (`ollama pull …`, or `podman exec ai-workspace-ollama ollama pull …` with Ollama in Podman), or choose an installed model on the project's Overview. |
| Replies are very slow | Use a smaller model, or set Ollama up to use your graphics card (Ollama's [GPU page](https://docs.ollama.com/gpu)). |
| A reply stopped halfway | **Retry**. |
| A note is missing from the context list | It may not be `.md`, its name may look like a secret, or it did not fit: ask about it by name, or use a model with a larger context window. |
| "…validation has blocking issues" | Open **Validation** on Whole workspace and fix what it lists. |
| "Already exists" | Pick another name, or rename or archive the other project first. |

## Privacy

AI Workspace connects only to Ollama on your own computer (`127.0.0.1:11434`). It sends no
telemetry, needs no account, and changes files only when you ask: creating, saving a note,
granting, archiving, cloning, renaming or deleting. Removing the app
(`sudo dnf remove ai-workspace`) leaves your workspace untouched.
