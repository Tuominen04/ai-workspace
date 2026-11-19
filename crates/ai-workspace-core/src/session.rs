//! `Session`: one interactive session in a project. It owns the project's
//! permission session, the local runtime and the chat history, and returns
//! replies as data; a front end does the prompting and rendering. See
//! ADR-0008.

use crate::config::load_workspace_config;
use crate::context::{ContextBudget, ContextSnapshot, build_context_snapshot_within};
use crate::engine::Engine;
use crate::error::{AppError, Result};
use crate::paths::WorkspacePaths;
use crate::runtime::{ChatMessage, OLLAMA_BASE_URL, RuntimeManager, RuntimeStatus};
use crate::workspace::{
    Project, ProjectMetadata, initialize_permission_session, inspect_project, resolve_model,
    terminate_permission_session,
};
use std::fmt;

/// Bytes of conversation sent with each message when the model's window is unknown.
const DEFAULT_HISTORY_BYTES: usize = 10_000;

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
    history: Vec<ChatMessage>,
    /// Earlier exchanges left out of the last request because they no longer fit the window.
    left_out: usize,
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
        let runtime_status = runtime.ensure_ollama()?;

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
                Err(error) => (AgentInstructions::Unknown(error.to_string()), None),
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
            history,
            left_out: 0,
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
        let mut history = self.history.clone();
        history.push(ChatMessage::user(NOTE_REQUEST.to_string()));
        let (messages, _) = self.messages_to_send(&history);
        let reply = self
            .runtime
            .send_chat_message(OLLAMA_BASE_URL, &self.model, &messages)?;
        Ok(reply.content.trim().to_string())
    }

    /// How many earlier exchanges the last message left out because the conversation no
    /// longer fits the model's window; 0 while everything fits.
    pub fn left_out_exchanges(&self) -> usize {
        self.left_out
    }

    /// Bytes of conversation (the person's messages and the replies) sent with each message:
    /// one byte per window token, about a quarter of the window, next to the workspace text
    /// (`ContextBudget`, 1.5 bytes per token), so even dense text leaves room for a long reply.
    fn history_budget(&self) -> usize {
        match self.budget.window_tokens {
            Some(tokens) => tokens as usize,
            None => DEFAULT_HISTORY_BYTES,
        }
    }

    fn messages_to_send(&self, history: &[ChatMessage]) -> (Vec<ChatMessage>, usize) {
        recent_messages(history, self.history_budget())
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
        let (messages, left_out) = self.messages_to_send(&self.history);

        match self
            .runtime
            .stream_chat_message(OLLAMA_BASE_URL, &self.model, &messages, on_piece)
        {
            Ok(reply) => {
                let content = reply.content.clone();
                self.history.push(reply);
                self.left_out = left_out;
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

/// The messages for the next request: the system message, then the most recent exchanges whose
/// text fits `budget` bytes, always including the last message. Whole exchanges are dropped
/// from the start, so the model never sees a reply without its question. Returns them with how
/// many exchanges were left out.
fn recent_messages(history: &[ChatMessage], budget: usize) -> (Vec<ChatMessage>, usize) {
    let Some((system, rest)) = history.split_first() else {
        return (Vec::new(), 0);
    };
    let mut start = rest.len();
    let mut used = 0;
    while start > 0 {
        let size = rest[start - 1].content.len();
        let is_last = start == rest.len();
        if !is_last && used + size > budget {
            break;
        }
        used += size;
        start -= 1;
    }
    // Begin on a question, not on a reply whose question no longer fits.
    while start < rest.len().saturating_sub(1) && rest[start].role != "user" {
        start += 1;
    }
    let left_out = rest[..start]
        .iter()
        .filter(|m| m.role == "assistant")
        .count();
    let mut messages = Vec::with_capacity(rest.len() - start + 1);
    messages.push(system.clone());
    messages.extend_from_slice(&rest[start..]);
    (messages, left_out)
}

#[cfg(test)]
mod recent_messages_tests {
    use super::*;

    fn chat(exchanges: usize, size: usize) -> Vec<ChatMessage> {
        let mut history = vec![ChatMessage::system("workspace notes".to_string())];
        for n in 0..exchanges {
            history.push(ChatMessage::user(format!("q{n}:{}", "x".repeat(size))));
            history.push(ChatMessage {
                role: "assistant".to_string(),
                content: format!("a{n}:{}", "y".repeat(size)),
            });
        }
        history
    }

    #[test]
    fn everything_is_sent_while_it_fits() {
        let mut history = chat(3, 10);
        history.push(ChatMessage::user("next".to_string()));
        let (messages, left_out) = recent_messages(&history, 10_000);
        assert_eq!(messages.len(), history.len());
        assert_eq!(left_out, 0);
    }

    #[test]
    fn the_oldest_exchanges_are_left_out_whole() {
        // Each exchange is about 2 x 1000 bytes; a 4500-byte budget keeps two of them.
        let mut history = chat(5, 1000);
        history.push(ChatMessage::user("next".to_string()));
        let (messages, left_out) = recent_messages(&history, 4500);
        assert_eq!(
            messages[0].role, "system",
            "the workspace notes always come first"
        );
        assert_eq!(messages[1].role, "user", "it starts on a question");
        assert!(
            messages[1].content.starts_with("q3:"),
            "{}",
            messages[1].content
        );
        assert_eq!(messages.last().map(|m| m.content.as_str()), Some("next"));
        assert_eq!(left_out, 3);
    }

    #[test]
    fn the_new_message_is_sent_even_when_it_alone_is_too_long() {
        let mut history = chat(2, 10);
        history.push(ChatMessage::user("z".repeat(50_000)));
        let (messages, left_out) = recent_messages(&history, 1000);
        assert_eq!(messages.len(), 2, "system and the new message");
        assert_eq!(left_out, 2);
    }
}
