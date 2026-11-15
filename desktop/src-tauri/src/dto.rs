//! The wire types: what crosses from the shell to the interface.
//!
//! The core's own types are not sent directly. The TypeScript the interface is written
//! against is **generated from the types in this file** (`cargo test -p ai-workspace-desktop`
//! writes `desktop/src/lib/bindings/`), and CI fails when the committed copy is stale, so the
//! boundary has one definition rather than two that drift. Counts are `u32` so they arrive as
//! plain numbers.

use ai_workspace_core::AppError;
use ai_workspace_core::context::ContextSnapshot;
use ai_workspace_core::paths::{RootSource, WorkspacePaths};
use ai_workspace_core::runtime::RuntimeStatus;
use ai_workspace_core::search::SearchOutcome;
use ai_workspace_core::workspace::{
    Action, Project, ProjectMetadata, SuggestedFix, ValidationIssue, ValidationResult,
    ValidationSeverity,
};
use serde::Serialize;
use std::path::Path;
use ts_rs::TS;

fn path_text(path: &Path) -> String {
    path.display().to_string()
}

fn count(n: usize) -> u32 {
    u32::try_from(n).unwrap_or(u32::MAX)
}

/// A failed command. `kind` is the core error's variant name, for the interface to branch on;
/// `message` is the core's own, front-end-neutral text.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../src/lib/bindings/")]
pub struct CommandError {
    pub kind: String,
    pub message: String,
}

impl From<AppError> for CommandError {
    fn from(error: AppError) -> Self {
        let kind = match &error {
            AppError::Io(_) => "Io",
            AppError::Message(_) => "Message",
            AppError::MissingHome => "MissingHome",
            AppError::InvalidSelection(_) => "InvalidSelection",
            AppError::ConfigMissing(_) => "ConfigMissing",
            AppError::ConfigInvalid { .. } => "ConfigInvalid",
            AppError::InvalidProjectName(_) => "InvalidProjectName",
            AppError::ProjectNotFound(_) => "ProjectNotFound",
            AppError::ValidationGateBlocked { .. } => "ValidationGateBlocked",
            AppError::PermissionDenied(_) => "PermissionDenied",
            AppError::InvalidPermissionPath(_) => "InvalidPermissionPath",
            AppError::PermissionGrantNotFound(_) => "PermissionGrantNotFound",
            AppError::MissingProjectContext => "MissingProjectContext",
            AppError::RuntimeRequestFailed(_) => "RuntimeRequestFailed",
            AppError::InvalidWorkspaceRoot { .. } => "InvalidWorkspaceRoot",
        };
        Self {
            kind: kind.to_string(),
            message: error.to_string(),
        }
    }
}

