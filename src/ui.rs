use crate::context::ContextSnapshot;
use crate::config::WorkspaceConfig;
use crate::error::{AppError, Result};
use crate::paths::WorkspacePaths;
use crate::runtime::RuntimeStatus;
use crate::workspace::{CreatedProject, Project, ProjectMetadata, ValidationIssue, ValidationResult, ValidationSeverity};
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
            _ => println!("Invalid selection. Enter a number between 1 and {}.", projects.len()),
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
    println!("AI-readable base files: {}", context.base_files.len());
    println!("AI access policy: read-only base scope (~/AI/Base)");

    if !context.base_files.is_empty() {
        println!("Base context:");
        for file in &context.base_files {
            println!("  - {}", file.display());
        }
    }

    println!();
    println!("Session is active. Press Enter to close the workspace.");
}

pub fn wait_for_exit() -> Result<()> {
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(())
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

    println!("Status: FAIL (blocking issues: {})", result.blocking_issue_count());

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
