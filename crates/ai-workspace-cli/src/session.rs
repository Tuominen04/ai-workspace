use crate::COMMAND_NAME;
use crate::ui;
use ai_workspace_core::Engine;
use ai_workspace_core::config::load_workspace_config;
use ai_workspace_core::context::{ContextSnapshot, build_context_snapshot};
use ai_workspace_core::error::{AppError, Result};
use ai_workspace_core::paths::WorkspacePaths;
use ai_workspace_core::runtime::{ChatMessage, OLLAMA_BASE_URL, RuntimeManager};
use ai_workspace_core::workspace::{
    Project, ProjectMetadata, initialize_permission_session, inspect_project, resolve_model,
    terminate_permission_session,
};

pub fn run_session(engine: &Engine) -> Result<()> {
    let paths = engine.paths();
    let projects = engine.projects()?;

    ui::render_startup_banner(paths);
    if projects.is_empty() {
        outln!("No workspaces found under {}", paths.projects.display());
        outln!("Create a project directory there and rerun {COMMAND_NAME}.");
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
