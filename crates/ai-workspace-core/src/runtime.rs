use crate::error::{AppError, Result};
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader};
use std::net::{SocketAddr, TcpStream};
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

const OLLAMA_PORT: u16 = 11434;
pub const OLLAMA_BASE_URL: &str = "http://127.0.0.1:11434";
/// How long to wait for Ollama to accept a chat request.
pub const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
/// How long to wait for each next piece of a streamed reply. Generous, because a large model
/// may first have to be loaded into memory; there is no limit on the whole answer (ADR-0013).
pub const PIECE_TIMEOUT: Duration = Duration::from_secs(300);
const SHOW_REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

impl ChatMessage {
    pub fn system(content: String) -> Self {
        Self {
            role: "system".to_string(),
            content,
        }
    }

    pub fn user(content: String) -> Self {
        Self {
            role: "user".to_string(),
            content,
        }
    }
}

#[derive(Serialize)]
struct ChatRequest<'a> {
    model: &'a str,
    messages: &'a [ChatMessage],
    stream: bool,
}

/// One line of a streamed chat reply.
#[derive(Deserialize)]
struct ChatChunk {
    #[serde(default)]
    message: Option<ChatMessage>,
    #[serde(default)]
    done: bool,
    #[serde(default)]
    error: Option<String>,
}

#[derive(Serialize)]
struct ShowRequest<'a> {
    model: &'a str,
}

#[derive(Deserialize)]
struct ShowResponse {
    #[serde(default)]
    system: Option<String>,
    #[serde(default)]
    parameters: Option<String>,
    #[serde(default)]
    modelfile: Option<String>,
}

#[derive(Deserialize)]
struct TagsResponse {
    #[serde(default)]
    models: Vec<TagEntry>,
}

#[derive(Deserialize)]
struct TagEntry {
    name: String,
}

/// What Ollama says about a model (`POST /api/show`), as far as a session needs it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ModelInfo {
    /// The model's own `SYSTEM` instructions from its Modelfile, if it has any.
    pub system: Option<String>,
    /// The model's context window in tokens (`num_ctx`), if its Modelfile sets one.
    pub num_ctx: Option<u32>,
}

impl ModelInfo {
    fn from_response(response: ShowResponse) -> Self {
        let system = response
            .system
            .or_else(|| {
                response
                    .modelfile
                    .as_deref()
                    .and_then(system_from_modelfile)
            })
            .map(|text| text.trim().to_string())
            .filter(|text| !text.is_empty());
        let num_ctx = response.parameters.as_deref().and_then(|parameters| {
            parameters.lines().find_map(|line| {
                let mut words = line.split_whitespace();
                match (words.next(), words.next()) {
                    (Some("num_ctx"), Some(value)) => value.parse().ok(),
                    _ => None,
                }
            })
        });
        Self { system, num_ctx }
    }
}

/// The `SYSTEM` text of a Modelfile, for Ollama versions that report it only inside the
/// Modelfile. Handles `SYSTEM """…"""` (possibly spanning lines) and a one-line `SYSTEM text`.
fn system_from_modelfile(modelfile: &str) -> Option<String> {
    let start = modelfile
        .lines()
        .scan(0usize, |offset, line| {
            let here = *offset;
            *offset += line.len() + 1;
            Some((here, line))
        })
        .find(|(_, line)| {
            line.trim_start()
                .to_ascii_uppercase()
                .starts_with("SYSTEM ")
        })
        .map(|(offset, line)| offset + (line.len() - line.trim_start().len()) + "SYSTEM ".len())?;
    let rest = modelfile.get(start..)?.trim_start();
    if let Some(quoted) = rest.strip_prefix("\"\"\"") {
        return quoted.find("\"\"\"").map(|end| quoted[..end].to_string());
    }
    Some(
        rest.lines()
            .next()
            .unwrap_or_default()
            .trim_matches('"')
            .to_string(),
    )
}

#[derive(Debug, Clone)]
pub struct RuntimeStatus {
    pub running: bool,
    pub started_here: bool,
    pub detail: String,
}

pub struct RuntimeManager {
    ollama: Option<Child>,
}

impl Default for RuntimeManager {
    fn default() -> Self {
        Self::new()
    }
}

impl RuntimeManager {
    pub fn new() -> Self {
        Self { ollama: None }
    }

