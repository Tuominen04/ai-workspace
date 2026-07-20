use crate::error::Result;
use crate::paths::WorkspacePaths;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Project {
    pub name: String,
    pub path: PathBuf,
}

pub fn discover_projects(paths: &WorkspacePaths) -> Result<Vec<Project>> {
    if !paths.projects.exists() {
        return Ok(Vec::new());
    }

    let mut projects = Vec::new();
    for entry in fs::read_dir(&paths.projects)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        if !file_type.is_dir() {
            continue;
        }

        let path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();
        projects.push(Project { name, path });
    }

    projects.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(projects)
}

pub fn collect_markdown_files(root: &Path, max_depth: usize) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    collect_markdown_files_recursive(root, root, max_depth, &mut files)?;
    files.sort();
    Ok(files)
}

fn collect_markdown_files_recursive(
    root: &Path,
    current: &Path,
    depth_left: usize,
    files: &mut Vec<PathBuf>,
) -> Result<()> {
    if depth_left == 0 {
        return Ok(());
    }

    for entry in fs::read_dir(current)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let path = entry.path();

        if file_type.is_file() {
            if path.extension().is_some_and(|ext| ext == "md") {
                files.push(path.strip_prefix(root).unwrap_or(&path).to_path_buf());
            }
            continue;
        }

        if file_type.is_dir() {
            collect_markdown_files_recursive(root, &path, depth_left - 1, files)?;
        }
    }

    Ok(())
}
