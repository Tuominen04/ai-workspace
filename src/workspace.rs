use crate::config::WorkspaceConfig;
use crate::error::{AppError, Result};
use crate::paths::canonical_local_existing_path;
use crate::paths::WorkspacePaths;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Project {
    pub name: String,
    pub path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMetadata {
    pub project_id: String,
    pub display_name: String,
    pub description: String,
    pub created_at: String,
    pub knowledge_dir: String,
    pub specs_dir: String,
    pub resources_dir: String,
    pub session_dir: String,
}

#[derive(Debug, Clone)]
pub struct CreatedProject {
    pub project: Project,
    pub metadata: ProjectMetadata,
}

#[derive(Debug, Clone)]
pub enum ValidationSeverity {
    Error,
    Warning,
}

#[derive(Debug, Clone)]
pub struct ValidationIssue {
    pub severity: ValidationSeverity,
    pub scope: String,
    pub subject: String,
    pub path: PathBuf,
    pub code: String,
    pub message: String,
    pub suggested_fix: String,
}

#[derive(Debug, Clone)]
pub struct ProjectValidation {
    pub project_id: String,
    pub issues: Vec<ValidationIssue>,
}

#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub valid: bool,
    pub workspace_issues: Vec<ValidationIssue>,
    pub project_issues: Vec<ProjectValidation>,
    pub checked_projects: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PermissionCategory {
    Default,
    Temporary,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PermissionStatus {
    Active,
    Unavailable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionGrant {
    pub grant_id: String,
    pub session_id: String,
    pub canonical_path: String,
    pub display_path: String,
    pub category: PermissionCategory,
    pub status: PermissionStatus,
    pub granted_at: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionPermissionSet {
    pub session_id: String,
    pub workspace_id: String,
    pub artifact_path: String,
    pub default_grants: Vec<PermissionGrant>,
    pub temporary_grants: Vec<PermissionGrant>,
    pub created_at: String,
    pub terminated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionEvent {
    pub event_id: String,
    pub session_id: String,
    pub event_type: String,
    pub path: String,
    pub category: String,
    pub previous_status: Option<String>,
    pub new_status: Option<String>,
    pub reason_code: Option<String>,
    pub occurred_at: String,
}

#[derive(Debug, Clone)]
pub struct PermissionViewOptions {
    pub filter: Option<String>,
    pub page: usize,
    pub page_size: usize,
}

#[derive(Debug, Clone)]
pub struct PermissionViewSnapshot {
    pub session_id: String,
    pub total_count: usize,
    pub page: usize,
    pub page_size: usize,
    pub entries: Vec<PermissionGrant>,
}

#[derive(Debug, Clone)]
pub struct PermissionGrantResult {
    pub grant: PermissionGrant,
    pub created: bool,
}

#[derive(Debug, Clone)]
pub struct PermissionRevokeResult {
    pub revoked_path: String,
}

impl ValidationResult {
    pub fn blocking_issue_count(&self) -> usize {
        let workspace = self
            .workspace_issues
            .iter()
            .filter(|issue| matches!(issue.severity, ValidationSeverity::Error))
            .count();
        let projects = self
            .project_issues
            .iter()
            .flat_map(|entry| entry.issues.iter())
            .filter(|issue| matches!(issue.severity, ValidationSeverity::Error))
            .count();
        workspace + projects
    }
}

pub fn discover_projects(paths: &WorkspacePaths) -> Result<Vec<Project>> {
    if !paths.projects.exists() {
        return Ok(Vec::new());
    }

    let mut projects = Vec::new();
    for entry in fs::read_dir(&paths.projects)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        if !file_type.is_dir() {
            continue;
        }

        let path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();
        projects.push(Project { name, path });
    }

    projects.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(projects)
}

pub fn normalize_project_name(input: &str) -> Result<String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(AppError::InvalidProjectName(input.to_string()));
    }

    if !trimmed
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == ' ')
    {
        return Err(AppError::InvalidProjectName(input.to_string()));
    }

    let normalized = trimmed
        .to_ascii_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("-");

    let reserved = ["base", "projects", "cache", "logs", "config"];
    if reserved.contains(&normalized.as_str()) {
        return Err(AppError::InvalidProjectName(input.to_string()));
    }

    if normalized.is_empty() {
        return Err(AppError::InvalidProjectName(input.to_string()));
    }

    Ok(normalized)
}

pub fn create_project(
    paths: &WorkspacePaths,
    config: &WorkspaceConfig,
    requested_name: &str,
    description: Option<&str>,
) -> Result<CreatedProject> {
    let project_id = normalize_project_name(requested_name)?;
    let project_root = paths.projects.join(&project_id);

    if project_root.exists() {
        return Err(AppError::Message(format!(
            "project '{project_id}' already exists"
        )));
    }

    fs::create_dir_all(&project_root).map_err(|error| map_path_error(&project_root, error))?;
    let create_result = (|| -> Result<CreatedProject> {
        for directory in &config.default_project_directories {
            let path = project_root.join(directory);
            fs::create_dir_all(&path).map_err(|error| map_path_error(&path, error))?;
        }

        let readme_path = project_root.join("README.md");
        if !readme_path.exists() {
            let body = format!(
                "# {}\n\n{}\n",
                requested_name.trim(),
                description.unwrap_or("Project workspace managed by AI Workspace.")
            );
            fs::write(&readme_path, body).map_err(|error| map_path_error(&readme_path, error))?;
        }

        let metadata = ProjectMetadata {
            project_id: project_id.clone(),
            display_name: requested_name.trim().to_string(),
            description: description.unwrap_or("No description provided.").to_string(),
            created_at: format!("{}", unix_timestamp()),
            knowledge_dir: "Knowledge".to_string(),
            specs_dir: "Specs".to_string(),
            resources_dir: "Resources".to_string(),
            session_dir: "Session".to_string(),
        };

        save_project_metadata(paths, &project_root, config.metadata_filename(), &metadata)?;

        Ok(CreatedProject {
            project: Project {
                name: project_id,
                path: project_root.clone(),
            },
            metadata,
        })
    })();

    if create_result.is_err() {
        let _ = fs::remove_dir_all(&project_root);
    }

    create_result
}

pub fn load_project_metadata(
    paths: &WorkspacePaths,
    project_root: &Path,
    metadata_filename: &str,
) -> Result<ProjectMetadata> {
    let metadata_path = paths.project_metadata_file(project_root, metadata_filename);
    let raw = fs::read_to_string(&metadata_path).map_err(|error| map_path_error(&metadata_path, error))?;

    toml::from_str::<ProjectMetadata>(&raw).map_err(|error| AppError::Message(format!(
        "invalid project metadata at {}: {error}",
        metadata_path.display()
    )))
}

pub fn save_project_metadata(
    paths: &WorkspacePaths,
    project_root: &Path,
    metadata_filename: &str,
    metadata: &ProjectMetadata,
) -> Result<()> {
    let metadata_path = paths.project_metadata_file(project_root, metadata_filename);
    let serialized = toml::to_string_pretty(metadata)
        .map_err(|error| AppError::Message(format!("failed to serialize project metadata: {error}")))?;
    fs::write(&metadata_path, serialized).map_err(|error| map_path_error(&metadata_path, error))
}

pub fn inspect_project(
    paths: &WorkspacePaths,
    config: &WorkspaceConfig,
    project_name: &str,
) -> Result<(Project, ProjectMetadata)> {
    let normalized = normalize_project_name(project_name)?;
    let project_path = paths.projects.join(&normalized);
    if !project_path.exists() {
        return Err(AppError::ProjectNotFound(project_name.to_string()));
    }

    let metadata = load_project_metadata(paths, &project_path, config.metadata_filename())?;
    Ok((
        Project {
            name: normalized,
            path: project_path,
        },
        metadata,
    ))
}

pub fn validate_workspace(paths: &WorkspacePaths) -> Result<ValidationResult> {
    let mut workspace_issues = Vec::new();
    let mut project_issues = Vec::new();

    for (name, path) in [
        ("root", &paths.root),
        ("base", &paths.base),
        ("projects", &paths.projects),
        ("cache", &paths.cache),
        ("logs", &paths.logs),
        ("config", &paths.config),
    ] {
        if !path.exists() {
            workspace_issues.push(ValidationIssue {
                severity: ValidationSeverity::Error,
                scope: "workspace".to_string(),
                subject: name.to_string(),
                path: path.clone(),
                code: "WS-MISSING-PATH".to_string(),
                message: format!("required workspace path is missing: {}", path.display()),
                suggested_fix: "run `ai init` to recreate the standard workspace layout".to_string(),
            });
        }
    }

    let config = match crate::config::load_workspace_config(paths) {
        Ok(config) => config,
        Err(AppError::ConfigMissing(path)) => {
            workspace_issues.push(ValidationIssue {
                severity: ValidationSeverity::Error,
                scope: "workspace".to_string(),
                subject: "configuration".to_string(),
                path,
                code: "WS-CONFIG-MISSING".to_string(),
                message: "workspace configuration file is missing".to_string(),
                suggested_fix: "run `ai init` to bootstrap default workspace configuration".to_string(),
            });
            crate::config::WorkspaceConfig::default_for_paths(paths)
        }
        Err(AppError::ConfigInvalid { path, reason }) => {
            workspace_issues.push(ValidationIssue {
                severity: ValidationSeverity::Error,
                scope: "workspace".to_string(),
                subject: "configuration".to_string(),
                path,
                code: "WS-CONFIG-INVALID".to_string(),
                message: format!("workspace configuration is invalid: {reason}"),
                suggested_fix: "fix Config/workspace.toml and rerun `ai validate`".to_string(),
            });
            crate::config::WorkspaceConfig::default_for_paths(paths)
        }
        Err(error) => return Err(error),
    };

    let discovered = discover_projects(paths)?;
    for project in &discovered {
        let mut issues = Vec::new();
        match load_project_metadata(paths, &project.path, config.metadata_filename()) {
            Ok(metadata) => {
                for folder in [
                    metadata.knowledge_dir,
                    metadata.specs_dir,
                    metadata.resources_dir,
                    metadata.session_dir,
                ] {
                    let dir = project.path.join(&folder);
                    if !dir.exists() {
                        issues.push(ValidationIssue {
                            severity: ValidationSeverity::Error,
                            scope: "project".to_string(),
                            subject: project.name.clone(),
                            path: dir,
                            code: "PROJECT-DIR-MISSING".to_string(),
                            message: format!("required project directory is missing: {folder}"),
                            suggested_fix: format!("recreate '{folder}' under project {}", project.name),
                        });
                    }
                }
            }
            Err(AppError::PermissionDenied(path)) => {
                issues.push(ValidationIssue {
                    severity: ValidationSeverity::Error,
                    scope: "project".to_string(),
                    subject: project.name.clone(),
                    path,
                    code: "PROJECT-PERMISSION".to_string(),
                    message: "permission denied while reading project metadata".to_string(),
                    suggested_fix: "adjust file permissions and rerun `ai validate`".to_string(),
                });
            }
            Err(error) => {
                issues.push(ValidationIssue {
                    severity: ValidationSeverity::Error,
                    scope: "project".to_string(),
                    subject: project.name.clone(),
                    path: project.path.clone(),
                    code: "PROJECT-METADATA-INVALID".to_string(),
                    message: format!("project metadata could not be loaded: {error}"),
                    suggested_fix: "fix project.toml and rerun `ai validate`".to_string(),
                });
            }
        }

        if !issues.is_empty() {
            project_issues.push(ProjectValidation {
                project_id: project.name.clone(),
                issues,
            });
        }
    }

    let result = ValidationResult {
        valid: workspace_issues
            .iter()
            .chain(project_issues.iter().flat_map(|entry| entry.issues.iter()))
            .all(|issue| matches!(issue.severity, ValidationSeverity::Warning)),
        workspace_issues,
        project_issues,
        checked_projects: discovered.len(),
    };

    Ok(result)
}

fn map_path_error(path: &Path, error: std::io::Error) -> AppError {
    if error.kind() == std::io::ErrorKind::PermissionDenied {
        return AppError::PermissionDenied(path.to_path_buf());
    }
    AppError::Io(error)
}

fn unix_timestamp() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

pub fn collect_markdown_files(root: &Path, max_depth: usize) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    collect_markdown_files_recursive(root, root, max_depth, &mut files)?;
    files.sort();
    Ok(files)
}

