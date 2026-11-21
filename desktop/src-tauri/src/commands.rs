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
    ChatReply, CommandError, GrantDone, InitDone, NoteSaved, PermissionPage, ProjectDeleted,
    ProjectDetails, ProjectMoved, ProjectRenamed, RevokeDone, RootSaved, SearchResults,
    SessionClosed, SessionInfo, SwitchDone, Validation, WorkspaceSettings, WorkspaceStatus,
};
use ai_workspace_core::workspace::{Deleted, PermissionViewOptions};
use ai_workspace_core::{Engine, Session};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, TryLockError};

type Outcome<T> = Result<T, CommandError>;

/// Search results shown when the interface asks for no particular number.
pub const DEFAULT_SEARCH_LIMIT: u32 = 10;

/// Whether the person chose **Switch now**: ignore `AI_WORKSPACE_HOME` until the app restarts
/// and use the saved folder (ADR-0011). In memory only, so a restart honours the variable again.
#[derive(Default)]
pub struct RootChoice(AtomicBool);

impl RootChoice {
    pub fn ignoring_env(&self) -> bool {
        self.0.load(Ordering::SeqCst)
    }
}

/// The workspace as this run should see it: the usual lookup, or the saved folder once the
/// person has switched. Resolved afresh on each call, touching nothing on disk.
pub fn resolve(choice: &RootChoice) -> Outcome<Engine> {
    if choice.ignoring_env() {
        Ok(Engine::open_saved()?)
    } else {
        Ok(Engine::open()?)
    }
}

/// Where the folder came from, in words, saying when the variable is being ignored.
pub fn source_text(choice: &RootChoice, described: String) -> String {
    match ai_workspace_core::paths::env_root_override() {
        Some(_) if choice.ignoring_env() => {
            format!("{described} (AI_WORKSPACE_HOME ignored until the app restarts)")
        }
        _ => described,
    }
}

/// The one open session, if any. Opening another closes it; so does quitting the app.
///
/// `slot` stays locked for as long as the model writes a reply or a note, so the window's own
/// thread never waits on it: it reads `project` instead, which is only ever locked briefly.
#[derive(Default)]
pub struct OpenSession {
    slot: Mutex<Option<Session>>,
    /// The open session's project, kept next to `slot` for checks that must not wait.
    project: Mutex<Option<String>>,
}

impl OpenSession {
    fn set_project(&self, project: Option<String>) {
        if let Ok(mut name) = self.project.lock() {
            *name = project;
        }
    }
}

/// Set once the person has answered the save-on-close question, so the next close request
/// really closes the window.
#[derive(Default)]
pub struct QuitAllowed(pub std::sync::atomic::AtomicBool);

/// Whether closing the window should first ask about saving the open chat: the project of a
/// session that has at least one answer, unless the person already answered. A session with
/// nothing answered has nothing to save, so it simply ends with the app.
///
/// Called on the window's thread, so it never waits for the session: while the model is
/// writing a reply or a note, the session is busy and there is a conversation to save, so it
/// asks at once (the answer is possible once the model is done).
pub fn needs_save_prompt(open: &OpenSession, allowed: &QuitAllowed) -> Option<String> {
    if allowed.0.load(Ordering::SeqCst) {
        return None;
    }
    match open.slot.try_lock() {
        Ok(slot) => slot
            .as_ref()
            .filter(|session| session.exchanges() > 0)
            .map(|session| session.summary().project.name.clone()),
        Err(TryLockError::WouldBlock) => open.project.lock().ok()?.clone(),
        Err(TryLockError::Poisoned(_)) => None,
    }
}

