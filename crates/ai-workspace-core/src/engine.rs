//! `Engine`: the workspace operations a front end calls. Each method does
//! one operation and returns its result as data; nothing here prints or
//! prompts. See ADR-0008.

use crate::config::{WorkspaceConfig, ensure_default_workspace_config, load_workspace_config};
use crate::error::{AppError, GatedOperation, Result};
use crate::paths::{
    RootSource, WorkspacePaths, canonical_workspace_root, env_root_override, user_config_file,
    write_user_root,
};
use crate::search::{self, SearchCandidate, SearchOutcome};
use crate::workspace::{
    CreatedProject, PermissionGrantResult, PermissionRevokeResult, PermissionViewOptions,
    PermissionViewSnapshot, Project, ProjectMetadata, ValidationResult, archive_project,
    create_project, discover_archived_projects, discover_projects, grant_temporary_permission,
    inspect_project, now_unix, restore_project, revoke_temporary_permission, save_session_note,
    set_project_model, validate_workspace, view_permissions,
};
use std::path::PathBuf;

/// An open workspace: its folder layout and where that folder was chosen
/// from.
#[derive(Debug, Clone)]
pub struct Engine {
    paths: WorkspacePaths,
    source: RootSource,
}

/// What `Engine::init` did.
#[derive(Debug, Clone)]
pub struct InitReport {
    /// The workspace configuration file.
    pub config_path: PathBuf,
    /// `true` if the configuration file was written now, `false` if an
    /// existing one was kept.
    pub config_created: bool,
}

/// What `set_workspace_root` saved.
#[derive(Debug, Clone)]
pub struct SetRootReport {
    /// The canonical folder that was saved.
    pub root: PathBuf,
    /// The per-user settings file it was saved in.
    pub config_file: PathBuf,
    /// Set when `AI_WORKSPACE_HOME` is set and so still overrides the saved
    /// folder: the folder that variable names.
    pub env_override: Option<PathBuf>,
}

impl Engine {
    /// Resolves the workspace folder: `AI_WORKSPACE_HOME`, else the per-user
    /// settings file, else `~/AI` (ADR-0007). Touches nothing on disk.
    pub fn open() -> Result<Self> {
        let (paths, source) = WorkspacePaths::detect_with_source()?;
        Ok(Self::at(paths, source))
    }

    /// Like [`Engine::open`], but ignores `AI_WORKSPACE_HOME`: the saved
    /// folder, else `~/AI`. The variable's priority is the rule (ADR-0007);
    /// this is for a front end the person has explicitly asked to use the
    /// saved folder anyway (ADR-0011).
    pub fn open_saved() -> Result<Self> {
        let (paths, source) = WorkspacePaths::detect_saved_with_source()?;
        Ok(Self::at(paths, source))
    }

    /// An engine for an already resolved workspace folder.
    pub fn at(paths: WorkspacePaths, source: RootSource) -> Self {
        Self { paths, source }
    }

    pub fn paths(&self) -> &WorkspacePaths {
        &self.paths
    }

    pub fn root_source(&self) -> &RootSource {
        &self.source
    }

    /// Creates the standard layout and, if missing, the default workspace
    /// configuration.
    pub fn init(&self) -> Result<InitReport> {
        self.paths.ensure_layout()?;
        let bootstrap = ensure_default_workspace_config(&self.paths, false)?;
        Ok(InitReport {
            config_path: bootstrap.path,
            config_created: bootstrap.created,
        })
    }

    /// The workspace's projects. Creates the standard layout and default
    /// configuration first if they are missing, as every interactive entry
    /// point does.
    pub fn projects(&self) -> Result<Vec<Project>> {
        self.paths.ensure_layout()?;
        let _ = ensure_default_workspace_config(&self.paths, false)?;
        discover_projects(&self.paths)
    }

    /// The workspace configuration (`Config/workspace.toml`).
    pub fn config(&self) -> Result<WorkspaceConfig> {
        self.paths.ensure_layout()?;
        load_workspace_config(&self.paths)
    }

    /// Validates the workspace. Never refuses: the result says whether it is
    /// valid.
    pub fn validate(&self) -> Result<ValidationResult> {
        self.paths.ensure_layout()?;
        validate_workspace(&self.paths)
    }

    pub fn create_project(&self, name: &str, description: Option<&str>) -> Result<CreatedProject> {
        self.paths.ensure_layout()?;
        self.enforce_validation_gate(GatedOperation::ProjectCreate)?;
        let config = load_workspace_config(&self.paths)?;
        create_project(&self.paths, &config, name, description)
    }

    pub fn project_details(&self, name: &str) -> Result<(Project, ProjectMetadata)> {
        self.paths.ensure_layout()?;
        self.enforce_validation_gate(GatedOperation::ProjectShow)?;
        let config = load_workspace_config(&self.paths)?;
        inspect_project(&self.paths, &config, name)
    }

    /// Sets the Ollama model `name` uses, or with `None` returns it to the workspace default.
    pub fn set_project_model(
        &self,
        name: &str,
        model: Option<&str>,
    ) -> Result<(Project, ProjectMetadata)> {
        self.paths.ensure_layout()?;
        self.enforce_validation_gate(GatedOperation::ProjectSetModel)?;
        let config = load_workspace_config(&self.paths)?;
        set_project_model(&self.paths, &config, name, model)
    }

