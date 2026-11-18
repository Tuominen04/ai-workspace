use crate::workspace::ValidationResult;
use std::error::Error;
use std::fmt;
use std::io;
use std::path::PathBuf;

/// An operation that refuses to run while workspace validation reports
/// blocking issues.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GatedOperation {
    Search,
    ProjectCreate,
    ProjectShow,
    ProjectSetModel,
    ProjectClone,
    PermissionGrant,
    PermissionRevoke,
    PermissionView,
}

impl GatedOperation {
    /// A short, front-end-neutral name, e.g. "project create".
    pub fn describe(self) -> &'static str {
        match self {
            Self::Search => "search",
            Self::ProjectCreate => "project create",
            Self::ProjectShow => "project show",
            Self::ProjectSetModel => "project set-model",
            Self::ProjectClone => "project clone",
            Self::PermissionGrant => "permission grant",
            Self::PermissionRevoke => "permission revoke",
            Self::PermissionView => "permission view",
        }
    }
}

#[derive(Debug)]
pub enum AppError {
    Io(io::Error),
    Message(String),
    MissingHome,
    InvalidSelection(String),
    ConfigMissing(PathBuf),
    ConfigInvalid {
        path: PathBuf,
        reason: String,
    },
    InvalidProjectName(String),
    InvalidModelName(String),
    ProjectNotFound(String),
    /// A project operation was refused because its destination exists; nothing was written.
    AlreadyExists(PathBuf),
    /// A project could not be moved to the Trash; nothing was deleted. The person may choose
    /// to erase it instead (ADR-0015).
    TrashUnavailable {
        path: PathBuf,
        reason: String,
    },
    /// A gated operation was refused because validation found blocking
    /// issues. Carries the full result so a front end can show them.
    ValidationGateBlocked {
        operation: GatedOperation,
        result: Box<ValidationResult>,
    },
    PermissionDenied(PathBuf),
    InvalidPermissionPath(String),
    PermissionGrantNotFound(String),
    MissingProjectContext,
    RuntimeRequestFailed(String),
    InvalidWorkspaceRoot {
        input: String,
        reason: String,
    },
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "{error}"),
            Self::Message(message) => f.write_str(message),
            Self::MissingHome => f.write_str("could not determine the home directory"),
            Self::InvalidSelection(message) => f.write_str(message),
            Self::ConfigMissing(path) => {
                write!(
                    f,
                    "workspace configuration file is missing: {}",
                    path.display()
                )
            }
            Self::ConfigInvalid { path, reason } => {
                write!(
                    f,
                    "workspace configuration is invalid at {}: {reason}",
                    path.display()
                )
            }
            Self::InvalidProjectName(name) => {
                write!(f, "invalid project name '{name}'")
            }
            Self::InvalidModelName(name) => write!(
                f,
                "invalid model name '{name}': use an Ollama model name such as sec or qwen3:8b, without spaces"
            ),
            Self::ProjectNotFound(project) => write!(f, "project '{project}' was not found"),
            Self::AlreadyExists(path) => write!(f, "Already exists: {}", path.display()),
            Self::TrashUnavailable { path, reason } => write!(
                f,
                "cannot move {} to the Trash: {reason}; nothing was deleted",
                path.display()
            ),
            Self::ValidationGateBlocked { operation, .. } => write!(
                f,
                "cannot run {} because workspace validation has blocking issues",
                operation.describe()
            ),
            Self::PermissionDenied(path) => {
                write!(f, "permission denied while accessing {}", path.display())
            }
            Self::InvalidPermissionPath(path) => {
                write!(f, "invalid permission path '{path}'")
            }
            Self::PermissionGrantNotFound(path) => {
                write!(f, "no active temporary permission found for '{path}'")
            }
            Self::MissingProjectContext => f.write_str(
                "this needs a project; name one, or keep only one project in the workspace",
            ),
            Self::RuntimeRequestFailed(reason) => {
                write!(f, "request to the AI runtime failed: {reason}")
            }
            Self::InvalidWorkspaceRoot { input, reason } => {
                write!(f, "cannot use '{input}' as the workspace folder: {reason}")
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