pub fn initialize_permission_session(paths: &WorkspacePaths, project: &Project) -> Result<SessionPermissionSet> {
    if let Some(existing) = load_permission_set(paths, project)? {
        return Ok(existing);
    }

    let now = unix_timestamp().to_string();
    let session_id = format!("{}-{}", project.name, now);
    let artifact = paths.project_permissions_file(&project.path);

    let set = SessionPermissionSet {
        session_id: session_id.clone(),
        workspace_id: project.name.clone(),
        artifact_path: artifact.display().to_string(),
        default_grants: vec![
            default_grant(&session_id, &paths.base, &now, "system-default"),
            default_grant(&session_id, &project.path, &now, "system-default"),
        ],
        temporary_grants: Vec::new(),
        created_at: now,
        terminated_at: None,
    };

    save_permission_set(paths, project, &set)?;
    Ok(set)
}

pub fn grant_temporary_permission(
    paths: &WorkspacePaths,
    project: &Project,
    requested_path: &str,
) -> Result<PermissionGrantResult> {
    let mut set = initialize_permission_session(paths, project)?;
    let canonical = match canonical_local_existing_path(requested_path) {
        Ok(path) => path,
        Err(error) => {
            append_permission_event(
                paths,
                &PermissionEvent {
                    event_id: generate_id("deny"),
                    session_id: set.session_id.clone(),
                    event_type: "deny".to_string(),
                    path: requested_path.to_string(),
                    category: "temporary".to_string(),
                    previous_status: None,
                    new_status: None,
                    reason_code: Some("invalid-path".to_string()),
                    occurred_at: unix_timestamp().to_string(),
                },
            )?;
            return Err(error);
        }
    };
    let canonical_str = canonical.display().to_string();
    if let Some(existing) = set
        .temporary_grants
        .iter_mut()
        .find(|grant| grant.canonical_path == canonical_str)
    {
        existing.status = PermissionStatus::Active;
        let existing_grant = existing.clone();
        save_permission_set(paths, project, &set)?;
        append_permission_event(
            paths,
            &PermissionEvent {
                event_id: generate_id("grant"),
                session_id: set.session_id.clone(),
                event_type: "grant".to_string(),
                path: existing_grant.canonical_path.clone(),
                category: "temporary".to_string(),
                previous_status: None,
                new_status: Some("active".to_string()),
                reason_code: Some("idempotent".to_string()),
                occurred_at: unix_timestamp().to_string(),
            },
        )?;
        return Ok(PermissionGrantResult {
            grant: existing_grant,
            created: false,
        });
    }

    let grant = PermissionGrant {
        grant_id: generate_id("grant"),
        session_id: set.session_id.clone(),
        canonical_path: canonical_str.clone(),
        display_path: requested_path.to_string(),
        category: PermissionCategory::Temporary,
        status: PermissionStatus::Active,
        granted_at: unix_timestamp().to_string(),
        source: "user-grant".to_string(),
    };
    set.temporary_grants.push(grant.clone());

    save_permission_set(paths, project, &set)?;
    append_permission_event(
        paths,
        &PermissionEvent {
            event_id: generate_id("grant"),
            session_id: set.session_id.clone(),
            event_type: "grant".to_string(),
            path: canonical_str,
            category: "temporary".to_string(),
            previous_status: None,
            new_status: Some("active".to_string()),
            reason_code: None,
            occurred_at: unix_timestamp().to_string(),
        },
    )?;

    Ok(PermissionGrantResult {
        grant,
        created: true,
    })
}

