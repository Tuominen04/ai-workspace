//! The desktop shell of AI Workspace (ADR-0009).
//!
//! Owns the window and the commands the interface calls. It holds no state of the workspace's
//! own: everything shown comes from a core call, made when it is asked for. The one thing it
//! keeps is the open chat session, which it closes when the app exits so temporary
//! permissions never outlive the window.

use tauri::{Manager, RunEvent};

pub mod commands;
pub mod dto;

pub fn run() {
    let app = tauri::Builder::default()
        .manage(commands::OpenSession::default())
        .invoke_handler(tauri::generate_handler![
            commands::workspace_status,
            commands::workspace_config,
            commands::workspace_validate,
            commands::project_show,
            commands::workspace_search,
            commands::workspace_init,
            commands::workspace_set_root,
            commands::project_create,
            commands::permission_list,
            commands::permission_grant,
            commands::permission_revoke,
            commands::chat_open,
            commands::chat_send,
            commands::chat_close,
        ])
        .build(tauri::generate_context!());

    let app = match app {
        Ok(app) => app,
        Err(error) => {
            eprintln!("error: could not start the desktop app: {error}");
            std::process::exit(1);
        }
    };

    app.run(|handle, event| {
        if let RunEvent::Exit = event {
            // Best effort: a failure here has nobody left to tell, and the core's `Drop`
            // closes the permission session even if `close` is never reached.
            let _ = commands::session_close(&handle.state::<commands::OpenSession>());
        }
    });
}
