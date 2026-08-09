use std::error::Error;
use std::fmt;
use std::io;
use std::path::PathBuf;

#[derive(Debug)]
pub enum AppError {
    Io(io::Error),
    Message(String),
    MissingHome,
    InvalidSelection(String),
    ConfigMissing(PathBuf),
    ConfigInvalid { path: PathBuf, reason: String },
    InvalidProjectName(String),
    ProjectNotFound(String),
    ValidationGateBlocked(String),
    PermissionDenied(PathBuf),
    InvalidPermissionPath(String),
    PermissionGrantNotFound(String),
    MissingProjectContext,
    RuntimeRequestFailed(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "{error}"),
            Self::Message(message) => f.write_str(message),
            Self::MissingHome => f.write_str("could not determine the home directory"),
            Self::InvalidSelection(message) => f.write_str(message),
            Self::ConfigMissing(path) => {
                write!(f, "workspace configuration file is missing: {}", path.display())
            }
            Self::ConfigInvalid { path, reason } => {
                write!(f, "workspace configuration is invalid at {}: {reason}", path.display())
            }
            Self::InvalidProjectName(name) => {
                write!(f, "invalid project name '{name}'")
            }
            Self::ProjectNotFound(project) => write!(f, "project '{project}' was not found"),
            Self::ValidationGateBlocked(action) => write!(
                f,
                "cannot run {action} because workspace validation has blocking issues"
            ),
            Self::PermissionDenied(path) => write!(
                f,
                "permission denied while accessing {}",
                path.display()
            ),
            Self::InvalidPermissionPath(path) => {
                write!(f, "invalid permission path '{path}'")
            }
            Self::PermissionGrantNotFound(path) => {
                write!(f, "no active temporary permission found for '{path}'")
            }
            Self::MissingProjectContext => f.write_str(
                "permission command requires a project context; use --project PROJECT or keep only one project",
            ),
            Self::RuntimeRequestFailed(reason) => {
                write!(f, "request to the AI runtime failed: {reason}")
            }
        }
    }
}

impl Error for AppError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            _ => None,
        }
    }
}

impl From<io::Error> for AppError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
