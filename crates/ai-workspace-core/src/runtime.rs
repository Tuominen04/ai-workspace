use crate::config::{Gpu, PodmanSettings, Runtime};
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
    /// Why the reply ended, on the last piece: "stop", or "length" when the window was full.
    #[serde(default)]
    done_reason: Option<String>,
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
    /// Set when this manager started the Podman container, so it stops it again.
    container_started: bool,
}

/// The name of the container the app starts, and of the volume that keeps its models
/// (ADR-0016). The app only ever stops or removes a container with this name.
pub const PODMAN_NAME: &str = "ai-workspace-ollama";
/// How long a freshly started container gets to answer; the first start can be slow.
const CONTAINER_READY_TIMEOUT: Duration = Duration::from_secs(30);
/// How long an `ollama serve` the app started, or one already holding the port, may take to
/// answer.
const OLLAMA_READY_TIMEOUT: Duration = Duration::from_secs(10);

/// The arguments of `podman run` for the Ollama container: detached, removed when stopped,
/// never pulling, published on 127.0.0.1 only, models in a named volume, plus the GPU's
/// devices (ADR-0016).
pub fn podman_run_args(settings: &PodmanSettings) -> Vec<String> {
    let mut args: Vec<String> = [
        "run",
        "--detach",
        "--rm",
        "--pull=never",
        "--name",
        PODMAN_NAME,
        "--publish",
        "127.0.0.1:11434:11434",
        "--volume",
        "ai-workspace-ollama:/root/.ollama",
    ]
    .iter()
    .map(|arg| arg.to_string())
    .collect();
    match settings.gpu {
        Gpu::None => {}
        Gpu::Nvidia => args.extend(["--device".into(), "nvidia.com/gpu=all".into()]),
        Gpu::Amd => args.extend([
            "--device".into(),
            "/dev/kfd".into(),
            "--device".into(),
            "/dev/dri".into(),
        ]),
    }
    args.push(settings.image().to_string());
    args
}

/// The command a person runs once to download the image (the app never does).
pub fn podman_pull_command(settings: &PodmanSettings) -> String {
    format!("podman pull {}", settings.image())
}

impl Default for RuntimeManager {
    fn default() -> Self {
        Self::new()
    }
}

impl RuntimeManager {
    pub fn new() -> Self {
        Self {
            ollama: None,
            container_started: false,
        }
    }

    /// Makes sure Ollama answers on 127.0.0.1:11434: uses whatever already does, else starts
    /// it the way `runtime` says (ADR-0016).
    pub fn ensure_runtime(
        &mut self,
        runtime: Runtime,
        podman: &PodmanSettings,
    ) -> Result<RuntimeStatus> {
        match runtime {
            Runtime::Ollama => self.ensure_ollama(),
            Runtime::Podman => self.ensure_podman(podman),
        }
    }

