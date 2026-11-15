//! The commands the interface can call.
//!
//! Each one opens the workspace, calls one `Engine` or `Session` method, and converts the
//! result to a wire type. The logic lives in plain functions taking an `Engine` (or the open
//! session), so tests drive them against a temporary workspace without a window; the
//! `#[tauri::command]` wrappers only resolve the workspace and reach the shell's state.
//!
//! Every command is `async` so it runs off the main thread: opening a session can start
//! Ollama, and a reply can take as long as the model needs, and neither may freeze the window.

use crate::dto::{
    CommandError, GrantDone, InitDone, PermissionPage, ProjectDetails, RevokeDone, RootSaved,
    SearchResults, SessionClosed, SessionInfo, Validation, WorkspaceSettings, WorkspaceStatus,
};
use ai_workspace_core::workspace::PermissionViewOptions;
use ai_workspace_core::{Engine, Session};
use std::sync::Mutex;

type Outcome<T> = Result<T, CommandError>;

/// Search results shown when the interface asks for no particular number.
pub const DEFAULT_SEARCH_LIMIT: u32 = 10;

/// The one open session, if any. Opening another closes it; so does quitting the app.
#[derive(Default)]
pub struct OpenSession(pub Mutex<Option<Session>>);

pub fn status(engine: &Engine) -> Outcome<WorkspaceStatus> {
    let projects = engine.projects()?;
    Ok(WorkspaceStatus::new(
        engine.paths(),
        engine.root_source(),
        &projects,
    ))
}

pub fn settings(engine: &Engine) -> Outcome<WorkspaceSettings> {
    let config = engine.config()?;
    Ok(WorkspaceSettings::new(
        engine.paths(),
        engine.root_source(),
        &config,
    ))
}

pub fn project_details(engine: &Engine, name: &str) -> Outcome<ProjectDetails> {
    let (project, metadata) = engine.project_details(name)?;
    Ok(ProjectDetails::new(&project, &metadata))
}

pub fn validate(engine: &Engine) -> Outcome<Validation> {
    Ok(Validation::from(&engine.validate()?))
}

pub fn search(
    engine: &Engine,
    query: &str,
    project: Option<&str>,
    limit: Option<u32>,
) -> Outcome<SearchResults> {
    if query.trim().is_empty() {
        return Err(CommandError::shell("type something to search for"));
    }
    let limit = limit.unwrap_or(DEFAULT_SEARCH_LIMIT).max(1) as usize;
    Ok(SearchResults::from(&engine.search(query, project, limit)?))
}

/// Grants shown per page when the interface asks for no particular size.
pub const DEFAULT_PAGE_SIZE: u32 = 25;

pub fn init(engine: &Engine) -> Outcome<InitDone> {
    let report = engine.init()?;
    Ok(InitDone {
        root: engine.paths().root.display().to_string(),
        config_file: report.config_path.display().to_string(),
        config_created: report.config_created,
    })
}

pub fn create_project(
    engine: &Engine,
    name: &str,
    description: Option<&str>,
) -> Outcome<ProjectDetails> {
    let description = description.map(str::trim).filter(|text| !text.is_empty());
    let created = engine.create_project(name, description)?;
    Ok(ProjectDetails::new(&created.project, &created.metadata))
}

pub fn permissions(
    engine: &Engine,
    project: &str,
    filter: Option<&str>,
    page: Option<u32>,
    page_size: Option<u32>,
) -> Outcome<PermissionPage> {
    let options = PermissionViewOptions {
        filter: filter
            .map(str::trim)
            .filter(|text| !text.is_empty())
            .map(str::to_string),
        page: page.unwrap_or(1).max(1) as usize,
        page_size: page_size.unwrap_or(DEFAULT_PAGE_SIZE).max(1) as usize,
    };
    let (project, snapshot) = engine.permissions(Some(project), options)?;
    Ok(PermissionPage::new(&project, &snapshot))
}

pub fn grant(engine: &Engine, project: &str, path: &str) -> Outcome<GrantDone> {
    let path = path.trim();
    if path.is_empty() {
        return Err(CommandError::shell("type or paste the path to grant"));
    }
    let (project, result) = engine.grant(Some(project), path)?;
    Ok(GrantDone {
        project: project.name,
        path: result.grant.canonical_path,
        created: result.created,
    })
}

pub fn revoke(engine: &Engine, project: &str, path: &str) -> Outcome<RevokeDone> {
    let (project, result) = engine.revoke(Some(project), path)?;
    Ok(RevokeDone {
        project: project.name,
        path: result.revoked_path,
    })
}

