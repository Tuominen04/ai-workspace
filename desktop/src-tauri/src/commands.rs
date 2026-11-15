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
    CommandError, ProjectDetails, SearchResults, SessionClosed, SessionInfo, Validation,
    WorkspaceStatus,
};
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
