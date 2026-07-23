use crate::error::{AppError, Result};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const AI_ROOT_NAME: &str = "AI";

#[derive(Debug, Clone)]
pub struct WorkspacePaths {
    pub root: PathBuf,
    pub base: PathBuf,
    pub projects: PathBuf,
    pub cache: PathBuf,
    pub logs: PathBuf,
    pub config: PathBuf,
}

impl WorkspacePaths {
    pub fn detect() -> Result<Self> {
        let root = if let Some(explicit_root) = env::var_os("AI_WORKSPACE_HOME") {
            PathBuf::from(explicit_root)
        } else {
            home_dir()?.join(AI_ROOT_NAME)
        };

        Ok(Self::from_root(root))
    }

    pub fn from_root(root: PathBuf) -> Self {
        Self {
            base: root.join("Base"),
            projects: root.join("Projects"),
            cache: root.join("Cache"),
            logs: root.join("Logs"),
            config: root.join("Config"),
            root,
        }
    }

    pub fn ensure_layout(&self) -> Result<()> {
        fs::create_dir_all(&self.root)?;
        fs::create_dir_all(&self.base)?;
        fs::create_dir_all(&self.projects)?;
        fs::create_dir_all(&self.cache)?;
        fs::create_dir_all(&self.logs)?;
        fs::create_dir_all(&self.config)?;

        for section in ["Common", "Identity", "Linux", "Fedora", "Development"] {
            fs::create_dir_all(self.base.join(section))?;
        }

        Ok(())
    }

    pub fn base_knowledge_path(&self) -> &Path {
        &self.base
    }

    pub fn workspace_config_file(&self) -> PathBuf {
        self.config.join("workspace.toml")
    }

    pub fn project_metadata_file(&self, project_root: &Path, metadata_filename: &str) -> PathBuf {
        project_root.join(metadata_filename)
    }

    pub fn project_permissions_file(&self, project_root: &Path) -> PathBuf {
        project_root.join("Session").join("permissions.toml")
    }
}

pub fn canonical_local_existing_path(input: &str) -> Result<PathBuf> {
    let candidate = PathBuf::from(input);
    if !candidate.exists() {
        return Err(AppError::InvalidPermissionPath(input.to_string()));
    }

    std::fs::canonicalize(&candidate).map_err(|_| AppError::InvalidPermissionPath(input.to_string()))
}

fn home_dir() -> Result<PathBuf> {
    env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or(AppError::MissingHome)
}
