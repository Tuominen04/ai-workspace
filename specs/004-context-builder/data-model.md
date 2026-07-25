# Data Model: v0.4 Context Builder

## Entity: ContextPackage

- **id**: Unique identifier for the assembled context package.
- **project_name**: Name of the selected project, if one is active.
- **base_sources**: Ordered list of base knowledge sources included in the context.
- **project_sources**: Ordered list of project knowledge sources included in the context.
- **session_sources**: Ordered list of current session resources that are included.
- **assembled_prompt**: The final prompt-ready content that will be sent to the runtime.
- **generated_at**: Timestamp for the assembly operation.
- **source_count**: Total number of included sources.
- **total_bytes**: Total size of the assembled content.

## Entity: KnowledgeSource

- **path**: Filesystem path of the source document.
- **kind**: One of `base`, `project`, or `session`.
- **title**: Display name derived from the source path.
- **bytes**: Size of the source content.
- **included**: Whether the source was selected for the current context package.

## Entity: ContextSelectionPolicy

- **max_sources**: Maximum number of sources to include in one context package.
- **max_bytes**: Maximum total payload size for the assembled prompt.
- **ordering**: Deterministic ordering by path.

## Relationships

- One active session produces one `ContextPackage`.
- A `ContextPackage` contains zero or more `KnowledgeSource` entries.
- `ContextPackage` is derived from one selected project and zero or more shared base sources.

## Validation Rules

- Base knowledge is included by default when it exists.
- Project knowledge is included only for the currently selected project.
- Missing or unreadable sources are skipped without breaking the package.
- The assembled package must remain within the configured size bounds.
