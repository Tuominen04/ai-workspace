use crate::config::WorkspaceConfig;
use crate::context::ContextSnapshot;
use crate::error::{AppError, Result};
use crate::paths::WorkspacePaths;
use crate::runtime::RuntimeStatus;
use crate::search::SearchOutcome;
use crate::workspace::{
    CreatedProject, PermissionCategory, PermissionGrantResult, PermissionRevokeResult,
    PermissionStatus, PermissionViewSnapshot, Project, ProjectMetadata, ValidationIssue,
    ValidationResult, ValidationSeverity,
};
use std::io::{self, Write};

pub fn render_startup_banner(paths: &WorkspacePaths) {
    println!("AI Workspace");
    println!("Root: {}", paths.root.display());
}

pub fn render_projects(projects: &[Project]) {
    println!("Available workspaces:");
    for (index, project) in projects.iter().enumerate() {
        println!("  {}. {}", index + 1, project.name);
    }
}

pub fn prompt_for_selection(projects: &[Project]) -> Result<usize> {
    if projects.len() == 1 {
        return Ok(0);
    }

    loop {
        print!("Select workspace [1-{}] or q to quit: ", projects.len());
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let value = input.trim();

        if value.eq_ignore_ascii_case("q") || value.eq_ignore_ascii_case("quit") {
            return Err(AppError::Message("Session cancelled by user".to_string()));
        }

        match value.parse::<usize>() {
            Ok(number) if number >= 1 && number <= projects.len() => return Ok(number - 1),
            _ => println!(
                "Invalid selection. Enter a number between 1 and {}.",
                projects.len()
            ),
        }
    }
}

pub fn render_session_summary(
    project: &Project,
    context: &ContextSnapshot,
    runtime: &RuntimeStatus,
) {
    println!();
    println!("Workspace: {}", project.name);
    println!("Project path: {}", project.path.display());
    println!("Runtime: {}", runtime.detail);
    println!("Base context files: {}", context.base_files.len());
    println!("Project context files: {}", context.project_files.len());
    println!("Session context files: {}", context.session_files.len());
    println!("Total context sources: {}", context.source_count);
    println!("AI access policy: read-only base scope (~/AI/Base)");

    if !context.base_files.is_empty() {
        println!("Base context:");
        for file in &context.base_files {
            println!("  - {}", file.display());
        }
    }

    if !context.project_files.is_empty() {
        println!("Project context:");
        for file in &context.project_files {
            println!("  - {}", file.display());
        }
    }

    if !context.session_files.is_empty() {
        println!("Session context:");
        for file in &context.session_files {
            println!("  - {}", file.display());
        }
    }

    println!();
}

pub fn render_chat_ready() {
    println!("Session is active. Ask a question, or type 'exit' to close the workspace.");
}

pub fn render_no_runtime_notice() {
    println!("Session is active. Press Enter to close the workspace.");
}

pub fn wait_for_exit() -> Result<()> {
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(())
}

