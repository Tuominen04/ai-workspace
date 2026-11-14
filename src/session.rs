use crate::COMMAND_NAME;
use crate::config::{ensure_default_workspace_config, load_workspace_config};
use crate::context::{ContextSnapshot, build_context_snapshot};
use crate::error::{AppError, Result};
use crate::paths::WorkspacePaths;
use crate::runtime::{ChatMessage, OLLAMA_BASE_URL, RuntimeManager};
use crate::search::{self, SearchCandidate, SourceKind};
use crate::ui;
use crate::workspace::{
    PermissionStatus, PermissionViewOptions, Project, ProjectMetadata, collect_markdown_files,
    create_project, discover_projects, grant_temporary_permission, initialize_permission_session,
    inspect_project, resolve_model, revoke_temporary_permission, terminate_permission_session,
    validate_workspace, view_permissions as build_permission_view,
};
use std::path::PathBuf;

pub fn init_workspace(paths: &WorkspacePaths) -> Result<()> {
    paths.ensure_layout()?;
    let bootstrap = ensure_default_workspace_config(paths, false)?;
    ui::render_init(paths, &bootstrap.path, bootstrap.created);
    Ok(())
}

pub fn print_status(paths: &WorkspacePaths) -> Result<()> {
    paths.ensure_layout()?;
    let _ = ensure_default_workspace_config(paths, false)?;
    let projects = discover_projects(paths)?;
    ui::render_status(paths, &projects);
    Ok(())
}

pub fn run_session(paths: &WorkspacePaths) -> Result<()> {
    paths.ensure_layout()?;
    let _ = ensure_default_workspace_config(paths, false)?;
    let projects = discover_projects(paths)?;

    ui::render_startup_banner(paths);
    if projects.is_empty() {
        println!("No workspaces found under {}", paths.projects.display());
        println!("Create a project directory there and rerun {COMMAND_NAME}.");
        return Err(AppError::Message("no workspaces found".to_string()));
    }

    ui::render_projects(&projects);
    let selection = ui::prompt_for_selection(&projects)?;
    let selected_project = projects.get(selection).ok_or_else(|| {
        AppError::InvalidSelection("workspace selection was out of range".to_string())
    })?;

    let _ = initialize_permission_session(paths, selected_project)?;

    let config = load_workspace_config(paths)?;
    let (_, metadata) = inspect_project(paths, &config, &selected_project.name)?;
    let initial_query = initial_context_query(&metadata);
    let context = build_context_snapshot(paths, Some(selected_project), Some(&initial_query))?;
    let mut runtime = RuntimeManager::new();
    let runtime_status = runtime.ensure_ollama()?;

    ui::render_session_summary(selected_project, &context, &runtime_status);

    if runtime_status.running {
        let model = resolve_model(&config, &metadata);
        run_chat_loop(paths, selected_project, &runtime, &model, context);
    } else {
        ui::render_no_runtime_notice();
        ui::wait_for_exit()?;
    }

    let revoked = terminate_permission_session(paths, selected_project)?;
    ui::render_permission_session_closed(selected_project, revoked);

    Ok(())
}

/// The implicit query used to build the very first context, before the user
/// has asked anything. Falls back to the project's display name only in the
/// unlikely case its description is empty. See `research.md` Decision 4 and
/// `contracts/context-builder-integration.md` (the fix for finding I1).
fn initial_context_query(metadata: &ProjectMetadata) -> String {
    let description = metadata.description.trim();
    if description.is_empty() {
        metadata.display_name.clone()
    } else {
        description.to_string()
    }
}

/// Collects the permission-scoped candidate set search ranks against: Base
/// knowledge, and — when a project is selected — that project's Knowledge
/// and Session content, each only if the active permission session currently
/// grants it. See `research.md` Decision 3.
pub fn collect_search_candidates(
    paths: &WorkspacePaths,
    project: Option<&Project>,
) -> Result<Vec<SearchCandidate>> {
    let mut candidates = Vec::new();

    for relative_path in collect_markdown_files(paths.base_knowledge_path(), 2)? {
        candidates.push(SearchCandidate {
            source_kind: SourceKind::Base,
            path: paths.base_knowledge_path().join(relative_path),
        });
    }

    let Some(project) = project else {
        return Ok(candidates);
    };

    let permission_set = initialize_permission_session(paths, project)?;
    let granted_roots: Vec<PathBuf> = permission_set
        .default_grants
        .iter()
        .chain(permission_set.temporary_grants.iter())
        .filter(|grant| grant.status == PermissionStatus::Active)
        .map(|grant| PathBuf::from(&grant.canonical_path))
        .collect();

    let knowledge_root = project.path.join("Knowledge");
    if granted_roots
        .iter()
        .any(|root| knowledge_root.starts_with(root))
    {
        for relative_path in collect_markdown_files(&knowledge_root, 2).unwrap_or_default() {
            candidates.push(SearchCandidate {
                source_kind: SourceKind::Project,
                path: knowledge_root.join(relative_path),
            });
        }
    }

    let session_root = project.path.join("Session");
    if granted_roots
        .iter()
        .any(|root| session_root.starts_with(root))
    {
        for relative_path in collect_markdown_files(&session_root, 2).unwrap_or_default() {
            candidates.push(SearchCandidate {
                source_kind: SourceKind::Session,
                path: session_root.join(relative_path),
            });
        }
    }

    Ok(candidates)
}

