use crate::COMMAND_NAME;
use ai_workspace_core::config::WorkspaceConfig;
use ai_workspace_core::error::{AppError, Result};
use ai_workspace_core::paths::{RootSource, WorkspacePaths};
use ai_workspace_core::search::SearchOutcome;
use ai_workspace_core::workspace::{
    Action, CreatedProject, PermissionCategory, PermissionGrantResult, PermissionRevokeResult,
    PermissionStatus, PermissionViewSnapshot, Project, ProjectMetadata, SuggestedFix,
    ValidationIssue, ValidationResult, ValidationSeverity,
};
use ai_workspace_core::{SessionSummary, SetRootReport};
use std::fmt;
use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, Ordering};

/// Writes command output to stdout without panicking when stdout fails.
///
/// `println!` panics when the reader has gone away, which turned
/// `ai-workspace --help | head` into a crash. A closed pipe is not an error
/// here: the reader asked for no more, so the rest of the output is dropped
/// and the command carries on to its normal end (so `Drop` still stops a
/// runtime this process started). Any other write failure is reported on
/// stderr once and the command carries on the same way.
pub fn write_stdout(args: fmt::Arguments<'_>) {
    static REPORTED: AtomicBool = AtomicBool::new(false);

    if let Err(error) = io::stdout().lock().write_fmt(args)
        && error.kind() != io::ErrorKind::BrokenPipe
        && !REPORTED.swap(true, Ordering::Relaxed)
    {
        eprintln!("error: could not write output: {error}");
    }
}

pub fn render_startup_banner(paths: &WorkspacePaths) {
    outln!("AI Workspace");
    outln!("Root: {}", paths.root.display());
}

pub fn render_projects(projects: &[Project]) {
    outln!("Available workspaces:");
    for (index, project) in projects.iter().enumerate() {
        outln!("  {}. {}", index + 1, project.name);
    }
}

pub fn prompt_for_selection(projects: &[Project]) -> Result<usize> {
    if projects.len() == 1 {
        return Ok(0);
    }

    loop {
        out!("Select workspace [1-{}] or q to quit: ", projects.len());
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let value = input.trim();

        if value.eq_ignore_ascii_case("q") || value.eq_ignore_ascii_case("quit") {
            return Err(AppError::Message("Session cancelled by user".to_string()));
        }

        match value.parse::<usize>() {
            Ok(number) if number >= 1 && number <= projects.len() => return Ok(number - 1),
            _ => outln!(
                "Invalid selection. Enter a number between 1 and {}.",
                projects.len()
            ),
        }
    }
}

pub fn render_session_summary(summary: &SessionSummary<'_>) {
    let SessionSummary {
        project,
        context,
        runtime,
        model,
        agent,
    } = *summary;
    outln!();
    outln!("Workspace: {}", project.name);
    outln!("Project path: {}", project.path.display());
    outln!("Runtime: {}", runtime.detail);
    outln!("Model: {model} ({agent})");
    outln!("Base context files: {}", context.base_files.len());
    outln!("Project context files: {}", context.project_files.len());
    outln!("Session context files: {}", context.session_files.len());
    outln!("Granted context files: {}", context.granted_files.len());
    outln!("Total context sources: {}", context.source_count);
    outln!("Context budget: {}", context.budget);
    for file in &context.truncated_files {
        outln!("Cut to fit: {file}");
    }
    if !context.skipped_files.is_empty() {
        outln!(
            "Skipped because the names look like secrets: {}",
            context.skipped_files.len()
        );
        for file in &context.skipped_files {
            outln!("  - {}", file.display());
        }
    }
    outln!("AI access policy: read-only base scope (~/AI/Base)");

    if !context.base_files.is_empty() {
        outln!("Base context:");
        for file in &context.base_files {
            outln!("  - {}", file.display());
        }
    }

    if !context.project_files.is_empty() {
        outln!("Project context:");
        for file in &context.project_files {
            outln!("  - {}", file.display());
        }
    }

    if !context.session_files.is_empty() {
        outln!("Session context:");
        for file in &context.session_files {
            outln!("  - {}", file.display());
        }
    }

    if !context.granted_files.is_empty() {
        outln!("Granted context:");
        for file in &context.granted_files {
            outln!("  - {}", file.display());
        }
    }

    outln!();
}

pub fn render_chat_ready() {
    outln!("Session is active. Ask a question, or type 'exit' to close the workspace.");
}

