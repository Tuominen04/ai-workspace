use crate::context::build_context_snapshot;
use crate::error::{AppError, Result};
use crate::paths::WorkspacePaths;
use crate::runtime::RuntimeManager;
use crate::ui;
use crate::workspace::discover_projects;

pub fn init_workspace(paths: &WorkspacePaths) -> Result<()> {
    paths.ensure_layout()?;
    ui::render_init(paths);
    Ok(())
}

pub fn print_status(paths: &WorkspacePaths) -> Result<()> {
    paths.ensure_layout()?;
    let projects = discover_projects(paths)?;
    ui::render_status(paths, &projects);
    Ok(())
}

pub fn run_session(paths: &WorkspacePaths) -> Result<()> {
    paths.ensure_layout()?;
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