    /// Starts the Ollama container unless something already answers. Never pulls: a missing
    /// image is reported with the command that downloads it.
    pub fn ensure_podman(&mut self, settings: &PodmanSettings) -> Result<RuntimeStatus> {
        // Something holds the port: use it once it answers (a container may still be starting).
        if is_ollama_running() {
            let mut status = already_running(Runtime::Podman);
            status.running = wait_for_ollama_ready(CONTAINER_READY_TIMEOUT);
            return Ok(status);
        }
        let not_running = |detail: String| RuntimeStatus {
            running: false,
            started_here: false,
            detail,
        };

        let image = settings.image();
        match podman(&["image", "exists", image]) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(not_running(
                    "Podman is not installed; session continues without a local runtime"
                        .to_string(),
                ));
            }
            Err(error) => return Err(error.into()),
            Ok(false) => {
                return Ok(not_running(format!(
                    "The Ollama image {image} is not downloaded. Download it once with: {}",
                    podman_pull_command(settings)
                )));
            }
            Ok(true) => {}
        }

        // A container of ours already running is still starting up: wait for it, leave it.
        if podman(&["container", "exists", PODMAN_NAME])? {
            if container_running()? {
                return Ok(RuntimeStatus {
                    running: wait_for_ollama_ready(CONTAINER_READY_TIMEOUT),
                    started_here: false,
                    detail: format!("Using the running {PODMAN_NAME} container"),
                });
            }
            // Left over and stopped (for example after a crash): only this one is removed.
            let _ = podman(&["rm", PODMAN_NAME]);
        }

        let output = Command::new("podman")
            .args(podman_run_args(settings))
            .stdin(Stdio::null())
            .output()?;
        if !output.status.success() {
            return Ok(not_running(format!(
                "Podman could not start Ollama: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            )));
        }
        self.container_started = true;
        Ok(RuntimeStatus {
            running: wait_for_ollama_ready(CONTAINER_READY_TIMEOUT),
            started_here: true,
            detail: format!("Started Ollama in the {PODMAN_NAME} container for this session"),
        })
    }

    pub fn ensure_ollama(&mut self) -> Result<RuntimeStatus> {
        if is_ollama_running() {
            let mut status = already_running(Runtime::Ollama);
            status.running = wait_for_ollama_ready(OLLAMA_READY_TIMEOUT);
            return Ok(status);
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
                Ok(RuntimeStatus {
                    running: wait_for_ollama_ready(OLLAMA_READY_TIMEOUT),
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
            .map_err(|e| request_failed(model, e))?;

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
                // An empty answer is a failure, not a reply: shown as one, the chat would just
                // seem to stop (it happened when a long chat filled the model's window).
                if content.trim().is_empty() {
                    return Err(failed(match chunk.done_reason.as_deref() {
                        Some("length") => {
                            "the model's context window was full, so it had no room to answer"
                                .to_string()
                        }
                        _ => "the model returned an empty reply".to_string(),
                    }));
                }
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
            .map_err(|error| request_failed(model, error))?
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
        // Only the container this manager started; `--rm` removes it once stopped. The
        // volume, and so the downloaded models, stays.
        if self.container_started {
            self.container_started = false;
            let _ = podman(&["stop", "--time", "5", PODMAN_NAME]);
        }
    }
}

/// A failed request to Ollama: a 404 means it does not have the model, said plainly with the
/// command that downloads it (a session using Podman puts in the `podman exec` form).
fn request_failed(model: &str, error: ureq::Error) -> AppError {
    match error {
        ureq::Error::Transport(transport)
            if transport.kind() == ureq::ErrorKind::ConnectionFailed =>
        {
            AppError::RuntimeNotAnswering
        }
        ureq::Error::Status(404, _) => AppError::ModelNotInstalled {
            model: model.to_string(),
            pull_command: format!("ollama pull {model}"),
        },
        other => AppError::RuntimeRequestFailed(other.to_string()),
    }
}

/// The command that downloads `model` into the Ollama the app started in Podman.
pub fn podman_model_pull_command(model: &str) -> String {
    format!("podman exec {PODMAN_NAME} ollama pull {model}")
}

/// Runs a `podman` subcommand quietly and says whether it succeeded.
fn podman(args: &[&str]) -> std::io::Result<bool> {
    Command::new("podman")
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
}

fn container_running() -> std::io::Result<bool> {
    let output = Command::new("podman")
        .args(["inspect", "--format", "{{.State.Running}}", PODMAN_NAME])
        .stdin(Stdio::null())
        .output()?;
    Ok(output.status.success() && String::from_utf8_lossy(&output.stdout).trim() == "true")
}

/// Something already answers on 127.0.0.1:11434 and is used as it is. Says which: the app's
/// own container (left running, for example started by hand), or another Ollama, which with
/// `runtime = "podman"` means the container is not what answers.
fn already_running(runtime: Runtime) -> RuntimeStatus {
    // Without Podman, or with no such container, it is not the app's container.
    let ours = container_running().unwrap_or(false);
    let detail = match (ours, runtime) {
        (true, _) => format!(
            "Using Ollama in the {PODMAN_NAME} container, which was already running (it is left running)"
        ),
        (false, Runtime::Podman) => "Using an Ollama already running on 127.0.0.1:11434, not the app's Podman container; stop it to use the container".to_string(),
        (false, Runtime::Ollama) => "Ollama is already running on 127.0.0.1:11434".to_string(),
    };
    RuntimeStatus {
        running: true,
        started_here: false,
        detail,
    }
}

fn ollama_addr() -> SocketAddr {
    SocketAddr::from(([127, 0, 0, 1], OLLAMA_PORT))
}

/// Whether something holds Ollama's port. Not proof that Ollama answers: rootless Podman opens
/// the published port as soon as the container starts, before Ollama inside listens.
fn is_ollama_running() -> bool {
    TcpStream::connect_timeout(&ollama_addr(), Duration::from_millis(200)).is_ok()
}

/// Whether an Ollama at `base_url` answers an HTTP request, the only proof it is ready.
pub fn ollama_answers_at(base_url: &str) -> bool {
    ureq::get(&format!("{base_url}/api/version"))
        .timeout(Duration::from_secs(1))
        .call()
        .is_ok()
}

/// Waits until Ollama answers, or `timeout` passes; returns whether it answers.
fn wait_for_ollama_ready(timeout: Duration) -> bool {
    let started = Instant::now();
    loop {
        if ollama_answers_at(OLLAMA_BASE_URL) {
            return true;
        }
        if started.elapsed() >= timeout {
            return false;
        }
        thread::sleep(Duration::from_millis(200));
    }
}