    pub fn ensure_ollama(&mut self) -> Result<RuntimeStatus> {
        if is_ollama_running() {
            return Ok(RuntimeStatus {
                running: true,
                started_here: false,
                detail: "Ollama is already running on 127.0.0.1:11434".to_string(),
            });
        }

        match Command::new("ollama")
            .arg("serve")
            .stdin(Stdio::null())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
        {
            Ok(child) => {
                self.ollama = Some(child);
                wait_for_ollama_ready(Duration::from_secs(10));
                Ok(RuntimeStatus {
                    running: true,
                    started_here: true,
                    detail: "Started Ollama for this session".to_string(),
                })
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(RuntimeStatus {
                running: false,
                started_here: false,
                detail: "Ollama is not installed; session continues without a local runtime"
                    .to_string(),
            }),
            Err(error) => Err(error.into()),
        }
    }

    /// Sends a chat and returns the whole reply. Streams underneath (ADR-0013), so it has the
    /// same timeouts as [`Self::stream_chat_message`]; the pieces are simply not shown.
    pub fn send_chat_message(
        &self,
        base_url: &str,
        model: &str,
        messages: &[ChatMessage],
    ) -> Result<ChatMessage> {
        self.stream_chat_message(base_url, model, messages, &mut |_| {})
    }

    /// Sends a chat with `stream: true` and calls `on_piece` with each piece of the reply as
    /// Ollama writes it, then returns the whole reply. Waits up to [`CONNECT_TIMEOUT`] to
    /// connect and up to [`PIECE_TIMEOUT`] for each next piece, with no limit on the whole
    /// answer. A stream that carries an `error`, or ends without `done`, fails the request.
    pub fn stream_chat_message(
        &self,
        base_url: &str,
        model: &str,
        messages: &[ChatMessage],
        on_piece: &mut dyn FnMut(&str),
    ) -> Result<ChatMessage> {
        let failed = |reason: String| AppError::RuntimeRequestFailed(reason);
        let request_body = ChatRequest {
            model,
            messages,
            stream: true,
        };
        let body_bytes = serde_json::to_vec(&request_body).map_err(|e| failed(e.to_string()))?;

        let agent = ureq::AgentBuilder::new()
            .timeout_connect(CONNECT_TIMEOUT)
            .timeout_read(PIECE_TIMEOUT)
            .build();
        let response = agent
            .post(&format!("{base_url}/api/chat"))
            .set("Content-Type", "application/json")
            .send_bytes(&body_bytes)
            .map_err(|e| failed(e.to_string()))?;

        let mut content = String::new();
        let mut role = "assistant".to_string();
        for line in BufReader::new(response.into_reader()).lines() {
            let line = line.map_err(|e| failed(e.to_string()))?;
            if line.trim().is_empty() {
                continue;
            }
            let chunk: ChatChunk =
                serde_json::from_str(&line).map_err(|e| failed(e.to_string()))?;
            if let Some(error) = chunk.error {
                return Err(failed(error));
            }
            if let Some(message) = chunk.message {
                role = message.role;
                if !message.content.is_empty() {
                    on_piece(&message.content);
                    content.push_str(&message.content);
                }
            }
            if chunk.done {
                return Ok(ChatMessage { role, content });
            }
        }
        Err(failed(
            "the runtime stopped before the reply was complete".to_string(),
        ))
    }
}

impl RuntimeManager {
    /// The models Ollama has installed (`GET /api/tags`), sorted by name.
    pub fn installed_models(&self, base_url: &str) -> Result<Vec<String>> {
        let text = ureq::get(&format!("{base_url}/api/tags"))
            .timeout(SHOW_REQUEST_TIMEOUT)
            .call()
            .map_err(|error| AppError::RuntimeRequestFailed(error.to_string()))?
            .into_string()
            .map_err(|error| AppError::RuntimeRequestFailed(error.to_string()))?;
        let parsed: TagsResponse = serde_json::from_str(&text)
            .map_err(|error| AppError::RuntimeRequestFailed(error.to_string()))?;
        let mut names: Vec<String> = parsed.models.into_iter().map(|model| model.name).collect();
        names.sort();
        Ok(names)
    }

    /// Asks Ollama about `model`: its own instructions and its context window.
    pub fn show_model(&self, base_url: &str, model: &str) -> Result<ModelInfo> {
        let body = serde_json::to_vec(&ShowRequest { model })
            .map_err(|error| AppError::RuntimeRequestFailed(error.to_string()))?;
        let text = ureq::post(&format!("{base_url}/api/show"))
            .set("Content-Type", "application/json")
            .timeout(SHOW_REQUEST_TIMEOUT)
            .send_bytes(&body)
            .map_err(|error| AppError::RuntimeRequestFailed(error.to_string()))?
            .into_string()
            .map_err(|error| AppError::RuntimeRequestFailed(error.to_string()))?;
        let parsed: ShowResponse = serde_json::from_str(&text)
            .map_err(|error| AppError::RuntimeRequestFailed(error.to_string()))?;
        Ok(ModelInfo::from_response(parsed))
    }
}

impl Drop for RuntimeManager {
    fn drop(&mut self) {
        if let Some(mut child) = self.ollama.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

fn ollama_addr() -> SocketAddr {
    SocketAddr::from(([127, 0, 0, 1], OLLAMA_PORT))
}

fn is_ollama_running() -> bool {
    TcpStream::connect_timeout(&ollama_addr(), Duration::from_millis(200)).is_ok()
}

fn wait_for_ollama_ready(timeout: Duration) {
    let started = Instant::now();
    while started.elapsed() < timeout {
        if is_ollama_running() {
            return;
        }
        thread::sleep(Duration::from_millis(200));
    }
}