impl CommandError {
    /// A failure in the shell itself rather than in the core.
    pub fn shell(message: impl Into<String>) -> Self {
        Self {
            kind: "Shell".to_string(),
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../src/lib/bindings/")]
pub struct ProjectSummary {
    pub name: String,
    pub path: String,
}

impl From<&Project> for ProjectSummary {
    fn from(project: &Project) -> Self {
        Self {
            name: project.name.clone(),
            path: path_text(&project.path),
        }
    }
}

/// The workspace at a glance: where it is, why that folder, and its projects.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../src/lib/bindings/")]
pub struct WorkspaceStatus {
    pub root: String,
    /// Where the folder was chosen from, in words.
    pub root_source: String,
    pub base: String,
    pub projects_dir: String,
    pub projects: Vec<ProjectSummary>,
}

impl WorkspaceStatus {
    pub fn new(paths: &WorkspacePaths, source: &RootSource, projects: &[Project]) -> Self {
        Self {
            root: path_text(&paths.root),
            root_source: source.describe(),
            base: path_text(&paths.base),
            projects_dir: path_text(&paths.projects),
            projects: projects.iter().map(ProjectSummary::from).collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../src/lib/bindings/")]
pub struct ProjectDetails {
    pub name: String,
    pub path: String,
    pub display_name: String,
    pub description: String,
    /// Unix seconds, as the project file stores it.
    pub created_at: String,
    /// The model this project asks for, if it names one.
    pub model: Option<String>,
    pub knowledge_dir: String,
    pub specs_dir: String,
    pub resources_dir: String,
    pub session_dir: String,
}

impl ProjectDetails {
    pub fn new(project: &Project, metadata: &ProjectMetadata) -> Self {
        Self {
            name: project.name.clone(),
            path: path_text(&project.path),
            display_name: metadata.display_name.clone(),
            description: metadata.description.clone(),
            created_at: metadata.created_at.clone(),
            model: metadata.model.clone(),
            knowledge_dir: metadata.knowledge_dir.clone(),
            specs_dir: metadata.specs_dir.clone(),
            resources_dir: metadata.resources_dir.clone(),
            session_dir: metadata.session_dir.clone(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[ts(export, export_to = "../../src/lib/bindings/")]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Error,
    Warning,
}

/// A workspace operation a fix points to. The interface words it; the desktop app names no
/// command line.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[ts(export, export_to = "../../src/lib/bindings/")]
#[serde(rename_all = "snake_case")]
pub enum FixAction {
    Init,
    Validate,
}

impl From<Action> for FixAction {
    fn from(action: Action) -> Self {
        match action {
            Action::Init => Self::Init,
            Action::Validate => Self::Validate,
        }
    }
}

/// How to clear an issue, mirroring the core's `SuggestedFix`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[ts(export, export_to = "../../src/lib/bindings/")]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Fix {
    /// Run `action` to achieve `purpose`.
    Run { action: FixAction, purpose: String },
    /// Do `step` by hand, then run `action` again.
    FixThenRerun { step: String, action: FixAction },
    /// A manual step, nothing to run afterwards.
    Manual { step: String },
}

impl From<&SuggestedFix> for Fix {
    fn from(fix: &SuggestedFix) -> Self {
        match fix {
            SuggestedFix::Run { action, purpose } => Self::Run {
                action: (*action).into(),
                purpose: purpose.clone(),
            },
            SuggestedFix::FixThenRerun { fix, action } => Self::FixThenRerun {
                step: fix.clone(),
                action: (*action).into(),
            },
            SuggestedFix::Manual(step) => Self::Manual { step: step.clone() },
        }
    }
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../src/lib/bindings/")]
pub struct Issue {
    pub severity: Severity,
    pub scope: String,
    pub subject: String,
    pub path: String,
    pub code: String,
    pub message: String,
    pub fix: Fix,
}

impl From<&ValidationIssue> for Issue {
    fn from(issue: &ValidationIssue) -> Self {
        Self {
            severity: match issue.severity {
                ValidationSeverity::Error => Severity::Error,
                ValidationSeverity::Warning => Severity::Warning,
            },
            scope: issue.scope.clone(),
            subject: issue.subject.clone(),
            path: path_text(&issue.path),
            code: issue.code.clone(),
            message: issue.message.clone(),
            fix: Fix::from(&issue.suggested_fix),
        }
    }
}

/// A validation run. Workspace and project issues arrive in one list; `scope` and `subject`
/// say where each belongs.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../src/lib/bindings/")]
pub struct Validation {
    pub valid: bool,
    pub checked_projects: u32,
    pub blocking: u32,
    pub issues: Vec<Issue>,
}

impl From<&ValidationResult> for Validation {
    fn from(result: &ValidationResult) -> Self {
        Self {
            valid: result.valid,
            checked_projects: count(result.checked_projects),
            blocking: count(result.blocking_issue_count()),
            issues: result
                .workspace_issues
                .iter()
                .chain(result.project_issues.iter().flat_map(|p| p.issues.iter()))
                .map(Issue::from)
                .collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../src/lib/bindings/")]
pub struct SearchHit {
    /// "project", "base" or "session".
    pub source_kind: String,
    pub path: String,
    pub score: f64,
    pub snippet: String,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../src/lib/bindings/")]
pub struct SearchResults {
    pub query: String,
    pub hits: Vec<SearchHit>,
}

impl From<&SearchOutcome> for SearchResults {
    fn from(outcome: &SearchOutcome) -> Self {
        Self {
            query: outcome.query_text.clone(),
            hits: outcome
                .results
                .iter()
                .map(|result| SearchHit {
                    source_kind: result.source_kind.as_str().to_string(),
                    path: path_text(&result.path),
                    score: result.relevance_score,
                    snippet: result.snippet.clone(),
                })
                .collect(),
        }
    }
}

/// What an opened session sees: its project, the files in its first context, and whether a
/// runtime can answer.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../src/lib/bindings/")]
pub struct SessionInfo {
    pub project: String,
    pub can_chat: bool,
    /// The runtime's state, in words.
    pub runtime: String,
    pub base_files: Vec<String>,
    pub project_files: Vec<String>,
    pub session_files: Vec<String>,
}

impl SessionInfo {
    pub fn new(project: &Project, context: &ContextSnapshot, runtime: &RuntimeStatus) -> Self {
        let paths = |files: &[std::path::PathBuf]| files.iter().map(|f| path_text(f)).collect();
        Self {
            project: project.name.clone(),
            can_chat: runtime.running,
            runtime: runtime.detail.clone(),
            base_files: paths(&context.base_files),
            project_files: paths(&context.project_files),
            session_files: paths(&context.session_files),
        }
    }
}

/// What closing a session did.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../src/lib/bindings/")]
pub struct SessionClosed {
    pub project: String,
    /// Temporary permissions revoked as the session ended.
    pub revoked: u32,
}
