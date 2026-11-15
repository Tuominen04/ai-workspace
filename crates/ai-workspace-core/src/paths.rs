use crate::error::{AppError, Result};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

const AI_ROOT_NAME: &str = "AI";

/// Environment variable that overrides every other way of choosing the
/// workspace folder. Mostly useful for testing against a scratch folder.
pub const WORKSPACE_ROOT_ENV: &str = "AI_WORKSPACE_HOME";

const USER_CONFIG_DIR_NAME: &str = "ai-workspace";
const USER_CONFIG_FILE_NAME: &str = "config.toml";

/// Where the workspace folder was chosen from, highest priority first.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RootSource {
    /// The `AI_WORKSPACE_HOME` environment variable.
    Environment,
    /// The `workspace_root` setting in the user's config file.
    UserConfig(PathBuf),
    /// Neither was set, so `~/AI` is used.
    Default,
}

impl RootSource {
    pub fn describe(&self) -> String {
        match self {
            Self::Environment => format!("environment variable {WORKSPACE_ROOT_ENV}"),
            Self::UserConfig(path) => format!("user config file {}", path.display()),
            Self::Default => "default location (~/AI)".to_string(),
        }
    }
}

/// The per-user settings file, `~/.config/ai-workspace/config.toml`. It lives
/// outside the workspace because the tool has to find it before it knows
/// where the workspace is.
#[derive(Debug, Default, Serialize, Deserialize)]
struct UserConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    workspace_root: Option<String>,
}

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
        Ok(Self::detect_with_source()?.0)
    }

    /// Resolves the workspace folder and reports where the choice came from:
    /// the `AI_WORKSPACE_HOME` variable, else the user config file's
    /// `workspace_root`, else `~/AI`.
    pub fn detect_with_source() -> Result<(Self, RootSource)> {
        if let Some(explicit_root) = env_root_override() {
            return Ok((Self::from_root(explicit_root), RootSource::Environment));
        }
        Self::detect_saved_with_source()
    }

    /// Resolves the workspace folder as if `AI_WORKSPACE_HOME` were not set:
    /// the user config file's `workspace_root`, else `~/AI`. For a front end
    /// that has been asked to use the saved folder for the rest of its run.
    pub fn detect_saved_with_source() -> Result<(Self, RootSource)> {
        let config_file = user_config_file()?;
        if let Some(root) = read_user_root(&config_file)? {
            return Ok((Self::from_root(root), RootSource::UserConfig(config_file)));
        }

        Ok((
            Self::from_root(home_dir()?.join(AI_ROOT_NAME)),
            RootSource::Default,
        ))
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

/// The folder named by `AI_WORKSPACE_HOME`, if that variable is set.
pub fn env_root_override() -> Option<PathBuf> {
    env::var_os(WORKSPACE_ROOT_ENV).map(PathBuf::from)
}

/// Path of the per-user settings file: `$XDG_CONFIG_HOME/ai-workspace/config.toml`
/// when that variable holds an absolute path, else `~/.config/ai-workspace/config.toml`.
pub fn user_config_file() -> Result<PathBuf> {
    Ok(platform::user_config_dir()?
        .join(USER_CONFIG_DIR_NAME)
        .join(USER_CONFIG_FILE_NAME))
}

/// Reads `workspace_root` from the user config file. A missing file or a file
/// without the setting is not an error (`None`); an unreadable or malformed
/// file, or a relative path, is.
pub fn read_user_root(config_file: &Path) -> Result<Option<PathBuf>> {
    let invalid = |reason: String| AppError::ConfigInvalid {
        path: config_file.to_path_buf(),
        reason,
    };

    let text = match fs::read_to_string(config_file) {
        Ok(text) => text,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(invalid(error.to_string())),
    };
    let settings: UserConfig = toml::from_str(&text).map_err(|error| invalid(error.to_string()))?;

    let Some(root) = settings.workspace_root else {
        return Ok(None);
    };
    let root = PathBuf::from(root);
    if !root.is_absolute() {
        return Err(invalid(format!(
            "workspace_root must be an absolute path, got '{}'",
            root.display()
        )));
    }
    Ok(Some(root))
}

/// Saves `root` as the workspace folder in the user config file, creating the
/// file and its folder if needed. `root` must already be absolute.
pub fn write_user_root(config_file: &Path, root: &Path) -> Result<()> {
    let root = root.to_str().ok_or_else(|| {
        AppError::Message(format!(
            "workspace folder is not valid UTF-8: {}",
            root.display()
        ))
    })?;
    let body = toml::to_string(&UserConfig {
        workspace_root: Some(root.to_string()),
    })
    .map_err(|error| AppError::Message(format!("could not write user config: {error}")))?;

    if let Some(parent) = config_file.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(
        config_file,
        format!("# ai-workspace user settings, written by `ai-workspace config set-root`.\n{body}"),
    )?;
    Ok(())
}

/// Validates a folder the user wants as the workspace root: a leading `~` is
/// expanded, and the folder must exist and be a directory. Returns its
/// canonical absolute path.
pub fn canonical_workspace_root(input: &str) -> Result<PathBuf> {
    let reject = |reason: String| AppError::InvalidWorkspaceRoot {
        input: input.to_string(),
        reason,
    };

    let expanded = if input == "~" {
        home_dir()?
    } else if let Some(rest) = input.strip_prefix("~/") {
        home_dir()?.join(rest)
    } else {
        PathBuf::from(input)
    };

    let canonical = fs::canonicalize(&expanded).map_err(|_| {
        reject(format!(
            "it does not exist; create it first, for example with `mkdir -p '{input}'`"
        ))
    })?;
    if !canonical.is_dir() {
        return Err(reject("it is not a directory".to_string()));
    }
    Ok(canonical)
}

pub fn canonical_local_existing_path(input: &str) -> Result<PathBuf> {
    let candidate = PathBuf::from(input);
    if !candidate.exists() {
        return Err(AppError::InvalidPermissionPath(input.to_string()));
    }

    std::fs::canonicalize(&candidate)
        .map_err(|_| AppError::InvalidPermissionPath(input.to_string()))
}

use platform::home_dir;

/// Where the operating system keeps a user's folders. Every platform
/// difference in folder lookup lives here, so supporting another system means
/// changing this module only. Today only the Linux/XDG behavior exists
/// (ADR-0008).
mod platform {
    use crate::error::{AppError, Result};
    use std::env;
    use std::path::PathBuf;

    /// The user's home folder, from `HOME`.
    pub(super) fn home_dir() -> Result<PathBuf> {
        env::var_os("HOME")
            .map(PathBuf::from)
            .ok_or(AppError::MissingHome)
    }

    /// The folder for per-user settings: `$XDG_CONFIG_HOME` when that is an
    /// absolute path, else `~/.config`.
    pub(super) fn user_config_dir() -> Result<PathBuf> {
        match env::var_os("XDG_CONFIG_HOME").map(PathBuf::from) {
            Some(dir) if dir.is_absolute() => Ok(dir),
            _ => Ok(home_dir()?.join(".config")),
        }
    }
}
