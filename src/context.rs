use crate::error::Result;
use crate::paths::WorkspacePaths;
use crate::workspace::collect_markdown_files;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct ContextSnapshot {
    pub base_files: Vec<PathBuf>,
}

pub fn build_context_snapshot(paths: &WorkspacePaths) -> Result<ContextSnapshot> {
    let base_files = collect_markdown_files(paths.base_knowledge_path(), 2)?;

    Ok(ContextSnapshot { base_files })
}
