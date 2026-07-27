use crate::error::Result;
use crate::paths::WorkspacePaths;
use crate::workspace::{collect_markdown_files, Project};
use std::fs;
use std::path::{Path, PathBuf};

pub const DEFAULT_MAX_CONTEXT_SOURCES: usize = 8;
pub const DEFAULT_MAX_CONTEXT_BYTES: usize = 12000;

#[derive(Debug, Clone)]
pub struct ContextSnapshot {
    pub base_files: Vec<PathBuf>,
    pub project_files: Vec<PathBuf>,
    pub session_files: Vec<PathBuf>,
    pub assembled_content: String,
    pub source_count: usize,
    pub total_bytes: usize,
}

pub fn build_context_snapshot(paths: &WorkspacePaths, project: Option<&Project>) -> Result<ContextSnapshot> {
    let base_files = collect_markdown_files(paths.base_knowledge_path(), 2)?;
    let mut project_files = Vec::new();
    let mut session_files = Vec::new();
    let project_root = project.map(|project| project.path.clone());

    if let Some(project) = project {
        project_files = collect_markdown_files(&project.path.join("Knowledge"), 2).unwrap_or_default();
        session_files = collect_markdown_files(&project.path.join("Session"), 2).unwrap_or_default();
    }

    let mut all_sources = Vec::new();
    for file in base_files.iter() {
        all_sources.push(("base".to_string(), resolve_source_path(paths.base_knowledge_path(), file)));
    }
    for file in project_files.iter() {
        let project_root = project_root.as_deref().unwrap_or_else(|| Path::new(""));
        all_sources.push(("project".to_string(), resolve_source_path(project_root.join("Knowledge").as_path(), file)));
    }
    for file in session_files.iter() {
        let project_root = project_root.as_deref().unwrap_or_else(|| Path::new(""));
        all_sources.push(("session".to_string(), resolve_source_path(project_root.join("Session").as_path(), file)));
    }

    all_sources.sort_by(|left, right| {
        let rank = |kind: &str| match kind {
            "project" => 0,
            "base" => 1,
            "session" => 2,
            _ => 3,
        };
        rank(left.0.as_str())
            .cmp(&rank(right.0.as_str()))
            .then_with(|| left.1.cmp(&right.1))
    });

    let mut selected_sources = Vec::new();
    let mut assembled_content = String::new();
    let mut total_bytes = 0usize;

    for (kind, path) in all_sources.into_iter() {
        if selected_sources.len() >= DEFAULT_MAX_CONTEXT_SOURCES {
            break;
        }

        let content = match fs::read_to_string(&path) {
            Ok(content) => content,
            Err(_) => continue,
        };

        let bytes = content.len();
        if total_bytes + bytes > DEFAULT_MAX_CONTEXT_BYTES {
            continue;
        }

        selected_sources.push((kind.clone(), path.clone()));
        total_bytes += bytes;
        assembled_content.push_str(&format!("\n--- {} ---\n", path.display()));
        assembled_content.push_str(&content);
        assembled_content.push('\n');
    }

    let selected_base = selected_sources
        .iter()
        .filter(|(kind, _)| kind == "base")
        .map(|(_, path)| path.clone())
        .collect::<Vec<_>>();
    let selected_project = selected_sources
        .iter()
        .filter(|(kind, _)| kind == "project")
        .map(|(_, path)| path.clone())
        .collect::<Vec<_>>();
    let selected_session = selected_sources
        .iter()
        .filter(|(kind, _)| kind == "session")
        .map(|(_, path)| path.clone())
        .collect::<Vec<_>>();

    Ok(ContextSnapshot {
        base_files: selected_base,
        project_files: selected_project,
        session_files: selected_session,
        assembled_content,
        source_count: selected_sources.len(),
        total_bytes,
    })
}

fn resolve_source_path(root: &Path, relative_path: &Path) -> PathBuf {
    if relative_path.is_absolute() {
        return relative_path.to_path_buf();
    }

    root.join(relative_path)
}

pub fn collect_context_sources(root: &Path, max_depth: usize) -> Result<Vec<PathBuf>> {
    collect_markdown_files(root, max_depth)
}
