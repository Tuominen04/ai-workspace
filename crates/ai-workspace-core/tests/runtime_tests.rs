use ai_workspace_core::error::AppError;
use ai_workspace_core::runtime::{ChatMessage, RuntimeManager, ollama_answers_at};
use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;

fn spawn_mock_server(response: String) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("mock listener should bind");
    let addr = listener
        .local_addr()
        .expect("mock listener should have a local addr");
    thread::spawn(move || {
        if let Ok((mut stream, _)) = listener.accept() {
            let mut buffer = [0u8; 4096];
            let _ = stream.read(&mut buffer);
            let _ = stream.write_all(response.as_bytes());
        }
    });
    format!("http://{addr}")
}

#[test]
fn send_chat_message_parses_successful_response() {
    let body = r#"{"model":"qwen3.6:latest","created_at":"now","message":{"role":"assistant","content":"hello there"},"done":true}"#;
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    );
    let base_url = spawn_mock_server(response);

    let runtime = RuntimeManager::new();
    let messages = vec![
        ChatMessage::system("workspace context".to_string()),
        ChatMessage::user("hi".to_string()),
    ];
    let reply = runtime
        .send_chat_message(&base_url, "qwen3.6:latest", &messages)
        .expect("chat request should succeed against the mock server");

    assert_eq!(reply.role, "assistant");
    assert_eq!(reply.content, "hello there");
}

#[test]
fn send_chat_message_maps_non_200_to_runtime_request_failed() {
    let response =
        "HTTP/1.1 500 Internal Server Error\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
            .to_string();
    let base_url = spawn_mock_server(response);

    let runtime = RuntimeManager::new();
    let messages = vec![ChatMessage::system("workspace context".to_string())];
    let result = runtime.send_chat_message(&base_url, "qwen3.6:latest", &messages);

    assert!(matches!(result, Err(AppError::RuntimeRequestFailed(_))));
}

#[test]
fn send_chat_message_maps_malformed_body_to_runtime_request_failed() {
    let body = "not json";
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    );
    let base_url = spawn_mock_server(response);

    let runtime = RuntimeManager::new();
    let messages = vec![ChatMessage::system("workspace context".to_string())];
    let result = runtime.send_chat_message(&base_url, "qwen3.6:latest", &messages);

    assert!(matches!(result, Err(AppError::RuntimeRequestFailed(_))));
}

fn ok_json(body: &str) -> String {
    format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    )
}

#[test]
fn show_model_reads_the_models_instructions_and_context_window() {
    let body = r#"{"system":"You are a security advisor.\nNever weaken a firewall.","parameters":"temperature 0.3\nnum_ctx 8192","modelfile":"FROM qwen"}"#;
    let base_url = spawn_mock_server(ok_json(body));
    let info = RuntimeManager::new()
        .show_model(&base_url, "sec")
        .expect("show should succeed against the mock server");
    assert_eq!(
        info.system.as_deref(),
        Some("You are a security advisor.\nNever weaken a firewall.")
    );
    assert_eq!(info.num_ctx, Some(8192));
}

#[test]
fn show_model_falls_back_to_the_system_line_in_the_modelfile() {
    let body = r#"{"parameters":"temperature 0.3","modelfile":"FROM qwen3.8:27b\nPARAMETER temperature 0.3\nSYSTEM \"\"\"\nKeep Fedora hardened.\nExplain every change.\n\"\"\"\n"}"#;
    let base_url = spawn_mock_server(ok_json(body));
    let info = RuntimeManager::new()
        .show_model(&base_url, "sec")
        .expect("show should succeed");
    assert_eq!(
        info.system.as_deref(),
        Some("Keep Fedora hardened.\nExplain every change.")
    );
    assert_eq!(info.num_ctx, None, "no num_ctx parameter was set");
}