pub fn revoke_temporary_permission(
    paths: &WorkspacePaths,
    project: &Project,
    requested_path: &str,
) -> Result<PermissionRevokeResult> {
    let mut set = initialize_permission_session(paths, project)?;
    let lookup = normalize_permission_lookup_path(requested_path)?;

    let maybe_index = set.temporary_grants.iter().position(|grant| {
        grant.canonical_path == lookup || grant.display_path == requested_path
    });

    let index = maybe_index.ok_or_else(|| AppError::PermissionGrantNotFound(requested_path.to_string()))?;
    let removed = set.temporary_grants.remove(index);
    save_permission_set(paths, project, &set)?;

    append_permission_event(
        paths,
        &PermissionEvent {
            event_id: generate_id("revoke"),
            session_id: set.session_id,
            event_type: "revoke".to_string(),
            path: removed.canonical_path.clone(),
            category: "temporary".to_string(),
            previous_status: Some(permission_status_string(&removed.status)),
            new_status: None,
            reason_code: None,
            occurred_at: unix_timestamp().to_string(),
        },
    )?;

    Ok(PermissionRevokeResult {
        revoked_path: removed.canonical_path,
    })
}

pub fn view_permissions(
    paths: &WorkspacePaths,
    project: &Project,
    options: &PermissionViewOptions,
) -> Result<PermissionViewSnapshot> {
    let mut set = initialize_permission_session(paths, project)?;
    let mut status_events: Vec<PermissionEvent> = Vec::new();

    for grant in &mut set.temporary_grants {
        let is_available = PathBuf::from(&grant.canonical_path).exists();
        let new_status = if is_available {
            PermissionStatus::Active
        } else {
            PermissionStatus::Unavailable
        };

        if grant.status != new_status {
            status_events.push(PermissionEvent {
                event_id: generate_id("status-change"),
                session_id: set.session_id.clone(),
                event_type: "status-change".to_string(),
                path: grant.canonical_path.clone(),
                category: "temporary".to_string(),
                previous_status: Some(permission_status_string(&grant.status)),
                new_status: Some(permission_status_string(&new_status)),
                reason_code: Some("availability-check".to_string()),
                occurred_at: unix_timestamp().to_string(),
            });
            grant.status = new_status;
        }
    }

    if !status_events.is_empty() {
        save_permission_set(paths, project, &set)?;
        for event in status_events {
            append_permission_event(paths, &event)?;
        }
    }

    let filter = options.filter.as_ref().map(|value| value.to_ascii_lowercase());
    let mut all_entries: Vec<PermissionGrant> = set
        .default_grants
        .iter()
        .chain(set.temporary_grants.iter())
        .filter(|grant| {
            if let Some(filter_value) = &filter {
                let path = grant.canonical_path.to_ascii_lowercase();
                let display = grant.display_path.to_ascii_lowercase();
                return path.contains(filter_value) || display.contains(filter_value);
            }
            true
        })
        .cloned()
        .collect();

    all_entries.sort_by(|left, right| left.canonical_path.cmp(&right.canonical_path));

    let total_count = all_entries.len();
    let page = options.page.max(1);
    let page_size = options.page_size.max(1);
    let start = (page - 1) * page_size;
    let end = (start + page_size).min(total_count);
    let entries = if start >= total_count {
        Vec::new()
    } else {
        all_entries[start..end].to_vec()
    };

    Ok(PermissionViewSnapshot {
        session_id: set.session_id,
        total_count,
        page,
        page_size,
        entries,
    })
}