    /// The archived projects, in `Projects/Archive/` (ADR-0015).
    pub fn archived_projects(&self) -> Result<Vec<Project>> {
        self.paths.ensure_layout()?;
        discover_archived_projects(&self.paths)
    }

    /// Archives a project: moves its folder, unchanged, into `Projects/Archive/`. Not gated: it
    /// moves the folder without reading it, so a project with broken metadata can still be put
    /// away (ADR-0015). Ending an open session in the project is the front end's job.
    pub fn archive_project(&self, name: &str) -> Result<PathBuf> {
        self.paths.ensure_layout()?;
        archive_project(&self.paths, name)
    }

    /// Restores an archived project into `Projects/`. Not gated, like archiving.
    pub fn restore_project(&self, name: &str) -> Result<PathBuf> {
        self.paths.ensure_layout()?;
        restore_project(&self.paths, name)
    }

    /// Saves a session note in the project's Session folder and returns its path. Not gated by
    /// validation: it writes only into the project's own Session folder, at the person's
    /// explicit request, and a blocked workspace should not cost the record of a session.
    pub fn save_session_note(&self, name: &str, text: &str) -> Result<PathBuf> {
        self.paths.ensure_layout()?;
        let config = load_workspace_config(&self.paths)?;
        save_session_note(&self.paths, &config, name, text, now_unix())
    }

    /// Grants temporary access to `path` in a project's permission session.
    /// With no project named, the workspace's only project is used.
    pub fn grant(
        &self,
        project: Option<&str>,
        path: &str,
    ) -> Result<(Project, PermissionGrantResult)> {
        self.paths.ensure_layout()?;
        self.enforce_validation_gate(GatedOperation::PermissionGrant)?;
        let project = self.resolve_permission_project(project)?;
        let result = grant_temporary_permission(&self.paths, &project, path)?;
        Ok((project, result))
    }

    /// Revokes a temporary grant. With no project named, the workspace's only
    /// project is used.
    pub fn revoke(
        &self,
        project: Option<&str>,
        path: &str,
    ) -> Result<(Project, PermissionRevokeResult)> {
        self.paths.ensure_layout()?;
        self.enforce_validation_gate(GatedOperation::PermissionRevoke)?;
        let project = self.resolve_permission_project(project)?;
        let result = revoke_temporary_permission(&self.paths, &project, path)?;
        Ok((project, result))
    }

    /// One page of a project's permissions. With no project named, the
    /// workspace's only project is used.
    pub fn permissions(
        &self,
        project: Option<&str>,
        options: PermissionViewOptions,
    ) -> Result<(Project, PermissionViewSnapshot)> {
        self.paths.ensure_layout()?;
        self.enforce_validation_gate(GatedOperation::PermissionView)?;
        let project = self.resolve_permission_project(project)?;
        let snapshot = view_permissions(&self.paths, &project, &options)?;
        Ok((project, snapshot))
    }

    /// Searches what the caller may read: Base knowledge and, with a project,
    /// that project's granted Knowledge and Session content.
    pub fn search(
        &self,
        query: &str,
        project: Option<&str>,
        limit: usize,
    ) -> Result<SearchOutcome> {
        self.paths.ensure_layout()?;
        self.enforce_validation_gate(GatedOperation::Search)?;

        let project = match project {
            Some(name) => {
                let config = load_workspace_config(&self.paths)?;
                let (project, _) = inspect_project(&self.paths, &config, name)?;
                Some(project)
            }
            None => None,
        };

        let candidates = collect_search_candidates(&self.paths, project.as_ref())?;
        let search_query = search::SearchQuery::new(query, limit);
        Ok(search::search(&candidates, &search_query))
    }

    /// Refuses to continue while blocking validation issues exist.
    fn enforce_validation_gate(&self, operation: GatedOperation) -> Result<()> {
        let result = validate_workspace(&self.paths)?;
        if result.valid {
            return Ok(());
        }

        Err(AppError::ValidationGateBlocked {
            operation,
            result: Box::new(result),
        })
    }

    fn resolve_permission_project(&self, requested_project: Option<&str>) -> Result<Project> {
        let config = load_workspace_config(&self.paths)?;
        if let Some(project_name) = requested_project {
            let (project, _) = inspect_project(&self.paths, &config, project_name)?;
            return Ok(project);
        }

        let projects = discover_projects(&self.paths)?;
        if projects.len() == 1 {
            return Ok(projects[0].clone());
        }

        Err(AppError::MissingProjectContext)
    }
}

/// Saves `path` as the workspace folder in the per-user settings file. Needs
/// no existing workspace, so it never runs the validation gate.
pub fn set_workspace_root(path: &str) -> Result<SetRootReport> {
    let root = canonical_workspace_root(path)?;
    let config_file = user_config_file()?;
    write_user_root(&config_file, &root)?;
    Ok(SetRootReport {
        root,
        config_file,
        env_override: env_root_override(),
    })
}

/// The candidates search ranks: everything the model may read (`context::readable_sources`).
fn collect_search_candidates(
    paths: &WorkspacePaths,
    project: Option<&Project>,
) -> Result<Vec<SearchCandidate>> {
    // The same scope as the chat's context (ADR-0014): Base, the whole project, active grants.
    Ok(crate::context::readable_sources(paths, project)?
        .sources
        .into_iter()
        .map(|(source_kind, path)| SearchCandidate { source_kind, path })
        .collect())
}
