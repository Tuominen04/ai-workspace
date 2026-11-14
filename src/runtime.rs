use crate::error::{AppError, Result};
use serde::{Deserialize, Serialize};
use std::net::{SocketAddr, TcpStream};
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

const OLLAMA_PORT: u16 = 11434;
pub const OLLAMA_BASE_URL: &str = "http://127.0.0.1:11434";
const CHAT_REQUEST_TIMEOUT: Duration = Duration::from_secs(120);

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

#[derive(Deserialize)]
struct ChatResponse {
    message: ChatMessage,
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

    pub fn send_chat_message(
        &self,
        base_url: &str,
        model: &str,
        messages: &[ChatMessage],
    ) -> Result<ChatMessage> {
        let request_body = ChatRequest {
            model,
            messages,
            stream: false,
        };
        let body_bytes = serde_json::to_vec(&request_body)
            .map_err(|error| AppError::RuntimeRequestFailed(error.to_string()))?;

        let url = format!("{base_url}/api/chat");
        let response = ureq::post(&url)
            .set("Content-Type", "application/json")
            .timeout(CHAT_REQUEST_TIMEOUT)
            .send_bytes(&body_bytes)
            .map_err(|error| AppError::RuntimeRequestFailed(error.to_string()))?;

        let text = response
            .into_string()
            .map_err(|error| AppError::RuntimeRequestFailed(error.to_string()))?;

        let parsed: ChatResponse = serde_json::from_str(&text)
            .map_err(|error| AppError::RuntimeRequestFailed(error.to_string()))?;

        Ok(parsed.message)
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