pub fn render_no_runtime_notice() {
    outln!("Session is active. Press Enter to close the workspace.");
}

pub fn wait_for_exit() -> Result<()> {
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(())
}

pub fn prompt_for_chat_message() -> Result<Option<String>> {
    loop {
        out!("> ");
        io::stdout().flush()?;

        let mut input = String::new();
        let bytes_read = io::stdin().read_line(&mut input)?;

        if bytes_read == 0 {
            return Ok(None);
        }

        let value = input.trim();
        if value.eq_ignore_ascii_case("exit") || value.eq_ignore_ascii_case("quit") {
            return Ok(None);
        }

        if value.is_empty() {
            continue;
        }

        return Ok(Some(value.to_string()));
    }
}

pub fn render_ai_response(content: &str) {
    outln!("[AI] {content}");
    outln!();
}

pub fn render_chat_error(message: &str) {
    outln!("Could not reach the AI runtime: {message}");
    outln!("You can try again, or type 'exit' to close the workspace.");
}

pub fn render_status(paths: &WorkspacePaths, source: &RootSource, projects: &[Project]) {
    outln!("AI Workspace status");
    outln!("Root: {}", paths.root.display());
    outln!("Root chosen by: {}", source.describe());
    outln!("Base: {}", paths.base.display());
    outln!("Projects: {}", paths.projects.display());
    outln!("Discovered workspaces: {}", projects.len());

    for project in projects {
        outln!("- {} ({})", project.name, project.path.display());
    }
}

pub fn render_init(paths: &WorkspacePaths, config_path: &std::path::Path, config_created: bool) {
    outln!("Initialized workspace root at {}", paths.root.display());
    outln!("Base layout: {}", paths.base.display());
    outln!("Projects directory: {}", paths.projects.display());
    if config_created {
        outln!("Workspace config created: {}", config_path.display());
    } else {
        outln!("Workspace config kept: {}", config_path.display());
    }
}

pub fn render_workspace_config(
    config: &WorkspaceConfig,
    paths: &WorkspacePaths,
    source: &RootSource,
) {
    outln!("Workspace configuration");
    outln!("Active root: {}", paths.root.display());
    outln!("Root chosen by: {}", source.describe());
    outln!("Root: {}", config.root_path);
    outln!("Base: {}", config.base_path);
    outln!("Projects: {}", config.projects_path);
    outln!("Cache: {}", config.cache_path);
    outln!("Logs: {}", config.logs_path);
    outln!("Config: {}", config.config_path);
    outln!("Project metadata file: {}", config.project_metadata_file);
    outln!("Default project directories:");
    for directory in &config.default_project_directories {
        outln!("- {}", directory);
    }
}

pub fn render_workspace_root_set(report: &SetRootReport) {
    outln!("Workspace folder set to {}", report.root.display());
    outln!("Saved in {}", report.config_file.display());
    outln!(
        "Run `{COMMAND_NAME} init` to create the standard layout there if it is not set up yet."
    );
    if let Some(env_root) = &report.env_override {
        outln!(
            "Note: {} is set to {} and takes precedence. Unset it to use the saved folder.",
            ai_workspace_core::paths::WORKSPACE_ROOT_ENV,
            env_root.display()
        );
    }
}

pub fn render_project_created(project: &CreatedProject) {
    outln!("Project created: {}", project.project.name);
    outln!("Path: {}", project.project.path.display());
    outln!("Description: {}", project.metadata.description);
}

pub fn render_project_details(project: &Project, metadata: &ProjectMetadata) {
    outln!("Project: {}", project.name);
    outln!("Path: {}", project.path.display());
    outln!("Display name: {}", metadata.display_name);
    outln!("Description: {}", metadata.description);
    outln!(
        "Model: {}",
        metadata.model.as_deref().unwrap_or("workspace default")
    );
    outln!("Created at: {}", metadata.created_at);
    outln!("Knowledge dir: {}", metadata.knowledge_dir);
    outln!("Specs dir: {}", metadata.specs_dir);
    outln!("Resources dir: {}", metadata.resources_dir);
    outln!("Session dir: {}", metadata.session_dir);
}

