use crate::context::build_context_snapshot;
use crate::config::{ensure_default_workspace_config, load_workspace_config};
use crate::error::{AppError, Result};
use crate::paths::WorkspacePaths;
use crate::runtime::RuntimeManager;
use crate::ui;
use crate::workspace::{create_project, discover_projects, inspect_project, validate_workspace};

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

    let context = build_context_snapshot(paths)?;
    let mut runtime = RuntimeManager::new();
    let runtime_status = runtime.ensure_ollama()?;

    ui::render_session_summary(&selected_project, &context, &runtime_status);
    ui::wait_for_exit()?;

    Ok(())
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