pub fn prompt_for_chat_message() -> Result<Option<String>> {
    loop {
        print!("> ");
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
    println!("[AI] {content}");
    println!();
}

pub fn render_chat_error(message: &str) {
    println!("Could not reach the AI runtime: {message}");
    println!("You can try again, or type 'exit' to close the workspace.");
}

pub fn render_status(paths: &WorkspacePaths, projects: &[Project]) {
    println!("AI Workspace status");
    println!("Root: {}", paths.root.display());
    println!("Base: {}", paths.base.display());
    println!("Projects: {}", paths.projects.display());
    println!("Discovered workspaces: {}", projects.len());

    for project in projects {
        println!("- {} ({})", project.name, project.path.display());
    }
}

pub fn render_init(paths: &WorkspacePaths, config_path: &std::path::Path, config_created: bool) {
    println!("Initialized workspace root at {}", paths.root.display());
    println!("Base layout: {}", paths.base.display());
    println!("Projects directory: {}", paths.projects.display());
    if config_created {
        println!("Workspace config created: {}", config_path.display());
    } else {
        println!("Workspace config kept: {}", config_path.display());
    }
}

pub fn render_workspace_config(config: &WorkspaceConfig) {
    println!("Workspace configuration");
    println!("Root: {}", config.root_path);
    println!("Base: {}", config.base_path);
    println!("Projects: {}", config.projects_path);
    println!("Cache: {}", config.cache_path);
    println!("Logs: {}", config.logs_path);
    println!("Config: {}", config.config_path);
    println!("Project metadata file: {}", config.project_metadata_file);
    println!("Default project directories:");
    for directory in &config.default_project_directories {
        println!("- {}", directory);
    }
}

pub fn render_project_created(project: &CreatedProject) {
    println!("Project created: {}", project.project.name);
    println!("Path: {}", project.project.path.display());
    println!("Description: {}", project.metadata.description);
}

pub fn render_project_details(project: &Project, metadata: &ProjectMetadata) {
    println!("Project: {}", project.name);
    println!("Path: {}", project.path.display());
    println!("Display name: {}", metadata.display_name);
    println!("Description: {}", metadata.description);
    println!("Created at: {}", metadata.created_at);
    println!("Knowledge dir: {}", metadata.knowledge_dir);
    println!("Specs dir: {}", metadata.specs_dir);
    println!("Resources dir: {}", metadata.resources_dir);
    println!("Session dir: {}", metadata.session_dir);
}

pub fn render_validation_result(result: &ValidationResult) {
    println!("Workspace validation");
    println!("Checked projects: {}", result.checked_projects);

    if result.valid {
        println!("Status: PASS");
        return;
    }

    println!(
        "Status: FAIL (blocking issues: {})",
        result.blocking_issue_count()
    );

    for issue in &result.workspace_issues {
        render_issue(issue);
    }

    for project in &result.project_issues {
        println!("Project issues: {}", project.project_id);
        for issue in &project.issues {
            render_issue(issue);
        }
    }

    println!("Gated commands blocked: ai project create, ai project show");
    println!("Run `ai validate` after fixes to clear blocking issues.");
}

pub fn render_validation_gate_block(action: &str, result: &ValidationResult) {
    println!(
        "Cannot run {}: workspace validation has {} blocking issue(s).",
        action,
        result.blocking_issue_count()
    );
    println!("Run `ai validate` to inspect and clear blocking issues first.");
}

pub fn render_permission_grant(project: &Project, result: &PermissionGrantResult) {
    println!("Project: {}", project.name);
    if result.created {
        println!("Permission granted: {}", result.grant.canonical_path);
    } else {
        println!(
            "Permission already active (idempotent): {}",
            result.grant.canonical_path
        );
    }
}

pub fn render_permission_revoke(project: &Project, result: &PermissionRevokeResult) {
    println!("Project: {}", project.name);
    println!("Permission revoked: {}", result.revoked_path);
}

pub fn render_permission_view(project: &Project, snapshot: &PermissionViewSnapshot) {
    println!("Project: {}", project.name);
    println!("Session: {}", snapshot.session_id);
    println!(
        "Permissions: total={} page={} page_size={}",
        snapshot.total_count, snapshot.page, snapshot.page_size
    );

    if snapshot.entries.is_empty() {
        println!("No permissions in this view.");
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
        println!("- [{}][{}] {}", category, status, grant.canonical_path);
    }
}

pub fn render_permission_session_closed(project: &Project, revoked_count: usize) {
    println!(
        "Session cleanup for project {}: revoked {} temporary permission(s).",
        project.name, revoked_count
    );
}

pub fn render_search_results(outcome: &SearchOutcome) {
    println!(
        "Search: \"{}\"  ({} result(s))",
        outcome.query_text,
        outcome.results.len()
    );
    println!();

    for (index, result) in outcome.results.iter().enumerate() {
        println!(
            "{}. [{}] {}   score: {:.1}",
            index + 1,
            result.source_kind.as_str(),
            result.path.display(),
            result.relevance_score
        );
        println!("   \"{}\"", result.snippet);
    }
}

pub fn render_no_search_results(outcome: &SearchOutcome) {
    println!("Search: \"{}\"  (0 results)", outcome.query_text);
    println!(
        "No documents matched. Try a different term, or check `ai permission view` if you expected a path to be included."
    );
}

fn render_issue(issue: &ValidationIssue) {
    let severity = match issue.severity {
        ValidationSeverity::Error => "ERROR",
        ValidationSeverity::Warning => "WARN",
    };

    println!(
        "[{}] {} ({}) at {}",
        severity,
        issue.message,
        issue.code,
        issue.path.display()
    );
    println!("  Scope: {}:{}", issue.scope, issue.subject);
    println!("  Fix: {}", issue.suggested_fix);
}