pub fn render_validation_result(result: &ValidationResult) {
    outln!("Workspace validation");
    outln!("Checked projects: {}", result.checked_projects);

    if result.valid {
        outln!("Status: PASS");
        return;
    }

    outln!(
        "Status: FAIL (blocking issues: {})",
        result.blocking_issue_count()
    );

    for issue in &result.workspace_issues {
        render_issue(issue);
    }

    for project in &result.project_issues {
        outln!("Project issues: {}", project.project_id);
        for issue in &project.issues {
            render_issue(issue);
        }
    }

    outln!("Gated commands blocked: {COMMAND_NAME} project create, {COMMAND_NAME} project show");
    outln!("Run `{COMMAND_NAME} validate` after fixes to clear blocking issues.");
}

pub fn render_validation_gate_block(action: &str, result: &ValidationResult) {
    outln!(
        "Cannot run {}: workspace validation has {} blocking issue(s).",
        action,
        result.blocking_issue_count()
    );
    outln!("Run `{COMMAND_NAME} validate` to inspect and clear blocking issues first.");
}

pub fn render_permission_grant(project: &Project, result: &PermissionGrantResult) {
    outln!("Project: {}", project.name);
    if result.created {
        outln!("Permission granted: {}", result.grant.canonical_path);
    } else {
        outln!(
            "Permission already active (idempotent): {}",
            result.grant.canonical_path
        );
    }
}

pub fn render_permission_revoke(project: &Project, result: &PermissionRevokeResult) {
    outln!("Project: {}", project.name);
    outln!("Permission revoked: {}", result.revoked_path);
}

pub fn render_permission_view(project: &Project, snapshot: &PermissionViewSnapshot) {
    outln!("Project: {}", project.name);
    outln!("Session: {}", snapshot.session_id);
    outln!(
        "Permissions: total={} page={} page_size={}",
        snapshot.total_count,
        snapshot.page,
        snapshot.page_size
    );

    if snapshot.entries.is_empty() {
        outln!("No permissions in this view.");
        return;
    }

    for grant in &snapshot.entries {
        let category = match grant.category {
            PermissionCategory::Default => "default",
            PermissionCategory::Temporary => "temporary",
        };
        let status = match grant.status {
            PermissionStatus::Active => "active",
            PermissionStatus::Unavailable => "unavailable",
        };
        outln!("- [{}][{}] {}", category, status, grant.canonical_path);
    }
}

pub fn render_permission_session_closed(project: &Project, revoked_count: usize) {
    outln!(
        "Session cleanup for project {}: revoked {} temporary permission(s).",
        project.name,
        revoked_count
    );
}

pub fn render_search_results(outcome: &SearchOutcome) {
    outln!(
        "Search: \"{}\"  ({} result(s))",
        outcome.query_text,
        outcome.results.len()
    );
    outln!();

    for (index, result) in outcome.results.iter().enumerate() {
        outln!(
            "{}. [{}] {}   score: {:.1}",
            index + 1,
            result.source_kind.as_str(),
            result.path.display(),
            result.relevance_score
        );
        outln!("   \"{}\"", result.snippet);
    }
}

pub fn render_no_search_results(outcome: &SearchOutcome) {
    outln!("Search: \"{}\"  (0 results)", outcome.query_text);
    outln!(
        "No documents matched. Try a different term, or check `{COMMAND_NAME} permission view` if you expected a path to be included."
    );
}

fn render_issue(issue: &ValidationIssue) {
    let severity = match issue.severity {
        ValidationSeverity::Error => "ERROR",
        ValidationSeverity::Warning => "WARN",
    };

    outln!(
        "[{}] {} ({}) at {}",
        severity,
        issue.message,
        issue.code,
        issue.path.display()
    );
    outln!("  Scope: {}:{}", issue.scope, issue.subject);
    outln!("  Fix: {}", describe_fix(&issue.suggested_fix));
}

fn describe_fix(fix: &SuggestedFix) -> String {
    match fix {
        SuggestedFix::Run { action, purpose } => {
            format!("run `{}` to {purpose}", action_command(*action))
        }
        SuggestedFix::FixThenRerun { fix, action } => {
            format!("{fix} and rerun `{}`", action_command(*action))
        }
        SuggestedFix::Manual(step) => step.clone(),
    }
}

/// The command line that performs a core `Action`.
fn action_command(action: Action) -> String {
    let subcommand = match action {
        Action::Init => "init",
        Action::Validate => "validate",
    };
    format!("{COMMAND_NAME} {subcommand}")
}

pub fn render_project_model_set(project: &Project, metadata: &ProjectMetadata) {
    match &metadata.model {
        Some(model) => outln!("Project {} now uses the model {model}.", project.name),
        None => outln!(
            "Project {} now uses the workspace's default model.",
            project.name
        ),
    }
}