pub fn ensure_permission_access(
    paths: &WorkspacePaths,
    project: &Project,
    requested_path: &str,
) -> Result<()> {
    let set = initialize_permission_session(paths, project)?;
    let lookup = normalize_permission_lookup_path(requested_path)?;

    for grant in set.default_grants.iter().chain(set.temporary_grants.iter()) {
        if grant.canonical_path == lookup || grant.display_path == requested_path {
            if grant.status == PermissionStatus::Unavailable {
                return Err(AppError::Message(format!(
                    "permission exists but path is currently unavailable: {}",
                    grant.canonical_path
                )));
            }
            return Ok(());
        }
    }

    Err(AppError::PermissionGrantNotFound(requested_path.to_string()))
}

pub fn terminate_permission_session(paths: &WorkspacePaths, project: &Project) -> Result<usize> {
    let Some(set) = load_permission_set(paths, project)? else {
        return Ok(0);
    };

    for grant in &set.temporary_grants {
        append_permission_event(
            paths,
            &PermissionEvent {
                event_id: generate_id("revoke"),
                session_id: set.session_id.clone(),
                event_type: "revoke".to_string(),
                path: grant.canonical_path.clone(),
                category: "temporary".to_string(),
                previous_status: Some(permission_status_string(&grant.status)),
                new_status: None,
                reason_code: Some("session-end".to_string()),
                occurred_at: unix_timestamp().to_string(),
            },
        )?;
    }

    let permissions_path = paths.project_permissions_file(&project.path);
    if permissions_path.exists() {
        fs::remove_file(&permissions_path).map_err(|error| map_path_error(&permissions_path, error))?;
    }

    Ok(set.temporary_grants.len())
}