pub fn status(engine: &Engine) -> Outcome<WorkspaceStatus> {
    // A folder without a workspace is described, not created (issue #55).
    if !engine.is_initialized() {
        return Ok(WorkspaceStatus::uninitialized(
            engine.paths(),
            engine.root_source(),
        ));
    }
    let projects = engine.projects()?;
    let archived = engine.archived_projects()?;
    Ok(WorkspaceStatus::new(
        engine.paths(),
        engine.root_source(),
        &projects,
        &archived,
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

pub fn set_project_model(
    engine: &Engine,
    name: &str,
    model: Option<&str>,
) -> Outcome<ProjectDetails> {
    let (project, metadata) = engine.set_project_model(name, model)?;
    Ok(ProjectDetails::new(&project, &metadata))
}

/// The models Ollama has installed, to suggest when choosing one. Empty when Ollama is not
/// running or cannot be asked: the person can still type a name.
pub fn installed_models() -> Vec<String> {
    ai_workspace_core::runtime::RuntimeManager::new()
        .installed_models(ai_workspace_core::runtime::OLLAMA_BASE_URL)
        .unwrap_or_default()
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
pub fn set_root(choice: &RootChoice, open: &OpenSession, path: &str) -> Outcome<RootSaved> {
    let path = path.trim();
    if path.is_empty() {
        return Err(CommandError::shell("type or paste the folder to use"));
    }
    let session_closed = session_close(open)?;
    let report = ai_workspace_core::set_workspace_root(path)?;
    Ok(RootSaved {
        root: report.root.display().to_string(),
        config_file: report.config_file.display().to_string(),
        // Once switched, the variable no longer decides, so there is nothing to warn about.
        env_override: report
            .env_override
            .filter(|_| !choice.ignoring_env())
            .map(|p| p.display().to_string()),
        session_closed,
    })
}

/// **Archive**: ends an open chat session in the project, then moves the project into
/// `Projects/Archive/` (ADR-0015).
pub fn archive(engine: &Engine, open: &OpenSession, name: &str) -> Outcome<ProjectMoved> {
    let session_closed = close_session_in(open, name)?;
    let path = engine.archive_project(name)?;
    Ok(ProjectMoved {
        name: name.to_string(),
        path: path.display().to_string(),
        session_closed,
    })
}

/// **Restore**: moves an archived project back into `Projects/`. An archived project has no
/// chat session, so there is none to end.
pub fn restore(engine: &Engine, name: &str) -> Outcome<ProjectMoved> {
    let path = engine.restore_project(name)?;
    Ok(ProjectMoved {
        name: name.to_string(),
        path: path.display().to_string(),
        session_closed: None,
    })
}

/// **Delete**: ends an open chat session in the project, then moves it to the Trash. With
/// `erase`, sent only after the person confirmed erasing, a project the Trash cannot take is
/// erased instead (ADR-0015).
pub fn delete(
    engine: &Engine,
    open: &OpenSession,
    name: &str,
    archived: bool,
    erase: bool,
) -> Outcome<ProjectDeleted> {
    let session_closed = if archived {
        None
    } else {
        close_session_in(open, name)?
    };
    let (path, how) = engine.delete_project(name, archived, erase)?;
    Ok(ProjectDeleted {
        name: name.to_string(),
        path: path.display().to_string(),
        erased: how == Deleted::Erased,
        session_closed,
    })
}

/// **Rename**: ends an open chat session in the project, then renames it, folder and display
/// name (ADR-0015 point 7).
pub fn rename(
    engine: &Engine,
    open: &OpenSession,
    name: &str,
    new_name: &str,
) -> Outcome<ProjectRenamed> {
    let session_closed = close_session_in(open, name)?;
    let (project, metadata) = engine.rename_project(name, new_name)?;
    Ok(ProjectRenamed {
        project: ProjectDetails::new(&project, &metadata),
        session_closed,
    })
}

/// **Clone**: copies a project into `<folder>-clone` (ADR-0015) and returns the clone.
pub fn clone(engine: &Engine, name: &str) -> Outcome<ProjectDetails> {
    let (project, metadata) = engine.clone_project(name)?;
    Ok(ProjectDetails::new(&project, &metadata))
}

/// Ends the open chat session if it is in `project`; a session elsewhere is left alone.
fn close_session_in(open: &OpenSession, project: &str) -> Outcome<Option<SessionClosed>> {
    let in_project = lock(open)?
        .as_ref()
        .is_some_and(|session| session.summary().project.name == project);
    if in_project {
        session_close(open)
    } else {
        Ok(None)
    }
}

/// **Switch now**: from here until the app restarts, use the saved folder (else `~/AI`) even
/// though `AI_WORKSPACE_HOME` is set. An open chat session belongs to the old folder, so it is
/// ended first.
pub fn switch_to_saved(choice: &RootChoice, open: &OpenSession) -> Outcome<SwitchDone> {
    let session_closed = session_close(open)?;
    choice.0.store(true, Ordering::SeqCst);
    let engine = resolve(choice)?;
    Ok(SwitchDone {
        root: engine.paths().root.display().to_string(),
        root_source: source_text(choice, engine.root_source().describe()),
        session_closed,
    })
}

/// Opens a session in `name`, closing any session already open first.
pub fn session_open(engine: &Engine, open: &OpenSession, name: &str) -> Outcome<SessionInfo> {
    let mut slot = lock(open)?;
    if let Some(previous) = slot.take() {
        open.set_project(None);
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
    let info = SessionInfo::new(&session.summary());
    open.set_project(Some(info.project.clone()));
    *slot = Some(session);
    Ok(info)
}

/// Sends one message in the open session and returns the reply.
/// Sends one message in the open session, passing each piece of the reply to `on_piece` as
/// the model writes it (ADR-0013), and returns the whole reply.
pub fn session_send(
    open: &OpenSession,
    message: &str,
    on_piece: &mut dyn FnMut(&str),
) -> Outcome<ChatReply> {
    let mut slot = lock(open)?;
    let session = slot
        .as_mut()
        .ok_or_else(|| CommandError::shell("no session is open"))?;
    let text = session.send_streaming(message, on_piece)?;
    Ok(ChatReply {
        text,
        left_out_exchanges: u32::try_from(session.left_out_exchanges()).unwrap_or(u32::MAX),
        session: SessionInfo::new(&session.summary()),
    })
}

/// The open session with its context built again now, for `message` when one is about to be
/// sent: a grant or revoke, or the message, shows in the context list at once.
pub fn session_context(open: &OpenSession, message: Option<&str>) -> Outcome<SessionInfo> {
    let mut slot = lock(open)?;
    let session = slot
        .as_mut()
        .ok_or_else(|| CommandError::shell("no session is open"))?;
    session.refresh_context(message)?;
    Ok(SessionInfo::new(&session.summary()))
}

/// Asks the open session's model to draft a note about the session. Nothing is saved.
pub fn session_draft_note(open: &OpenSession) -> Outcome<String> {
    let slot = lock(open)?;
    let session = slot
        .as_ref()
        .ok_or_else(|| CommandError::shell("no session is open"))?;
    Ok(session.draft_note()?)
}

/// Saves `text` as a session note in the project's Session folder, on the person's say-so.
pub fn save_note(engine: &Engine, project: &str, text: &str) -> Outcome<NoteSaved> {
    let path = engine.save_session_note(project, text)?;
    Ok(NoteSaved {
        project: project.to_string(),
        path: path.display().to_string(),
    })
}

/// Closes the open session, if there is one.
pub fn session_close(open: &OpenSession) -> Outcome<Option<SessionClosed>> {
    let mut slot = lock(open)?;
    let Some(session) = slot.take() else {
        return Ok(None);
    };
    open.set_project(None);
    let project = session.summary().project.name.clone();
    let revoked = session.close()?;
    Ok(Some(SessionClosed {
        project,
        revoked: u32::try_from(revoked).unwrap_or(u32::MAX),
    }))
}

fn lock(open: &OpenSession) -> Outcome<std::sync::MutexGuard<'_, Option<Session>>> {
    open.slot.lock().map_err(|_| {
        CommandError::shell(
            "the session state is unusable after an earlier failure; restart the app",
        )
    })
}

// ---- Tauri wrappers -------------------------------------------------------------------------

type Choice<'a> = tauri::State<'a, RootChoice>;

#[tauri::command]
pub async fn workspace_status(choice: Choice<'_>) -> Outcome<WorkspaceStatus> {
    let mut status = status(&resolve(&choice)?)?;
    status.root_source = source_text(&choice, status.root_source);
    Ok(status)
}

#[tauri::command]
pub async fn workspace_config(choice: Choice<'_>) -> Outcome<WorkspaceSettings> {
    let mut settings = settings(&resolve(&choice)?)?;
    settings.root_source = source_text(&choice, settings.root_source);
    Ok(settings)
}

#[tauri::command]
pub async fn workspace_validate(choice: Choice<'_>) -> Outcome<Validation> {
    validate(&resolve(&choice)?)
}

#[tauri::command]
pub async fn project_show(choice: Choice<'_>, name: String) -> Outcome<ProjectDetails> {
    project_details(&resolve(&choice)?, &name)
}

#[tauri::command]
pub async fn workspace_search(
    choice: Choice<'_>,

    query: String,
    project: Option<String>,
    limit: Option<u32>,
) -> Outcome<SearchResults> {
    search(&resolve(&choice)?, &query, project.as_deref(), limit)
}

#[tauri::command]
pub async fn workspace_init(choice: Choice<'_>) -> Outcome<InitDone> {
    init(&resolve(&choice)?)
}

#[tauri::command]
pub async fn workspace_set_root(
    choice: Choice<'_>,
    open: tauri::State<'_, OpenSession>,
    path: String,
) -> Outcome<RootSaved> {
    set_root(&choice, &open, &path)
}

#[tauri::command]
pub async fn workspace_switch_to_saved(
    choice: Choice<'_>,
    open: tauri::State<'_, OpenSession>,
) -> Outcome<SwitchDone> {
    switch_to_saved(&choice, &open)
}

#[tauri::command]
pub async fn project_create(
    choice: Choice<'_>,
    name: String,
    description: Option<String>,
) -> Outcome<ProjectDetails> {
    create_project(&resolve(&choice)?, &name, description.as_deref())
}

#[tauri::command]
pub async fn project_archive(
    choice: Choice<'_>,
    open: tauri::State<'_, OpenSession>,
    name: String,
) -> Outcome<ProjectMoved> {
    archive(&resolve(&choice)?, &open, &name)
}

#[tauri::command]
pub async fn project_delete(
    choice: Choice<'_>,
    open: tauri::State<'_, OpenSession>,
    name: String,
    archived: bool,
    erase: bool,
) -> Outcome<ProjectDeleted> {
    delete(&resolve(&choice)?, &open, &name, archived, erase)
}

#[tauri::command]
pub async fn project_rename(
    choice: Choice<'_>,
    open: tauri::State<'_, OpenSession>,
    name: String,
    new_name: String,
) -> Outcome<ProjectRenamed> {
    rename(&resolve(&choice)?, &open, &name, &new_name)
}

#[tauri::command]
pub async fn project_clone(choice: Choice<'_>, name: String) -> Outcome<ProjectDetails> {
    clone(&resolve(&choice)?, &name)
}

#[tauri::command]
pub async fn project_restore(choice: Choice<'_>, name: String) -> Outcome<ProjectMoved> {
    restore(&resolve(&choice)?, &name)
}

#[tauri::command]
pub async fn project_set_model(
    choice: Choice<'_>,
    name: String,
    model: Option<String>,
) -> Outcome<ProjectDetails> {
    set_project_model(&resolve(&choice)?, &name, model.as_deref())
}

#[tauri::command]
pub async fn models_installed() -> Outcome<Vec<String>> {
    Ok(installed_models())
}

#[tauri::command]
pub async fn permission_list(
    choice: Choice<'_>,

    project: String,
    filter: Option<String>,
    page: Option<u32>,
    page_size: Option<u32>,
) -> Outcome<PermissionPage> {
    permissions(
        &resolve(&choice)?,
        &project,
        filter.as_deref(),
        page,
        page_size,
    )
}

#[tauri::command]
pub async fn permission_grant(
    choice: Choice<'_>,
    project: String,
    path: String,
) -> Outcome<GrantDone> {
    grant(&resolve(&choice)?, &project, &path)
}

#[tauri::command]
pub async fn permission_revoke(
    choice: Choice<'_>,
    project: String,
    path: String,
) -> Outcome<RevokeDone> {
    revoke(&resolve(&choice)?, &project, &path)
}

#[tauri::command]
pub async fn chat_open(
    choice: Choice<'_>,
    open: tauri::State<'_, OpenSession>,
    project: String,
) -> Outcome<SessionInfo> {
    session_open(&resolve(&choice)?, &open, &project)
}

#[tauri::command]
pub async fn chat_send(
    open: tauri::State<'_, OpenSession>,
    message: String,
    on_piece: tauri::ipc::Channel<String>,
) -> Outcome<ChatReply> {
    // A piece the window can no longer receive (it was closed) is simply dropped; the whole
    // reply is still returned and kept in the history.
    session_send(&open, &message, &mut |piece| {
        let _ = on_piece.send(piece.to_string());
    })
}

#[tauri::command]
pub async fn chat_context(
    open: tauri::State<'_, OpenSession>,
    message: Option<String>,
) -> Outcome<SessionInfo> {
    session_context(&open, message.as_deref())
}

#[tauri::command]
pub async fn chat_draft_note(open: tauri::State<'_, OpenSession>) -> Outcome<String> {
    session_draft_note(&open)
}

#[tauri::command]
pub async fn note_save(choice: Choice<'_>, project: String, text: String) -> Outcome<NoteSaved> {
    save_note(&resolve(&choice)?, &project, &text)
}

/// Closes the window after the save-on-close question was answered (the note saved, or
/// declined). The chat session is closed on exit, as always.
#[tauri::command]
pub async fn app_quit(
    window: tauri::Window,
    allowed: tauri::State<'_, QuitAllowed>,
) -> Outcome<()> {
    allowed.0.store(true, Ordering::SeqCst);
    window
        .close()
        .map_err(|error| CommandError::shell(format!("could not close the window: {error}")))
}

#[tauri::command]
pub async fn chat_close(open: tauri::State<'_, OpenSession>) -> Outcome<Option<SessionClosed>> {
    session_close(&open)
}

#[cfg(test)]
mod close_check_tests {
    use super::*;
    use std::sync::mpsc;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn closing_during_a_reply_asks_at_once_instead_of_waiting() {
        let open = std::sync::Arc::new(OpenSession::default());
        open.set_project(Some("project-a".to_string()));
        let allowed = QuitAllowed::default();

        // A reply being written holds the session for its whole length.
        let (held, release) = (mpsc::channel(), mpsc::channel::<()>());
        let busy = std::sync::Arc::clone(&open);
        let writer = thread::spawn(move || {
            let _slot = busy.slot.lock().expect("lock");
            held.0.send(()).expect("held");
            let _ = release.1.recv_timeout(Duration::from_secs(10));
        });
        held.1.recv().expect("the reply holds the session");

        let (answer, answered) = mpsc::channel();
        let checking = std::sync::Arc::clone(&open);
        thread::spawn(move || {
            let _ = answer.send(needs_save_prompt(&checking, &allowed));
        });
        let asked = answered.recv_timeout(Duration::from_secs(2));
        release.0.send(()).expect("release");
        writer.join().expect("writer");
        assert_eq!(
            asked.expect("the close check must not wait for the reply"),
            Some("project-a".to_string())
        );
    }

    #[test]
    fn a_closed_session_leaves_nothing_to_ask_about() {
        let open = OpenSession::default();
        open.set_project(Some("project-a".to_string()));
        assert!(
            session_close(&open)
                .expect("nothing open is fine")
                .is_none()
        );
        assert_eq!(
            needs_save_prompt(&open, &QuitAllowed::default()),
            None,
            "free and empty: close at once"
        );
    }
}
