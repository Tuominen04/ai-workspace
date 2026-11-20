//! `Session`: one interactive session in a project. It owns the project's
//! permission session, the local runtime and the chat history, and returns
//! replies as data; a front end does the prompting and rendering. See
//! ADR-0008.

use crate::config::{Runtime, load_workspace_config};
use crate::context::{ContextBudget, ContextSnapshot, build_context_snapshot_within};
use crate::engine::Engine;
use crate::error::{AppError, Result};
use crate::paths::WorkspacePaths;
use crate::runtime::{
    ChatMessage, OLLAMA_BASE_URL, RuntimeManager, RuntimeStatus, podman_model_pull_command,
};
use crate::workspace::{
    Project, ProjectMetadata, initialize_permission_session, inspect_project, resolve_model,
    terminate_permission_session,
};
use std::fmt;

/// An open session. Close it with [`Session::close`]; dropping it without
/// closing still ends its permission session (temporary grants are revoked)
/// and stops a runtime it started, so a front end that fails half-way does not
/// leave either behind.
pub struct Session {
    paths: WorkspacePaths,
    project: Project,
    model: String,
    agent: AgentInstructions,
    budget: ContextBudget,
    initial_context: ContextSnapshot,
    runtime_status: RuntimeStatus,
    /// How Ollama is started for this workspace, so a missing model's download command fits.
    runtime_kind: Runtime,
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
    /// The model the session talks to.
    pub model: &'a str,
    /// Whether the model's own instructions are part of the system message.
    pub agent: &'a AgentInstructions,
}

/// The model's own `SYSTEM` instructions, as far as the session could tell (ADR-0012).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentInstructions {
    /// The model has instructions; they come first in every system message.
    Included(String),
    /// The model has no `SYSTEM` instructions; the context is the whole system message.
    NoneInModel,
    /// Ollama could not be asked (not running, or the request failed): the session goes on
    /// with the context alone. The reason, in words.
    Unknown(String),
}

impl AgentInstructions {
    pub fn text(&self) -> Option<&str> {
        match self {
            Self::Included(text) => Some(text),
            _ => None,
        }
    }
}