/// Saves `path` as the workspace folder. A chat session belongs to the folder it was opened
/// in, so an open one is ended first, revoking its temporary grants there.
pub fn set_root(open: &OpenSession, path: &str) -> Outcome<RootSaved> {
    let path = path.trim();
    if path.is_empty() {
        return Err(CommandError::shell("type or paste the folder to use"));
    }
    let session_closed = session_close(open)?;
    let report = ai_workspace_core::set_workspace_root(path)?;
    Ok(RootSaved {
        root: report.root.display().to_string(),
        config_file: report.config_file.display().to_string(),
        env_override: report.env_override.map(|p| p.display().to_string()),
        session_closed,
    })
}

/// Opens a session in `name`, closing any session already open first.
pub fn session_open(engine: &Engine, open: &OpenSession, name: &str) -> Outcome<SessionInfo> {
    let mut slot = lock(open)?;
    if let Some(previous) = slot.take() {
        previous.close()?;
    }

    let projects = engine.projects()?;
    let project = projects
        .iter()
        .find(|project| project.name == name)
        .ok_or_else(|| {
            CommandError::from(ai_workspace_core::AppError::ProjectNotFound(name.into()))
        })?;

    let session = Session::open(engine, project)?;
    let summary = session.summary();
    let info = SessionInfo::new(summary.project, summary.context, summary.runtime);
    *slot = Some(session);
    Ok(info)
}

/// Sends one message in the open session and returns the reply.
pub fn session_send(open: &OpenSession, message: &str) -> Outcome<String> {
    let mut slot = lock(open)?;
    let session = slot
        .as_mut()
        .ok_or_else(|| CommandError::shell("no session is open"))?;
    Ok(session.send(message)?)
}

/// Closes the open session, if there is one.
pub fn session_close(open: &OpenSession) -> Outcome<Option<SessionClosed>> {
    let mut slot = lock(open)?;
    let Some(session) = slot.take() else {
        return Ok(None);
    };
    let project = session.summary().project.name.clone();
    let revoked = session.close()?;
    Ok(Some(SessionClosed {
        project,
        revoked: u32::try_from(revoked).unwrap_or(u32::MAX),
    }))
}

fn lock(open: &OpenSession) -> Outcome<std::sync::MutexGuard<'_, Option<Session>>> {
    open.0.lock().map_err(|_| {
        CommandError::shell(
            "the session state is unusable after an earlier failure; restart the app",
        )
    })
}

// ---- Tauri wrappers -------------------------------------------------------------------------

/// The workspace, resolved afresh on each call so a changed setting is picked up without a
/// restart. Resolving touches nothing on disk.
fn engine() -> Outcome<Engine> {
    Ok(Engine::open()?)
}

#[tauri::command]
pub async fn workspace_status() -> Outcome<WorkspaceStatus> {
    status(&engine()?)
}

#[tauri::command]
pub async fn workspace_config() -> Outcome<WorkspaceSettings> {
    settings(&engine()?)
}

#[tauri::command]
pub async fn workspace_validate() -> Outcome<Validation> {
    validate(&engine()?)
}

#[tauri::command]
pub async fn project_show(name: String) -> Outcome<ProjectDetails> {
    project_details(&engine()?, &name)
}

#[tauri::command]
pub async fn workspace_search(
    query: String,
    project: Option<String>,
    limit: Option<u32>,
) -> Outcome<SearchResults> {
    search(&engine()?, &query, project.as_deref(), limit)
}

#[tauri::command]
pub async fn workspace_init() -> Outcome<InitDone> {
    init(&engine()?)
}

#[tauri::command]
pub async fn workspace_set_root(
    open: tauri::State<'_, OpenSession>,
    path: String,
) -> Outcome<RootSaved> {
    set_root(&open, &path)
}

#[tauri::command]
pub async fn project_create(name: String, description: Option<String>) -> Outcome<ProjectDetails> {
    create_project(&engine()?, &name, description.as_deref())
}

#[tauri::command]
pub async fn permission_list(
    project: String,
    filter: Option<String>,
    page: Option<u32>,
    page_size: Option<u32>,
) -> Outcome<PermissionPage> {
    permissions(&engine()?, &project, filter.as_deref(), page, page_size)
}

#[tauri::command]
pub async fn permission_grant(project: String, path: String) -> Outcome<GrantDone> {
    grant(&engine()?, &project, &path)
}

#[tauri::command]
pub async fn permission_revoke(project: String, path: String) -> Outcome<RevokeDone> {
    revoke(&engine()?, &project, &path)
}

#[tauri::command]
pub async fn chat_open(
    open: tauri::State<'_, OpenSession>,
    project: String,
) -> Outcome<SessionInfo> {
    session_open(&engine()?, &open, &project)
}

#[tauri::command]
pub async fn chat_send(open: tauri::State<'_, OpenSession>, message: String) -> Outcome<String> {
    session_send(&open, &message)
}

#[tauri::command]
pub async fn chat_close(open: tauri::State<'_, OpenSession>) -> Outcome<Option<SessionClosed>> {
    session_close(&open)
}
