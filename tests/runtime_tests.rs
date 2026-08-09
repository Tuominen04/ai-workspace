use ai_workspace::error::AppError;
use ai_workspace::runtime::{ChatMessage, RuntimeManager};
use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;

fn spawn_mock_server(response: String) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("mock listener should bind");
    let addr = listener.local_addr().expect("mock listener should have a local addr");
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
    let response = "HTTP/1.1 500 Internal Server Error\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".to_string();
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
