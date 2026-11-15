use crate::error::Result;
use crate::paths::WorkspacePaths;
use crate::search::{self, SourceKind};
use crate::workspace::{MarkdownScan, Project, collect_markdown_files, scan_markdown_files};
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
    /// Files and folders left out because their names look like secrets (ADR-0012).
    pub skipped_files: Vec<PathBuf>,
}

impl ContextSnapshot {
    pub fn as_system_prompt(&self) -> String {
        format!(
            "You are assisting inside the AI Workspace tool. Use the following workspace context to answer the user's questions. If the context does not cover something, say so instead of guessing.\n{}",
            self.assembled_content
        )
    }

    /// The system message for a model with its own instructions (ADR-0012): the model's
    /// `SYSTEM` text first, unchanged, then the workspace context. Ollama uses a request's
    /// system message in place of the Modelfile's, so leaving the instructions out would
    /// drop the agent's rules. Without instructions this is [`Self::as_system_prompt`].
    pub fn as_system_prompt_for(&self, agent_instructions: Option<&str>) -> String {
        match agent_instructions {
            Some(instructions) => format!("{instructions}\n\n---\n\n{}", self.as_system_prompt()),
            None => self.as_system_prompt(),
        }
    }
}

pub fn build_context_snapshot(
    paths: &WorkspacePaths,
    project: Option<&Project>,
    query: Option<&str>,
) -> Result<ContextSnapshot> {
    let mut skipped_files = Vec::new();
    let mut take = |root: &Path, scan: MarkdownScan| {
        skipped_files.extend(scan.skipped.iter().map(|path| root.join(path)));
        scan.files
    };
    let base_files = take(
        paths.base_knowledge_path(),
        scan_markdown_files(paths.base_knowledge_path(), 2)?,
    );
    let mut project_files = Vec::new();
    let mut session_files = Vec::new();
    let project_root = project.map(|project| project.path.clone());

    if let Some(project) = project {
        let knowledge = project.path.join("Knowledge");
        project_files = take(
            &knowledge,
            scan_markdown_files(&knowledge, 2).unwrap_or_default(),
        );
        let session = project.path.join("Session");
        session_files = take(
            &session,
            scan_markdown_files(&session, 2).unwrap_or_default(),
        );
    }

    let mut all_sources = Vec::new();
    for file in base_files.iter() {
        all_sources.push((
            "base".to_string(),
            resolve_source_path(paths.base_knowledge_path(), file),
        ));
    }
    for file in project_files.iter() {
        let project_root = project_root.as_deref().unwrap_or_else(|| Path::new(""));
        all_sources.push((
            "project".to_string(),
            resolve_source_path(project_root.join("Knowledge").as_path(), file),
        ));
    }
    for file in session_files.iter() {
        let project_root = project_root.as_deref().unwrap_or_else(|| Path::new(""));
        all_sources.push((
            "session".to_string(),
            resolve_source_path(project_root.join("Session").as_path(), file),
        ));
    }

    let ordered_sources: Vec<(String, PathBuf)> = match query {
        Some(text) => {
            let candidates: Vec<search::SearchCandidate> = all_sources
                .iter()
                .map(|(kind, path)| search::SearchCandidate {
                    source_kind: source_kind_from_tag(kind),
                    path: path.clone(),
                })
                .collect();

            search::order_by_relevance(&candidates, text)
                .into_iter()
                .map(|result| (result.source_kind.as_str().to_string(), result.path))
                .collect()
        }
        None => {
            let mut sources = all_sources;
            sources.sort_by(|left, right| {
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
            sources
        }
    };

    let mut selected_sources = Vec::new();
    let mut assembled_content = String::new();
    let mut total_bytes = 0usize;

    for (kind, path) in ordered_sources.into_iter() {
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
        skipped_files,
    })
}

fn source_kind_from_tag(kind: &str) -> SourceKind {
    match kind {
        "project" => SourceKind::Project,
        "session" => SourceKind::Session,
        _ => SourceKind::Base,
    }
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