pub fn run_search(
    paths: &WorkspacePaths,
    query: &str,
    project: Option<&str>,
    limit: usize,
) -> Result<()> {
    paths.ensure_layout()?;
    enforce_validation_gate(paths, "search")?;

    let project = match project {
        Some(name) => {
            let config = load_workspace_config(paths)?;
            let (project, _) = inspect_project(paths, &config, name)?;
            Some(project)
        }
        None => None,
    };

    let candidates = collect_search_candidates(paths, project.as_ref())?;
    let search_query = search::SearchQuery::new(query, limit);
    let outcome = search::search(&candidates, &search_query);

    if outcome.results.is_empty() {
        ui::render_no_search_results(&outcome);
    } else {
        ui::render_search_results(&outcome);
    }

    Ok(())
}

fn run_chat_loop(
    paths: &WorkspacePaths,
    project: &Project,
    runtime: &RuntimeManager,
    model: &str,
    initial_context: ContextSnapshot,
) {
    let mut history = vec![ChatMessage::system(initial_context.as_system_prompt())];

    ui::render_chat_ready();

    loop {
        let message = match ui::prompt_for_chat_message() {
            Ok(Some(text)) => text,
            Ok(None) => break,
            Err(_) => break,
        };

        if let Ok(context) = build_context_snapshot(paths, Some(project), Some(&message)) {
            history[0] = ChatMessage::system(context.as_system_prompt());
        }

        history.push(ChatMessage::user(message));

        match runtime.send_chat_message(OLLAMA_BASE_URL, model, &history) {
            Ok(reply) => {
                ui::render_ai_response(&reply.content);
                history.push(reply);
            }
            Err(error) => {
                history.pop();
                ui::render_chat_error(&error.to_string());
            }
        }
    }
}

pub fn show_workspace_config(paths: &WorkspacePaths) -> Result<()> {
    paths.ensure_layout()?;
    let config = load_workspace_config(paths)?;
    ui::render_workspace_config(&config);
    Ok(())
}

pub fn create_managed_project(
    paths: &WorkspacePaths,
    name: &str,
    description: Option<&str>,
) -> Result<()> {
    paths.ensure_layout()?;
    enforce_validation_gate(paths, "project create")?;
    let config = load_workspace_config(paths)?;
    let created = create_project(paths, &config, name, description)?;
    ui::render_project_created(&created);
    Ok(())
}

pub fn show_managed_project(paths: &WorkspacePaths, project: &str) -> Result<()> {
    paths.ensure_layout()?;
    enforce_validation_gate(paths, "project show")?;
    let config = load_workspace_config(paths)?;
    let (project, metadata) = inspect_project(paths, &config, project)?;
    ui::render_project_details(&project, &metadata);
    Ok(())
}

pub fn validate_workspace_state(paths: &WorkspacePaths) -> Result<()> {
    paths.ensure_layout()?;
    let result = validate_workspace(paths)?;
    ui::render_validation_result(&result);

    if result.valid {
        return Ok(());
    }

    Err(AppError::Message("workspace validation failed".to_string()))
}

/// Refuses to continue while blocking validation issues exist. `subcommand`
/// is the gated command without the command name, e.g. `"project create"`.
fn enforce_validation_gate(paths: &WorkspacePaths, subcommand: &str) -> Result<()> {
    let result = validate_workspace(paths)?;
    if result.valid {
        return Ok(());
    }

    let action = format!("{COMMAND_NAME} {subcommand}");
    ui::render_validation_gate_block(&action, &result);
    Err(AppError::ValidationGateBlocked(action))
}

pub fn grant_permission(paths: &WorkspacePaths, project: Option<&str>, path: &str) -> Result<()> {
    paths.ensure_layout()?;
    enforce_validation_gate(paths, "permission grant")?;
    let config = load_workspace_config(paths)?;
    let project = resolve_permission_project(paths, &config, project)?;
    let result = grant_temporary_permission(paths, &project, path)?;
    ui::render_permission_grant(&project, &result);
    Ok(())
}

pub fn revoke_permission(paths: &WorkspacePaths, project: Option<&str>, path: &str) -> Result<()> {
    paths.ensure_layout()?;
    enforce_validation_gate(paths, "permission revoke")?;
    let config = load_workspace_config(paths)?;
    let project = resolve_permission_project(paths, &config, project)?;
    let result = revoke_temporary_permission(paths, &project, path)?;
    ui::render_permission_revoke(&project, &result);
    Ok(())
}

pub fn view_permissions(
    paths: &WorkspacePaths,
    project: Option<&str>,
    filter: Option<&str>,
    page: usize,
    page_size: usize,
) -> Result<()> {
    paths.ensure_layout()?;
    enforce_validation_gate(paths, "permission view")?;
    let config = load_workspace_config(paths)?;
    let project = resolve_permission_project(paths, &config, project)?;
    let snapshot = build_permission_view(
        paths,
        &project,
        &PermissionViewOptions {
            filter: filter.map(|value| value.to_string()),
            page,
            page_size,
        },
    )?;
    ui::render_permission_view(&project, &snapshot);
    Ok(())
}

fn resolve_permission_project(
    paths: &WorkspacePaths,
    config: &crate::config::WorkspaceConfig,
    requested_project: Option<&str>,
) -> Result<Project> {
    if let Some(project_name) = requested_project {
        let (project, _) = inspect_project(paths, config, project_name)?;
        return Ok(project);
    }

    let projects = discover_projects(paths)?;
    if projects.len() == 1 {
        return Ok(projects[0].clone());
    }

    Err(AppError::MissingProjectContext)
}
