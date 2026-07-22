# Data Model: Workspace Manager v0.2

## Entity: WorkspaceConfiguration

- Description: User-visible workspace-wide configuration stored under the managed AI
  workspace root.
- Fields:
  - rootPath: absolute path
  - basePath: absolute path
  - projectsPath: absolute path
  - cachePath: absolute path
  - logsPath: absolute path
  - configPath: absolute path
  - defaultProjectDirectories: ordered list of directory names
  - projectMetadataFile: file name for per-project metadata
- Validation rules:
  - All configured paths must remain inside the workspace root.
  - `projectsPath` and `configPath` must exist or be creatable by workspace-init and
    project-creation flows.
  - `defaultProjectDirectories` must include the minimum project structure required by
    the architecture.
  - `projectMetadataFile` must be a relative file name, not a path outside a project.

## Entity: ProjectMetadataRecord

- Description: User-visible metadata file describing one managed project.
- Fields:
  - projectId: stable directory-safe identifier
  - displayName: human-readable project name
  - description: human-readable description
  - createdAt: creation date or timestamp string
  - knowledgeDir: relative path
  - specsDir: relative path
  - resourcesDir: relative path
  - sessionDir: relative path
- Validation rules:
  - `projectId` must match the project directory name.
  - `displayName` must be non-empty.
  - Required relative directory fields must resolve within the project root.
  - Metadata must be parseable from TOML and contain all required fields.

## Entity: ProjectWorkspace

- Description: Isolated filesystem workspace for one managed project.
- Fields:
  - rootPath: absolute path
  - metadataPath: absolute path
  - readmePath: absolute path
  - knowledgePath: absolute path
  - specsPath: absolute path
  - resourcesPath: absolute path
  - sessionPath: absolute path
- Validation rules:
  - `rootPath` must be a direct child of the configured projects directory.
  - Required files and directories must exist after successful project creation.
  - Project contents must not rely on files outside the project root for required
    metadata.
- State transitions:
  - requested -> validated -> created
  - requested -> rejected
  - created -> invalid (when later validation detects a blocking issue)

## Entity: ValidationIssue

- Description: One actionable finding produced by workspace validation.
- Fields:
  - severity: error or warning
  - scope: workspace or project
  - subject: workspace root or project identifier
  - path: affected filesystem path
  - code: stable issue code
  - message: human-readable problem statement
  - suggestedFix: human-readable corrective action
- Validation rules:
  - Error severity blocks readiness.
  - Each issue must identify a concrete subject and path.
  - Message and suggested fix must be understandable without source-code knowledge.

## Entity: ValidationResult

- Description: Aggregate readiness outcome for a validation run.
- Fields:
  - valid: boolean
  - workspaceIssues: list of ValidationIssue
  - projectIssues: map of project identifier to list of ValidationIssue
  - checkedProjects: count
- Validation rules:
  - `valid` is true only when no error-severity issues are present.
  - Results must not imply that a project was modified as part of validation.
