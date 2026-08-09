use crate::context::{build_context_snapshot, ContextSnapshot};
use crate::config::{ensure_default_workspace_config, load_workspace_config};
use crate::error::{AppError, Result};
use crate::paths::WorkspacePaths;
use crate::runtime::{ChatMessage, RuntimeManager, OLLAMA_BASE_URL};
use crate::ui;
use crate::workspace::{
    create_project, discover_projects, grant_temporary_permission, initialize_permission_session,
    inspect_project, resolve_model, revoke_temporary_permission, terminate_permission_session,
    validate_workspace, view_permissions as build_permission_view, PermissionViewOptions,
    Project,
};

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
        println!("Create a project directory there and rerun ai.");
        return Err(AppError::Message("no workspaces found".to_string()));
    }

    ui::render_projects(&projects);
    let selection = ui::prompt_for_selection(&projects)?;
    let selected_project = projects
        .get(selection)
        .ok_or_else(|| AppError::InvalidSelection("workspace selection was out of range".to_string()))?;

    let _ = initialize_permission_session(paths, selected_project)?;

    let context = build_context_snapshot(paths, Some(selected_project))?;
    let mut runtime = RuntimeManager::new();
    let runtime_status = runtime.ensure_ollama()?;

    ui::render_session_summary(&selected_project, &context, &runtime_status);

    if runtime_status.running {
        let config = load_workspace_config(paths)?;
        let (_, metadata) = inspect_project(paths, &config, &selected_project.name)?;
        let model = resolve_model(&config, &metadata);
        run_chat_loop(&runtime, &model, &context);
    } else {
        ui::render_no_runtime_notice();
        ui::wait_for_exit()?;
    }

    let revoked = terminate_permission_session(paths, selected_project)?;
    ui::render_permission_session_closed(selected_project, revoked);

    Ok(())
}

fn run_chat_loop(runtime: &RuntimeManager, model: &str, context: &ContextSnapshot) {
    let mut history = vec![ChatMessage::system(context.as_system_prompt())];

    ui::render_chat_ready();

    loop {
        let message = match ui::prompt_for_chat_message() {
            Ok(Some(text)) => text,
            Ok(None) => break,
            Err(_) => break,
        };

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
    enforce_validation_gate(paths, "ai project create")?;
    let config = load_workspace_config(paths)?;
    let created = create_project(paths, &config, name, description)?;
    ui::render_project_created(&created);
    Ok(())
}

pub fn show_managed_project(paths: &WorkspacePaths, project: &str) -> Result<()> {
    paths.ensure_layout()?;
    enforce_validation_gate(paths, "ai project show")?;
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

fn enforce_validation_gate(paths: &WorkspacePaths, action: &str) -> Result<()> {
    let result = validate_workspace(paths)?;
    if result.valid {
        return Ok(());
    }

    ui::render_validation_gate_block(action, &result);
    Err(AppError::ValidationGateBlocked(action.to_string()))
}

pub fn grant_permission(paths: &WorkspacePaths, project: Option<&str>, path: &str) -> Result<()> {
    paths.ensure_layout()?;
    enforce_validation_gate(paths, "ai permission grant")?;
    let config = load_workspace_config(paths)?;
    let project = resolve_permission_project(paths, &config, project)?;
    let result = grant_temporary_permission(paths, &project, path)?;
    ui::render_permission_grant(&project, &result);
    Ok(())
}

pub fn revoke_permission(paths: &WorkspacePaths, project: Option<&str>, path: &str) -> Result<()> {
    paths.ensure_layout()?;
    enforce_validation_gate(paths, "ai permission revoke")?;
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
    enforce_validation_gate(paths, "ai permission view")?;
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
