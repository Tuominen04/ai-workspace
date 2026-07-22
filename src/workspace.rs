use crate::config::WorkspaceConfig;
use crate::error::{AppError, Result};
use crate::paths::WorkspacePaths;
use serde::{Deserialize, Serialize};
use std::fs;
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
