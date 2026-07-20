# Data Model: AI CLI Foundation v0.1

## Entity: WorkspaceRoot
- Description: Top-level user-owned directory for AI Workspace runtime and knowledge layout.
- Fields:
  - rootPath: absolute path
  - basePath: absolute path (~/AI/Base)
  - projectsPath: absolute path
  - cachePath: absolute path
  - logsPath: absolute path
  - configPath: absolute path
- Validation rules:
  - rootPath must be resolvable on local filesystem.
  - basePath must exist or be creatable.
  - projectsPath must exist or be creatable.

## Entity: ProjectWorkspace
- Description: A discovered project directory under Projects used for user session targeting.
- Fields:
  - name: directory name
  - path: absolute path
- Validation rules:
  - name must be non-empty.
  - path must be a directory.

## Entity: ContextSnapshot
- Description: Base-only AI-readable context file inventory for current session.
- Fields:
  - baseFiles: list of relative markdown file paths discovered under Base
- Validation rules:
  - Include only files under Base root.
  - File type limited to markdown for v0.1.

## Entity: RuntimeStatus
- Description: Session-visible local runtime state.
- Fields:
  - running: boolean
  - startedHere: boolean
  - detail: human-readable status message
- Validation rules:
  - running true with startedHere false means pre-existing runtime reused.
  - running false must include actionable detail.

## Entity: SessionState
- Description: Active CLI workspace session context.
- Fields:
  - selectedProject: ProjectWorkspace
  - contextSnapshot: ContextSnapshot
  - runtimeStatus: RuntimeStatus
  - active: boolean
- State transitions:
  - initialized -> project_selected -> context_built -> runtime_checked -> active -> closed
  - initialized -> failed (for unrecoverable startup errors)