fn load_permission_set(paths: &WorkspacePaths, project: &Project) -> Result<Option<SessionPermissionSet>> {
    let permissions_path = paths.project_permissions_file(&project.path);
    if !permissions_path.exists() {
        return Ok(None);
    }

    let raw = fs::read_to_string(&permissions_path).map_err(|error| map_path_error(&permissions_path, error))?;
    let parsed = toml::from_str::<SessionPermissionSet>(&raw).map_err(|error| {
        AppError::Message(format!(
            "invalid permission session artifact at {}: {error}",
            permissions_path.display()
        ))
    })?;
    Ok(Some(parsed))
}

fn save_permission_set(paths: &WorkspacePaths, project: &Project, set: &SessionPermissionSet) -> Result<()> {
    let session_dir = project.path.join("Session");
    fs::create_dir_all(&session_dir).map_err(|error| map_path_error(&session_dir, error))?;
    let permissions_path = paths.project_permissions_file(&project.path);
    let serialized = toml::to_string_pretty(set)
        .map_err(|error| AppError::Message(format!("failed to serialize permission state: {error}")))?;
    fs::write(&permissions_path, serialized).map_err(|error| map_path_error(&permissions_path, error))
}

fn append_permission_event(paths: &WorkspacePaths, event: &PermissionEvent) -> Result<()> {
    fs::create_dir_all(&paths.logs).map_err(|error| map_path_error(&paths.logs, error))?;
    let log_path = paths.logs.join("permissions-events.log");
    let serialized = toml::to_string(event)
        .map_err(|error| AppError::Message(format!("failed to serialize permission event: {error}")))?;

    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .map_err(|error| map_path_error(&log_path, error))?;

    file.write_all(serialized.as_bytes())
        .map_err(AppError::Io)?;
    file.write_all(b"\n---\n").map_err(AppError::Io)
}