#[test]
fn show_model_reports_a_model_without_instructions() {
    let base_url = spawn_mock_server(ok_json(r#"{"modelfile":"FROM qwen3.6"}"#));
    let info = RuntimeManager::new()
        .show_model(&base_url, "plain")
        .expect("show should succeed");
    assert_eq!(info.system, None);
}

#[test]
fn show_model_maps_an_unknown_model_to_model_not_installed() {
    // Was a generic RuntimeRequestFailed ("status code 404") until #42.
    let response =
        "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".to_string();
    let base_url = spawn_mock_server(response);
    let result = RuntimeManager::new().show_model(&base_url, "missing");
    assert!(matches!(
        result,
        Err(AppError::ModelNotInstalled { ref model, .. }) if model == "missing"
    ));
}

#[test]
fn other_http_errors_stay_runtime_request_failed() {
    let response =
        "HTTP/1.1 500 Internal Server Error\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
            .to_string();
    let base_url = spawn_mock_server(response);
    let result = RuntimeManager::new().show_model(&base_url, "sec");
    assert!(matches!(result, Err(AppError::RuntimeRequestFailed(_))));
}

#[test]
fn installed_models_lists_names_sorted() {
    let body = r#"{"models":[{"name":"sec:latest","size":1},{"name":"qwen3.6:latest","size":2}]}"#;
    let base_url = spawn_mock_server(ok_json(body));
    let names = RuntimeManager::new()
        .installed_models(&base_url)
        .expect("tags should succeed");
    assert_eq!(names, ["qwen3.6:latest", "sec:latest"]);
}

fn ndjson(lines: &[&str]) -> String {
    let body = lines.join("\n") + "\n";
    format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/x-ndjson\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    )
}

#[test]
fn a_streamed_reply_arrives_in_pieces_and_is_assembled() {
    let base_url = spawn_mock_server(ndjson(&[
        r###"{"message":{"role":"assistant","content":"## My "},"done":false}"###,
        r#"{"message":{"role":"assistant","content":"Rules"},"done":false}"#,
        r#"{"message":{"role":"assistant","content":"\n1. Exact commands"},"done":false}"#,
        r#"{"message":{"role":"assistant","content":""},"done":true,"total_duration":1}"#,
    ]));
    let mut pieces = Vec::new();
    let reply = RuntimeManager::new()
        .stream_chat_message(
            &base_url,
            "sec",
            &[ChatMessage::user("rules?".to_string())],
            &mut |piece| pieces.push(piece.to_string()),
        )
        .expect("the stream should complete");
    assert_eq!(pieces, ["## My ", "Rules", "\n1. Exact commands"]);
    assert_eq!(reply.content, "## My Rules\n1. Exact commands");
    assert_eq!(reply.role, "assistant");
}

#[test]
fn an_error_in_the_stream_fails_the_request() {
    let base_url = spawn_mock_server(ndjson(&[
        r#"{"message":{"role":"assistant","content":"Half"},"done":false}"#,
        r#"{"error":"model ran out of memory"}"#,
    ]));
    let result = RuntimeManager::new().stream_chat_message(
        &base_url,
        "sec",
        &[ChatMessage::user("hi".to_string())],
        &mut |_| {},
    );
    match result {
        Err(AppError::RuntimeRequestFailed(reason)) => assert!(reason.contains("out of memory")),
        other => panic!("expected a failure, got {other:?}"),
    }
}

#[test]
fn a_stream_that_stops_before_done_fails_the_request() {
    let base_url = spawn_mock_server(ndjson(&[
        r#"{"message":{"role":"assistant","content":"Half an answ"},"done":false}"#,
    ]));
    let result = RuntimeManager::new().send_chat_message(
        &base_url,
        "sec",
        &[ChatMessage::user("hi".to_string())],
    );
    assert!(matches!(result, Err(AppError::RuntimeRequestFailed(_))));
}

fn ndjson_response(lines: &[&str]) -> String {
    let body = lines.join("\n");
    format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/x-ndjson\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    )
}

#[test]
fn an_empty_reply_is_a_failure_that_says_the_window_was_full() {
    // What Ollama sends when the conversation fills the window: no text, done_reason "length".
    let base_url = spawn_mock_server(ndjson_response(&[
        r#"{"message":{"role":"assistant","content":""},"done":false}"#,
        r#"{"message":{"role":"assistant","content":""},"done":true,"done_reason":"length"}"#,
    ]));
    let messages = vec![ChatMessage::user("hi".to_string())];
    match RuntimeManager::new().send_chat_message(&base_url, "sec", &messages) {
        Err(AppError::RuntimeRequestFailed(reason)) => {
            assert!(reason.contains("context window was full"), "{reason}")
        }
        other => panic!("expected a failure, got {other:?}"),
    }
}

#[test]
fn an_empty_reply_for_another_reason_is_a_failure_too() {
    let base_url = spawn_mock_server(ndjson_response(&[
        r#"{"message":{"role":"assistant","content":"  "},"done":true,"done_reason":"stop"}"#,
    ]));
    let messages = vec![ChatMessage::user("hi".to_string())];
    match RuntimeManager::new().send_chat_message(&base_url, "sec", &messages) {
        Err(AppError::RuntimeRequestFailed(reason)) => {
            assert!(reason.contains("empty reply"), "{reason}")
        }
        other => panic!("expected a failure, got {other:?}"),
    }
}

#[test]
fn a_missing_model_is_said_plainly_with_the_download_command() {
    // What Ollama answers for a model it does not have.
    let body = r#"{"error":"model 'qwen3.6:latest' not found"}"#;
    let not_found = format!(
        "HTTP/1.1 404 Not Found\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    );
    let messages = vec![ChatMessage::user("hi".to_string())];
    let chat = RuntimeManager::new().send_chat_message(
        &spawn_mock_server(not_found.clone()),
        "qwen3.6:latest",
        &messages,
    );
    match chat {
        Err(AppError::ModelNotInstalled {
            model,
            pull_command,
        }) => {
            assert_eq!(model, "qwen3.6:latest");
            assert_eq!(pull_command, "ollama pull qwen3.6:latest");
        }
        other => panic!("expected ModelNotInstalled, got {other:?}"),
    }

    let show = RuntimeManager::new().show_model(&spawn_mock_server(not_found), "qwen3.6:latest");
    let message = show.expect_err("a missing model").to_string();
    assert!(
        message.starts_with("the model 'qwen3.6:latest' is not installed in Ollama")
            && !message.contains("404"),
        "{message}"
    );
}

#[test]
fn a_port_that_only_accepts_is_not_a_ready_ollama() {
    // What rootless Podman's published port does before Ollama inside listens: the connection
    // is accepted, then reset without an answer.
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    let addr = listener.local_addr().expect("addr");
    thread::spawn(move || {
        for stream in listener.incoming().take(3) {
            drop(stream);
        }
    });
    assert!(!ollama_answers_at(&format!("http://{addr}")));
}

#[test]
fn an_ollama_that_answers_is_ready() {
    let base_url = spawn_mock_server(ok_json(r#"{"version":"0.12.0"}"#));
    assert!(ollama_answers_at(&base_url));
}
