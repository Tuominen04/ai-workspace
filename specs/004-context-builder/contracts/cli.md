# CLI Contract: Session Context Surface

## Overview

The `ai` session startup flow must expose a clear context summary before the runtime is initialized.

## Inputs

- Selected workspace project from the existing project-selection flow.
- Workspace root resolved from `AI_WORKSPACE_HOME` or the default `~/AI` location.

## Outputs

- Standard output includes the selected project, the number of base sources loaded, and the number of project sources loaded.
- The command exits successfully when context assembly completes for the available sources.
- The command returns a non-zero exit code when the workspace root or selected project path is invalid or unreadable.

## Behavioral Rules

- Base context is shown whenever base knowledge exists.
- Project context is shown only when a project is selected.
- Unavailable sources are skipped and do not break session startup.
- The context summary must clearly distinguish shared base knowledge from project-specific knowledge.
