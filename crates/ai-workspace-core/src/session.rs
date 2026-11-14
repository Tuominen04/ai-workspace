//! `Session`: one interactive session in a project. It owns the project's
//! permission session, the local runtime and the chat history, and returns
//! replies as data; a front end does the prompting and rendering. See
//! ADR-0008.

use crate::config::load_workspace_config;
use crate::context::{ContextSnapshot, build_context_snapshot};
use crate::engine::Engine;
use crate::error::{AppError, Result};
use crate::paths::WorkspacePaths;
use crate::runtime::{ChatMessage, OLLAMA_BASE_URL, RuntimeManager, RuntimeStatus};
use crate::workspace::{
    Project, ProjectMetadata, initialize_permission_session, inspect_project, resolve_model,
    terminate_permission_session,
};

/// An open session. Close it with [`Session::close`]; dropping it without
/// closing still ends its permission session (temporary grants are revoked)
/// and stops a runtime it started, so a front end that fails half-way does not
/// leave either behind.
pub struct Session {
    paths: WorkspacePaths,
    project: Project,
    model: String,
    initial_context: ContextSnapshot,
    runtime_status: RuntimeStatus,
    history: Vec<ChatMessage>,
    closed: bool,
    // Declared last so it is dropped after the permission session is closed
    // in `Drop`, matching the order of `close`.
    runtime: RuntimeManager,
}

/// What a front end shows when a session starts.
#[derive(Debug, Clone, Copy)]
pub struct SessionSummary<'a> {
    pub project: &'a Project,
    /// The context built for the start of the session, before any message.
    pub context: &'a ContextSnapshot,
    pub runtime: &'a RuntimeStatus,
}

impl Session {
    /// Opens a session in `project`, which should come from
    /// [`Engine::projects`]: starts its permission session (default grants),
    /// builds the first context from the project's description, and starts or
    /// finds the local runtime.
    pub fn open(engine: &Engine, project: &Project) -> Result<Self> {
        let paths = engine.paths().clone();
        let _ = initialize_permission_session(&paths, project)?;

        let config = load_workspace_config(&paths)?;
        let (_, metadata) = inspect_project(&paths, &config, &project.name)?;
        let initial_query = initial_context_query(&metadata);
        let initial_context = build_context_snapshot(&paths, Some(project), Some(&initial_query))?;
        let mut runtime = RuntimeManager::new();
        let runtime_status = runtime.ensure_ollama()?;

        let history = vec![ChatMessage::system(initial_context.as_system_prompt())];
        Ok(Self {
            model: resolve_model(&config, &metadata),
            paths,
            project: project.clone(),
            initial_context,
            runtime_status,
            history,
            closed: false,
            runtime,
        })
    }

    pub fn summary(&self) -> SessionSummary<'_> {
        SessionSummary {
            project: &self.project,
            context: &self.initial_context,
            runtime: &self.runtime_status,
        }
    }

    /// `true` when a runtime is available, so [`Session::send`] can be used.
    pub fn can_chat(&self) -> bool {
        self.runtime_status.running
    }

    /// Sends one user message and returns the model's reply. The context is
    /// rebuilt for this message first (if that fails, the previous context is
    /// kept). On an error the message is dropped from the history, so the
    /// user can simply try again.
    pub fn send(&mut self, message: &str) -> Result<String> {
        if !self.can_chat() {
            return Err(AppError::RuntimeRequestFailed(
                "no local runtime is running".to_string(),
            ));
        }

        if let Ok(context) = build_context_snapshot(&self.paths, Some(&self.project), Some(message))
        {
            self.history[0] = ChatMessage::system(context.as_system_prompt());
        }

        self.history.push(ChatMessage::user(message.to_string()));

        match self
            .runtime
            .send_chat_message(OLLAMA_BASE_URL, &self.model, &self.history)
        {
            Ok(reply) => {
                let content = reply.content.clone();
                self.history.push(reply);
                Ok(content)
            }
            Err(error) => {
                self.history.pop();
                Err(error)
            }
        }
    }

    /// Ends the session: revokes its temporary grants and returns how many
    /// there were, then stops a runtime the session started.
    pub fn close(mut self) -> Result<usize> {
        self.closed = true;
        terminate_permission_session(&self.paths, &self.project)
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        if !self.closed {
            let _ = terminate_permission_session(&self.paths, &self.project);
        }
    }
}

/// The implicit query used to build the very first context, before the user
/// has asked anything. Falls back to the project's display name only in the
/// unlikely case its description is empty. See
/// `specs/006-filesystem-search/research.md` Decision 4 and
/// `contracts/context-builder-integration.md` (the fix for finding I1).
fn initial_context_query(metadata: &ProjectMetadata) -> String {
    let description = metadata.description.trim();
    if description.is_empty() {
        metadata.display_name.clone()
    } else {
        description.to_string()
    }
}
