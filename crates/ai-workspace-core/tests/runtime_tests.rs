use ai_workspace_core::error::AppError;
use ai_workspace_core::runtime::{ChatMessage, RuntimeManager};
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
fn show_model_maps_an_unknown_model_to_runtime_request_failed() {
    let response =
        "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".to_string();
    let base_url = spawn_mock_server(response);
    let result = RuntimeManager::new().show_model(&base_url, "missing");
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
