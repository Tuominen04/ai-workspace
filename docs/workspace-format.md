# Workspace format (1.0)

Every folder and file AI Workspace reads or writes, as of 1.0. The workspace is the
person's, and the filesystem is the only store, so this format is the contract: every 1.x
reads a 1.0 workspace unchanged (ADR-0016).

`crates/ai-workspace-core/tests/fixtures/workspace-1.0/` is a workspace written by 1.0
itself, and `format_tests.rs` checks that it still loads, validates, lists, reads and
searches. A failing test there means a change broke this promise; the fix is the change,
never the fixture.

## Compatibility rule

- Every 1.x reads every file described here as written by 1.0.
- New keys may be added, with defaults that keep the old behaviour. Unknown keys are
  ignored when reading, so an older 1.x keeps working with a file a newer one wrote.
- Nothing here is renamed, moved or given a new meaning within 1.x. A change that cannot
  stay compatible waits for 2.0, or comes with a migration the person starts, which keeps a
  copy of what it changes.

## Layout

```text
<workspace>/                    ~/AI unless chosen otherwise (see "User settings")
├── Base/                       notes shared by every project; Markdown read, three levels deep
├── Projects/
│   ├── <project>/              one folder per project; its name is the project's folder name
│   │   ├── project.toml        the project's metadata (the name comes from workspace.toml)
│   │   ├── README.md           the description, written at creation
│   │   ├── Knowledge/          the four folders from default_project_directories
│   │   ├── Specs/
│   │   ├── Resources/
│   │   └── Session/
│   │       ├── permissions.toml               the permission session, while there is one
│   │       └── session-note-YYYY-MM-DD-HHMMSS.md   saved notes (UTC; -2, -3… if taken)
│   └── Archive/                archived projects, each a whole project folder as above
├── Config/
│   └── workspace.toml          the workspace configuration
├── Logs/
│   └── permissions-events.log  every permission change, appended
└── Cache/                      created, not used in 1.0
```

- **Project folder names** are the project name in lowercase, with spaces turned into `-`;
  letters, digits, `-` and `_` only. Reserved: `base`, `projects`, `cache`, `logs`, `config`,
  `archive`.
- **What the model may read:** Markdown (`.md`) in `Base/` and the whole project folder,
  three levels deep, plus granted files and folders (two levels deep), never names that look
  like secrets (ADR-0012, ADR-0014). Any other file in the workspace is the person's and is
  never read.
- `Base/` gets the subfolders `Common`, `Development`, `Fedora`, `Identity` and `Linux` at
  initialization; any others the person adds are read the same way.

## Config/workspace.toml

```toml
root_path = "/home/me/AI"          # recorded at creation; the folder in use always wins
base_path = "/home/me/AI/Base"
projects_path = "/home/me/AI/Projects"
cache_path = "/home/me/AI/Cache"
logs_path = "/home/me/AI/Logs"
config_path = "/home/me/AI/Config"
default_project_directories = ["Knowledge", "Specs", "Resources", "Session"]
project_metadata_file = "project.toml"
default_model = "qwen3.6:latest"   # optional; this default if missing
runtime = "ollama"                 # optional: "ollama" (default) or "podman"

[podman]                           # optional
image = "docker.io/ollama/ollama:latest"   # optional; the default, or :rocm for gpu = "amd"
gpu = "none"                       # optional: "none" (default), "nvidia" or "amd"
```

The six `*_path` keys record where the workspace was created; the folders actually used
always follow the workspace folder in use, so a moved workspace keeps working.

## Projects/&lt;project&gt;/project.toml

```toml
project_id = "home-lab"            # the folder name
display_name = "Home Lab"          # the name shown
description = "Router and network notes"
created_at = "1795111673"          # Unix seconds, as text
knowledge_dir = "Knowledge"
specs_dir = "Specs"
resources_dir = "Resources"
session_dir = "Session"
model = "qwen3:8b"                 # optional; the workspace default_model if missing
```

## Projects/&lt;project&gt;/Session/permissions.toml

Exists while the project has a permission session; removed when it ends.

```toml
session_id = "home-lab-1795111673"
workspace_id = "home-lab"
artifact_path = "/home/me/AI/Projects/home-lab/Session/permissions.toml"
created_at = "1795111673"
# terminated_at = "…"             # optional

[[default_grants]]                 # Base and the project folder
grant_id = "default-1795111673"
session_id = "home-lab-1795111673"
canonical_path = "/home/me/AI/Base"
display_path = "/home/me/AI/Base"
category = "default"               # "default" or "temporary"
status = "active"                  # "active" or "unavailable"
granted_at = "1795111673"
source = "system-default"

[[temporary_grants]]               # zero or more, same keys, category = "temporary"
```

## Logs/permissions-events.log

Appended, never rewritten: one TOML entry per change, each followed by a line `---`.

```toml
event_id = "grant-1795111673"
session_id = "home-lab-1795111673"
event_type = "grant"               # "grant", "deny", "revoke" or "status-change"
path = "/home/me/granted"
category = "temporary"
previous_status = "active"         # optional
new_status = "active"              # optional
reason_code = "session-end"        # optional
occurred_at = "1795111673"
```

## User settings

Outside the workspace, `~/.config/ai-workspace/config.toml` (or under `$XDG_CONFIG_HOME`)
holds the one per-user setting:

```toml
workspace_root = "/home/me/ai-local/workspaces"   # optional; ~/AI if missing
```

The `AI_WORKSPACE_HOME` environment variable, when set, decides instead. The app's only other
state is the light or dark theme, kept in the window's own storage.

## Outside the workspace

- **Deleted projects** go to the desktop Trash (`~/.local/share/Trash`, or a drive's
  `.Trash-<uid>`), following the freedesktop.org Trash specification (ADR-0015).
- **Ollama in Podman** keeps its models in the Podman volume `ai-workspace-ollama` (ADR-0016).
