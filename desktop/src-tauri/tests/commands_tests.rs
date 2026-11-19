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
    let sent = commands::session_send(&open, "hello", &mut |_| {}).expect_err("nothing to send to");
    assert_eq!(sent.kind, "Shell");
    let closed = commands::session_close(&open).expect("closing nothing is fine");
    assert!(closed.is_none());
}

#[test]
fn init_creates_a_workspace_and_keeps_an_existing_configuration() {
    let engine = test_engine("init");
    let first = commands::init(&engine).expect("init should succeed");
    assert!(first.config_created);
    assert!(engine.paths().workspace_config_file().exists());
    let again = commands::init(&engine).expect("init twice is fine");
    assert!(!again.config_created, "an existing configuration is kept");

    let _ = std::fs::remove_dir_all(&engine.paths().root);
}

#[test]
fn a_project_is_created_and_a_bad_name_refused() {
    let engine = test_engine("create");
    commands::init(&engine).expect("init should succeed");

    let created = commands::create_project(&engine, "Home Lab", Some("  Router notes  "))
        .expect("project should be created");
    assert_eq!(created.name, "home-lab");
    assert_eq!(created.description, "Router notes");
    assert_eq!(commands::status(&engine).expect("status").projects.len(), 1);

    let refused = commands::create_project(&engine, "Config", None).expect_err("reserved name");
    assert_eq!(refused.kind, "InvalidProjectName");

    let _ = std::fs::remove_dir_all(&engine.paths().root);
}

#[test]
fn permissions_are_granted_listed_and_revoked() {
    let engine = test_engine("permissions");
    commands::init(&engine).expect("init should succeed");
    commands::create_project(&engine, "Router", None).expect("project should be created");
    let outside = engine.paths().root.join("outside");
    std::fs::create_dir_all(&outside).expect("test should make a folder to grant");
    let outside_text = outside.display().to_string();

    let granted = commands::grant(&engine, "router", &outside_text).expect("grant should work");
    assert!(granted.created);
    let again = commands::grant(&engine, "router", &outside_text).expect("grants are idempotent");
    assert!(!again.created);

    let page = commands::permissions(&engine, "router", None, None, None).expect("list");
    assert_eq!(page.total, 3, "Base, the project and the temporary grant");
    let temporary = page
        .grants
        .iter()
        .find(|grant| grant.category == "temporary")
        .expect("the temporary grant is listed");
    assert_eq!(temporary.status, "active");

    let filtered =
        commands::permissions(&engine, "router", Some("outside"), None, None).expect("filter");
    assert_eq!(filtered.total, 1);

    commands::revoke(&engine, "router", &temporary.path).expect("revoke should work");
    let after = commands::permissions(&engine, "router", None, None, None).expect("list");
    assert!(after.grants.iter().all(|grant| grant.category == "default"));

    let missing = engine.paths().root.join("does-not-exist");
    let denied = commands::grant(&engine, "router", &missing.display().to_string())
        .expect_err("a missing path is refused");
    assert_eq!(denied.kind, "InvalidPermissionPath");
    let empty = commands::grant(&engine, "router", "  ").expect_err("an empty path is refused");
    assert_eq!(empty.kind, "Shell");

    let _ = std::fs::remove_dir_all(&engine.paths().root);
}

#[test]
fn a_projects_model_is_set_and_cleared_from_the_app() {
    let engine = test_engine("set-model");
    commands::init(&engine).expect("init should succeed");
    commands::create_project(&engine, "Security", None).expect("project should be created");

    let set = commands::set_project_model(&engine, "security", Some("sec")).expect("set");
    assert_eq!(set.model.as_deref(), Some("sec"));
    let cleared = commands::set_project_model(&engine, "security", None).expect("clear");
    assert_eq!(cleared.model, None);
    let bad = commands::set_project_model(&engine, "security", Some("a b")).expect_err("bad name");
    assert_eq!(bad.kind, "InvalidModelName");

    let _ = std::fs::remove_dir_all(&engine.paths().root);
}

#[test]
fn notes_are_saved_on_request_and_drafting_needs_a_session() {
    let engine = test_engine("notes");
    commands::init(&engine).expect("init should succeed");
    commands::create_project(&engine, "Security", None).expect("project should be created");

    let saved = commands::save_note(&engine, "security", "# Notes\n- one").expect("saved");
    assert!(saved.path.contains("session-note-"));
    assert!(std::path::Path::new(&saved.path).exists());
    let empty = commands::save_note(&engine, "security", " ").expect_err("empty refused");
    assert_eq!(empty.kind, "Message");

    let open = OpenSession::default();
    let no_session = commands::session_draft_note(&open).expect_err("no session open");
    assert_eq!(no_session.kind, "Shell");

    let _ = std::fs::remove_dir_all(&engine.paths().root);
}