impl fmt::Display for AgentInstructions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Included(_) => f.write_str("the model's own instructions are kept"),
            Self::NoneInModel => f.write_str("the model has no instructions of its own"),
            Self::Unknown(reason) => {
                write!(
                    f,
                    "the model's own instructions could not be read ({reason})"
                )
            }
        }
    }
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
        let mut runtime = RuntimeManager::new();
        let runtime_status = runtime.ensure_runtime(config.runtime, &config.podman)?;

        let model = resolve_model(&config, &metadata);
        let (agent, window) = if runtime_status.running {
            match runtime.show_model(OLLAMA_BASE_URL, &model) {
                Ok(info) => (
                    match info.system {
                        Some(text) => AgentInstructions::Included(text),
                        None => AgentInstructions::NoneInModel,
                    },
                    info.num_ctx,
                ),
                Err(error) => (
                    AgentInstructions::Unknown(for_runtime(error, config.runtime).to_string()),
                    None,
                ),
            }
        } else {
            (
                AgentInstructions::Unknown("no local runtime is running".to_string()),
                None,
            )
        };
        // The model's window decides how much workspace text fits (ADR-0012); unknown keeps
        // the fixed default.
        let budget = match window {
            Some(tokens) => ContextBudget::for_window(tokens, agent.text().map_or(0, str::len)),
            None => ContextBudget::default(),
        };
        let initial_context =
            build_context_snapshot_within(&paths, Some(project), Some(&initial_query), budget)?;

        let history = vec![ChatMessage::system(
            initial_context.as_system_prompt_for(agent.text()),
        )];
        Ok(Self {
            model,
            agent,
            budget,
            paths,
            project: project.clone(),
            initial_context,
            runtime_status,
            runtime_kind: config.runtime,
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
            model: &self.model,
            agent: &self.agent,
        }
    }

    /// How many messages the person has sent and had answered.
    pub fn exchanges(&self) -> usize {
        self.history
            .iter()
            .filter(|m| m.role == "assistant")
            .count()
    }

    /// Asks the model to draft a short note about this session for later sessions (ADR-0012).
    /// The request is not added to the history. The note is only text: saving it is a
    /// separate, explicit step ([`Engine::save_session_note`]).
    pub fn draft_note(&self) -> Result<String> {
        if !self.can_chat() {
            return Err(AppError::RuntimeRequestFailed(
                "no local runtime is running".to_string(),
            ));
        }
        if self.exchanges() == 0 {
            return Err(AppError::Message(
                "there is nothing to note yet: no message has been answered".to_string(),
            ));
        }
        let mut messages = self.history.clone();
        messages.push(ChatMessage::user(NOTE_REQUEST.to_string()));
        let reply = self
            .runtime
            .send_chat_message(OLLAMA_BASE_URL, &self.model, &messages)
            .map_err(|error| for_runtime(error, self.runtime_kind))?;
        Ok(reply.content.trim().to_string())
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
        self.send_streaming(message, &mut |_| {})
    }

    /// Like [`Session::send`], calling `on_piece` with each piece of the reply as the model
    /// writes it (ADR-0013). On an error the pieces already passed on are not part of any reply:
    /// the message is dropped from the history, as with `send`.
    pub fn send_streaming(
        &mut self,
        message: &str,
        on_piece: &mut dyn FnMut(&str),
    ) -> Result<String> {
        if !self.can_chat() {
            return Err(AppError::RuntimeRequestFailed(
                "no local runtime is running".to_string(),
            ));
        }

        if let Ok(context) = build_context_snapshot_within(
            &self.paths,
            Some(&self.project),
            Some(message),
            self.budget,
        ) {
            self.history[0] = ChatMessage::system(context.as_system_prompt_for(self.agent.text()));
        }

        self.history.push(ChatMessage::user(message.to_string()));

        match self.runtime.stream_chat_message(
            OLLAMA_BASE_URL,
            &self.model,
            &self.history,
            on_piece,
        ) {
            Ok(reply) => {
                let content = reply.content.clone();
                self.history.push(reply);
                Ok(content)
            }
            Err(error) => {
                self.history.pop();
                Err(for_runtime(error, self.runtime_kind))
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

/// What the model is asked when drafting a session note.
const NOTE_REQUEST: &str = "Write a short Markdown note about this session for future sessions in this project. Start with a level-one heading naming the topic. Then list, as bullet points: what was discussed, what was decided or changed, open questions, and what to keep in mind next time. Use only what was actually said here. Write only the note.";

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

/// A missing model's download command, for the runtime in use: with Podman, the model goes into
/// the container the app runs (ADR-0016).
fn for_runtime(error: AppError, runtime: Runtime) -> AppError {
    match (error, runtime) {
        (AppError::ModelNotInstalled { model, .. }, Runtime::Podman) => {
            let pull_command = podman_model_pull_command(&model);
            AppError::ModelNotInstalled {
                model,
                pull_command,
            }
        }
        (error, _) => error,
    }
}

#[cfg(test)]
mod for_runtime_tests {
    use super::*;

    fn missing() -> AppError {
        AppError::ModelNotInstalled {
            model: "qwen3:8b".to_string(),
            pull_command: "ollama pull qwen3:8b".to_string(),
        }
    }

    #[test]
    fn the_installed_ollama_keeps_its_own_command() {
        let message = for_runtime(missing(), Runtime::Ollama).to_string();
        assert!(message.contains("`ollama pull qwen3:8b`"), "{message}");
    }

    #[test]
    fn podman_gets_the_command_for_its_container() {
        let message = for_runtime(missing(), Runtime::Podman).to_string();
        assert!(
            message.contains("`podman exec ai-workspace-ollama ollama pull qwen3:8b`"),
            "{message}"
        );
        assert!(message.starts_with("the model 'qwen3:8b' is not installed in Ollama"));
    }

    #[test]
    fn other_errors_pass_through() {
        let other = for_runtime(
            AppError::RuntimeRequestFailed("timed out".to_string()),
            Runtime::Podman,
        );
        assert!(matches!(other, AppError::RuntimeRequestFailed(_)));
    }
}
