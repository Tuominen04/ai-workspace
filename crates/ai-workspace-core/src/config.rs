use crate::error::{AppError, Result};
use crate::paths::WorkspacePaths;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceConfig {
    pub root_path: String,
    pub base_path: String,
    pub projects_path: String,
    pub cache_path: String,
    pub logs_path: String,
    pub config_path: String,
    pub default_project_directories: Vec<String>,
    pub project_metadata_file: String,
    #[serde(default = "default_model_value")]
    pub default_model: String,
}

fn default_model_value() -> String {
    "qwen3.6:latest".to_string()
}

#[derive(Debug, Clone)]
pub struct ConfigBootstrap {
    pub path: PathBuf,
    pub created: bool,
}

impl WorkspaceConfig {
    pub fn default_for_paths(paths: &WorkspacePaths) -> Self {
        Self {
            root_path: paths.root.display().to_string(),
            base_path: paths.base.display().to_string(),
            projects_path: paths.projects.display().to_string(),
            cache_path: paths.cache.display().to_string(),
            logs_path: paths.logs.display().to_string(),
            config_path: paths.config.display().to_string(),
            default_project_directories: vec![
                "Knowledge".to_string(),
                "Specs".to_string(),
                "Resources".to_string(),
                "Session".to_string(),
            ],
            project_metadata_file: "project.toml".to_string(),
            default_model: default_model_value(),
        }
    }

    pub fn metadata_filename(&self) -> &str {
        self.project_metadata_file.as_str()
    }
}

pub fn load_workspace_config(paths: &WorkspacePaths) -> Result<WorkspaceConfig> {
    let config_path = paths.workspace_config_file();
    if !config_path.exists() {
        return Err(AppError::ConfigMissing(config_path));
    }

    let raw =
        fs::read_to_string(&config_path).map_err(|error| map_path_error(&config_path, error))?;
    toml::from_str::<WorkspaceConfig>(&raw).map_err(|error| AppError::ConfigInvalid {
        path: config_path,
        reason: error.to_string(),
    })
}

pub fn ensure_default_workspace_config(
    paths: &WorkspacePaths,
    overwrite: bool,
) -> Result<ConfigBootstrap> {
    let config_path = paths.workspace_config_file();
    if config_path.exists() && !overwrite {
        return Ok(ConfigBootstrap {
            path: config_path,
            created: false,
        });
    }

    let config = WorkspaceConfig::default_for_paths(paths);
    let serialized = toml::to_string_pretty(&config).map_err(|error| {
        AppError::Message(format!("failed to serialize workspace config: {error}"))
    })?;
    fs::write(&config_path, serialized).map_err(|error| map_path_error(&config_path, error))?;

    Ok(ConfigBootstrap {
        path: config_path,
        created: true,
    })
}

fn map_path_error(path: &std::path::Path, error: std::io::Error) -> AppError {
    if error.kind() == std::io::ErrorKind::PermissionDenied {
        return AppError::PermissionDenied(path.to_path_buf());
    }
    AppError::Io(error)
}
