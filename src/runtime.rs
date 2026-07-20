use crate::error::Result;
use std::net::{SocketAddr, TcpStream};
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

const OLLAMA_PORT: u16 = 11434;

#[derive(Debug, Clone)]
pub struct RuntimeStatus {
    pub running: bool,
    pub started_here: bool,
    pub detail: String,
}

pub struct RuntimeManager {
    ollama: Option<Child>,
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
                detail: "Ollama is not installed; session continues without a local runtime".to_string(),
            }),
            Err(error) => Err(error.into()),
        }
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
