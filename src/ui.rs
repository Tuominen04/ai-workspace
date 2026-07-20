use crate::context::ContextSnapshot;
use crate::error::{AppError, Result};
use crate::paths::WorkspacePaths;
use crate::runtime::RuntimeStatus;
use crate::workspace::Project;
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

pub fn render_init(paths: &WorkspacePaths) {
    println!("Initialized workspace root at {}", paths.root.display());
    println!("Base layout: {}", paths.base.display());
    println!("Projects directory: {}", paths.projects.display());
}
