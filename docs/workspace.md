# Workspace Notes

## v0.2 Workspace Manager Overview

Workspace manager behavior is filesystem-first and user-visible:

- Workspace configuration lives in `Config/workspace.toml`.
- Project metadata lives in `Projects/<project>/project.toml`.
- Users can maintain these files directly; v0.2 command surfaces inspect and
  validate definitions rather than editing them.

## Command Surface

```text
ai-workspace config show
ai-workspace project create --name NAME [--description TEXT]
ai-workspace project show PROJECT
ai-workspace validate
```

## Safety Gate

Validation uses an enforcement gate for safety:

- `ai-workspace validate` is always available and reports blocking issues.
- `ai-workspace project create` and `ai-workspace project show` run pre-action checks and refuse to
  continue when blocking issues are present.
- After fixing issues, run `ai-workspace validate` again, then retry gated commands.
