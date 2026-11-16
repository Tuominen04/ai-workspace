//! The wire types: what crosses from the shell to the interface.
//!
//! The core's own types are not sent directly. The TypeScript the interface is written
//! against is **generated from the types in this file** (`cargo test -p ai-workspace-desktop`
//! writes `desktop/src/lib/bindings/`), and CI fails when the committed copy is stale, so the
//! boundary has one definition rather than two that drift. Counts are `u32` so they arrive as
//! plain numbers.

use ai_workspace_core::config::WorkspaceConfig;
use ai_workspace_core::paths::{RootSource, WorkspacePaths};
use ai_workspace_core::search::SearchOutcome;
use ai_workspace_core::workspace::{
    Action, PermissionCategory, PermissionGrant, PermissionStatus, PermissionViewSnapshot, Project,
    ProjectMetadata, SuggestedFix, ValidationIssue, ValidationResult, ValidationSeverity,
};
use ai_workspace_core::{AgentInstructions, AppError, SessionSummary};
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
            AppError::InvalidModelName(_) => "InvalidModelName",
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
    /// The model the session talks to.
    pub model: String,
    /// `true` when the model's own instructions come first in every system message.
    pub agent_included: bool,
    /// The state of the model's own instructions, in words.
    pub agent: String,
    pub base_files: Vec<String>,
    pub project_files: Vec<String>,
    pub session_files: Vec<String>,
    /// Files from the session's temporary grants (ADR-0014).
    pub granted_files: Vec<String>,
    /// Files and folders left out because their names look like secrets.
    pub skipped_files: Vec<String>,
    /// How much workspace text fits, and why, in words.
    pub budget: String,
    /// Files included only in part, each as "path (N of M bytes shown)".
    pub truncated_files: Vec<String>,
}

impl SessionInfo {
    pub fn new(summary: &SessionSummary<'_>) -> Self {
        let SessionSummary {
            project,
            context,
            runtime,
            model,
            agent,
        } = *summary;
        let paths = |files: &[std::path::PathBuf]| files.iter().map(|f| path_text(f)).collect();
        Self {
            project: project.name.clone(),
            can_chat: runtime.running,
            runtime: runtime.detail.clone(),
            model: model.to_string(),
            agent_included: matches!(agent, AgentInstructions::Included(_)),
            agent: agent.to_string(),
            base_files: paths(&context.base_files),
            project_files: paths(&context.project_files),
            session_files: paths(&context.session_files),
            granted_files: paths(&context.granted_files),
            skipped_files: paths(&context.skipped_files),
            budget: context.budget.to_string(),
            truncated_files: context
                .truncated_files
                .iter()
                .map(ToString::to_string)
                .collect(),
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

/// The workspace configuration (`Config/workspace.toml`) and which folder is in use.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../src/lib/bindings/")]
pub struct WorkspaceSettings {
    /// The folder in use now, and where that choice came from, in words.
    pub active_root: String,
    pub root_source: String,
    /// The configuration file these settings were read from.
    pub config_file: String,
    /// The model used by projects that name none.
    pub default_model: String,
    pub project_metadata_file: String,
    pub default_project_directories: Vec<String>,
    /// The folders as recorded in the file when it was written. Shown for reference: the
    /// folders in use always follow `active_root`.
    pub recorded_root: String,
    pub recorded_base: String,
    pub recorded_projects: String,
    pub recorded_cache: String,
    pub recorded_logs: String,
    pub recorded_config: String,
}

impl WorkspaceSettings {
    pub fn new(paths: &WorkspacePaths, source: &RootSource, config: &WorkspaceConfig) -> Self {
        Self {
            active_root: path_text(&paths.root),
            root_source: source.describe(),
            config_file: path_text(&paths.workspace_config_file()),
            default_model: config.default_model.clone(),
            project_metadata_file: config.project_metadata_file.clone(),
            default_project_directories: config.default_project_directories.clone(),
            recorded_root: config.root_path.clone(),
            recorded_base: config.base_path.clone(),
            recorded_projects: config.projects_path.clone(),
            recorded_cache: config.cache_path.clone(),
            recorded_logs: config.logs_path.clone(),
            recorded_config: config.config_path.clone(),
        }
    }
}

/// One grant in a project's permission session.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../src/lib/bindings/")]
pub struct Grant {
    /// The canonical path; pass it back to revoke.
    pub path: String,
    /// "default" (Base and the project, always granted) or "temporary".
    pub category: String,
    /// "active", or "unavailable" when the path has disappeared since it was granted.
    pub status: String,
    /// Unix seconds.
    pub granted_at: String,
}

impl From<&PermissionGrant> for Grant {
    fn from(grant: &PermissionGrant) -> Self {
        Self {
            path: grant.canonical_path.clone(),
            category: match grant.category {
                PermissionCategory::Default => "default",
                PermissionCategory::Temporary => "temporary",
            }
            .to_string(),
            status: match grant.status {
                PermissionStatus::Active => "active",
                PermissionStatus::Unavailable => "unavailable",
            }
            .to_string(),
            granted_at: grant.granted_at.clone(),
        }
    }
}

/// One page of a project's permissions.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../src/lib/bindings/")]
pub struct PermissionPage {
    pub project: String,
    pub session_id: String,
    /// Grants matching the filter, across all pages.
    pub total: u32,
    pub page: u32,
    pub page_size: u32,
    pub grants: Vec<Grant>,
}

impl PermissionPage {
    pub fn new(project: &Project, snapshot: &PermissionViewSnapshot) -> Self {
        Self {
            project: project.name.clone(),
            session_id: snapshot.session_id.clone(),
            total: count(snapshot.total_count),
            page: count(snapshot.page),
            page_size: count(snapshot.page_size),
            grants: snapshot.entries.iter().map(Grant::from).collect(),
        }
    }
}

/// What granting did.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../src/lib/bindings/")]
pub struct GrantDone {
    pub project: String,
    pub path: String,
    /// `false` when the path was already granted (grants are idempotent).
    pub created: bool,
}

/// What revoking did.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../src/lib/bindings/")]
pub struct RevokeDone {
    pub project: String,
    pub path: String,
}

/// What initializing did.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../src/lib/bindings/")]
pub struct InitDone {
    pub root: String,
    pub config_file: String,
    /// `true` if the configuration file was written now, `false` if one was kept.
    pub config_created: bool,
}

/// What saving a workspace folder did.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../src/lib/bindings/")]
pub struct RootSaved {
    /// The canonical folder saved.
    pub root: String,
    /// The settings file it was saved in.
    pub config_file: String,
    /// Set when `AI_WORKSPACE_HOME` is set and so still decides the folder: what it names.
    pub env_override: Option<String>,
    /// The chat session that was ended because it belonged to the previous folder.
    pub session_closed: Option<SessionClosed>,
}

/// What **Switch now** did: the folder now in use, and why.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../src/lib/bindings/")]
pub struct SwitchDone {
    pub root: String,
    pub root_source: String,
    /// The chat session that was ended because it belonged to the previous folder.
    pub session_closed: Option<SessionClosed>,
}

/// Where a session note was saved.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../src/lib/bindings/")]
pub struct NoteSaved {
    pub project: String,
    pub path: String,
}