fn permission_status_string(status: &PermissionStatus) -> String {
    match status {
        PermissionStatus::Active => "active".to_string(),
        PermissionStatus::Unavailable => "unavailable".to_string(),
    }
}

fn default_grant(session_id: &str, path: &Path, now: &str, source: &str) -> PermissionGrant {
    PermissionGrant {
        grant_id: generate_id("default"),
        session_id: session_id.to_string(),
        canonical_path: path.display().to_string(),
        display_path: path.display().to_string(),
        category: PermissionCategory::Default,
        status: PermissionStatus::Active,
        granted_at: now.to_string(),
        source: source.to_string(),
    }
}

fn generate_id(prefix: &str) -> String {
    format!("{prefix}-{}", unix_timestamp())
}

fn normalize_permission_lookup_path(input: &str) -> Result<String> {
    let path = PathBuf::from(input);
    if path.exists() {
        return Ok(canonical_local_existing_path(input)?.display().to_string());
    }

    if path.is_absolute() {
        return Ok(path.display().to_string());
    }

    let cwd = std::env::current_dir().map_err(AppError::Io)?;
    Ok(cwd.join(path).display().to_string())
}

fn collect_markdown_files_recursive(
    root: &Path,
    current: &Path,
    depth_left: usize,
    files: &mut Vec<PathBuf>,
) -> Result<()> {
    if depth_left == 0 {
        return Ok(());
    }

    for entry in fs::read_dir(current)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let path = entry.path();

        if file_type.is_file() {
            if path.extension().is_some_and(|ext| ext == "md") {
                files.push(path.strip_prefix(root).unwrap_or(&path).to_path_buf());
            }
            continue;
        }

        if file_type.is_dir() {
            collect_markdown_files_recursive(root, &path, depth_left - 1, files)?;
        }
    }

    Ok(())
}
