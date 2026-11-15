//! The shell's commands against temporary workspaces, without a window.
//!
//! No test opens a chat session: `Session::open` starts a real `ollama serve` wherever Ollama
//! is installed. The session commands' no-session paths are covered instead.

use ai_workspace_core::Engine;
use ai_workspace_core::paths::{RootSource, WorkspacePaths};
use aiw_desktop::commands::{self, OpenSession};
use aiw_desktop::dto::{Fix, FixAction, Severity};
use std::time::{SystemTime, UNIX_EPOCH};

fn test_engine(name: &str) -> Engine {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("ai-workspace-desktop-{name}-{unique}"));
    Engine::at(WorkspacePaths::from_root(root), RootSource::Environment)
}

#[test]
fn status_lists_projects_and_says_where_the_root_came_from() {
    let engine = test_engine("status");
    engine.init().expect("init should succeed");
    engine
        .create_project("Router", Some("Firewall notes"))
        .expect("project should be created");

    let status = commands::status(&engine).expect("status should succeed");
    assert_eq!(status.root, engine.paths().root.display().to_string());
    assert_eq!(status.root_source, "environment variable AI_WORKSPACE_HOME");
    assert_eq!(status.projects.len(), 1);
    assert_eq!(status.projects[0].name, "router");

    let details = commands::project_details(&engine, "router").expect("details should load");
    assert_eq!(details.description, "Firewall notes");

    let _ = std::fs::remove_dir_all(&engine.paths().root);
}

#[test]
fn configuration_is_read_and_a_missing_file_is_reported() {
    let engine = test_engine("config");
    engine.init().expect("init should succeed");

    let settings = commands::settings(&engine).expect("settings should load");
    assert_eq!(
        settings.active_root,
        engine.paths().root.display().to_string()
    );
    assert_eq!(settings.project_metadata_file, "project.toml");
    assert_eq!(
        settings.default_project_directories,
        ["Knowledge", "Specs", "Resources", "Session"]
    );
    assert!(settings.config_file.ends_with("workspace.toml"));

    std::fs::remove_file(engine.paths().workspace_config_file())
        .expect("test should remove the workspace config");
    let missing = commands::settings(&engine).expect_err("a missing file is an error");
    assert_eq!(missing.kind, "ConfigMissing");

    let _ = std::fs::remove_dir_all(&engine.paths().root);
}

#[test]
fn search_returns_hits_and_refuses_an_empty_query() {
    let engine = test_engine("search");
    engine.init().expect("init should succeed");
    let created = engine
        .create_project("Router", None)
        .expect("project should be created");
    std::fs::write(
        created.project.path.join("Knowledge").join("fw.md"),
        "# Firewall\nThe firewall blocks inbound traffic.\n",
    )
    .expect("test should write a knowledge file");

    let results =
        commands::search(&engine, "firewall", Some("router"), None).expect("search should work");
    assert_eq!(results.hits.len(), 1);
    assert_eq!(results.hits[0].source_kind, "project");

    let empty = commands::search(&engine, "   ", None, None).expect_err("empty query is refused");
    assert_eq!(empty.kind, "Shell");

    let _ = std::fs::remove_dir_all(&engine.paths().root);
}

#[test]
fn a_blocked_gate_and_its_issues_reach_the_interface_as_data() {
    let engine = test_engine("gate");
    engine.init().expect("init should succeed");
    let created = engine
        .create_project("Broken", None)
        .expect("project should be created");
    std::fs::write(created.project.path.join("project.toml"), "garbage = [")
        .expect("test should corrupt the project metadata");

    let refused = commands::project_details(&engine, "broken").expect_err("the gate refuses");
    assert_eq!(refused.kind, "ValidationGateBlocked");

    let validation = commands::validate(&engine).expect("validate always answers");
    assert!(!validation.valid);
    assert!(validation.blocking >= 1);
    let issue = validation
        .issues
        .iter()
        .find(|issue| issue.code == "PROJECT-METADATA-INVALID")
        .expect("the broken project should be reported");
    assert_eq!(issue.severity, Severity::Error);
    assert_eq!(
        issue.fix,
        Fix::FixThenRerun {
            step: "fix project.toml".to_string(),
            action: FixAction::Validate,
        }
    );

    let _ = std::fs::remove_dir_all(&engine.paths().root);
}

#[test]
fn chat_commands_without_a_session_answer_plainly() {
    let open = OpenSession::default();
    let sent = commands::session_send(&open, "hello").expect_err("nothing to send to");
    assert_eq!(sent.kind, "Shell");
    let closed = commands::session_close(&open).expect("closing nothing is fine");
    assert!(closed.is_none());
}