#[test]
fn archive_and_restore_move_a_project_and_status_lists_it() {
    let engine = test_engine("archive");
    engine.init().expect("init should succeed");
    engine
        .create_project("Router", None)
        .expect("project should be created");
    let open = OpenSession::default();

    let moved = commands::archive(&engine, &open, "router").expect("archive should succeed");
    assert!(moved.path.ends_with("Archive/router"));
    assert!(moved.session_closed.is_none(), "no session was open");
    let status = commands::status(&engine).expect("status should succeed");
    assert!(status.projects.is_empty());
    assert_eq!(status.archived.len(), 1);
    assert_eq!(status.archived[0].name, "router");

    engine
        .create_project("Router", None)
        .expect("a new project of the same name");
    let refused = commands::restore(&engine, "router").expect_err("the destination exists");
    assert_eq!(refused.kind, "AlreadyExists");
    assert!(refused.message.starts_with("Already exists: "));

    std::fs::remove_dir_all(engine.paths().projects.join("router")).expect("clear the way");
    let restored = commands::restore(&engine, "router").expect("restore should succeed");
    assert_eq!(
        restored.path,
        engine.paths().projects.join("router").display().to_string()
    );
    let status = commands::status(&engine).expect("status should succeed");
    assert_eq!(status.projects.len(), 1);
    assert!(status.archived.is_empty());

    let _ = std::fs::remove_dir_all(&engine.paths().root);
}

#[test]
fn clone_returns_the_new_project_and_refuses_a_taken_name() {
    let engine = test_engine("clone");
    engine.init().expect("init should succeed");
    engine
        .create_project("Router", Some("Firewall notes"))
        .expect("project should be created");

    let clone = commands::clone(&engine, "router").expect("clone should succeed");
    assert_eq!(clone.name, "router-clone");
    assert_eq!(clone.display_name, "Router (Clone)");
    assert_eq!(clone.description, "Firewall notes");

    let refused = commands::clone(&engine, "router").expect_err("the clone exists");
    assert_eq!(refused.kind, "AlreadyExists");

    let _ = std::fs::remove_dir_all(&engine.paths().root);
}

#[test]
fn deleting_a_project_that_is_not_there_is_refused() {
    // The Trash itself is covered in the core (trash_tests), against a temporary Trash; this
    // test must not reach the real one.
    let engine = test_engine("delete");
    engine.init().expect("init should succeed");
    let open = OpenSession::default();
    let refused = commands::delete(&engine, &open, "missing", false, false)
        .expect_err("there is no such project");
    assert_eq!(refused.kind, "ProjectNotFound");
    let refused = commands::delete(&engine, &open, "../Base", false, true)
        .expect_err("only project folders can be deleted");
    assert_eq!(refused.kind, "ProjectNotFound");
    assert!(engine.paths().base.is_dir());

    let _ = std::fs::remove_dir_all(&engine.paths().root);
}

#[test]
fn rename_returns_the_project_under_its_new_name() {
    let engine = test_engine("rename");
    engine.init().expect("init should succeed");
    engine
        .create_project("Security", None)
        .expect("project should be created");
    engine
        .create_project("Lab", None)
        .expect("project should be created");
    let open = OpenSession::default();

    let renamed = commands::rename(&engine, &open, "security", "Home Security").expect("renamed");
    assert_eq!(renamed.project.name, "home-security");
    assert_eq!(renamed.project.display_name, "Home Security");
    assert!(renamed.session_closed.is_none());

    let refused = commands::rename(&engine, &open, "lab", "Home Security").expect_err("taken");
    assert_eq!(refused.kind, "AlreadyExists");
    let refused = commands::rename(&engine, &open, "lab", "Archive").expect_err("reserved");
    assert_eq!(refused.kind, "InvalidProjectName");

    let _ = std::fs::remove_dir_all(&engine.paths().root);
}

#[test]
fn closing_asks_first_only_about_an_answered_chat() {
    use std::sync::atomic::Ordering;
    let open = OpenSession::default();
    let allowed = commands::QuitAllowed::default();
    assert_eq!(
        commands::needs_save_prompt(&open, &allowed),
        None,
        "no chat open: close at once"
    );
    allowed.0.store(true, Ordering::SeqCst);
    assert_eq!(
        commands::needs_save_prompt(&open, &allowed),
        None,
        "answered already: close"
    );
}
