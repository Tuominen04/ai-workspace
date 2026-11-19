//! The 1.0 workspace format (ADR-0016, docs/workspace-format.md). `fixtures/workspace-1.0` was
//! written by 1.0 itself; every 1.x must keep reading it unchanged. Never edit the fixture to
//! make a test pass: a failure here means a change broke the 1.x promise.

use ai_workspace_core::config::Runtime;
use ai_workspace_core::paths::{RootSource, WorkspacePaths, read_user_root};
use ai_workspace_core::workspace::PermissionViewOptions;
use ai_workspace_core::{Engine, context};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

fn fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn copy_tree(from: &Path, to: &Path) {
    fs::create_dir_all(to).expect("create");
    for entry in fs::read_dir(from).expect("read fixture") {
        let entry = entry.expect("entry");
        let target = to.join(entry.file_name());
        if entry.file_type().expect("type").is_dir() {
            copy_tree(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), &target).expect("copy");
        }
    }
}

/// A fresh copy of the 1.0 workspace, so tests can never change the fixture itself.
fn workspace_1_0(name: &str) -> Engine {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock after epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("ai-workspace-format-{name}-{unique}"));
    copy_tree(&fixture().join("workspace-1.0"), &root);
    Engine::at(WorkspacePaths::from_root(root), RootSource::Environment)
}

fn names(projects: &[ai_workspace_core::workspace::Project]) -> Vec<String> {
    projects.iter().map(|p| p.name.clone()).collect()
}

#[test]
fn a_1_0_workspace_loads_and_validates_clean() {
    let engine = workspace_1_0("loads");
    let result = engine.validate().expect("validate");
    assert!(result.valid, "a 1.0 workspace must validate: {result:?}");

    assert_eq!(names(&engine.projects().expect("projects")), ["home-lab"]);
    assert_eq!(
        names(&engine.archived_projects().expect("archived")),
        ["old-lab"]
    );

    let config = engine.config().expect("workspace.toml");
    assert_eq!(config.default_model, "qwen3.6:latest");
    assert_eq!(config.runtime, Runtime::Ollama);
    assert_eq!(
        config.default_project_directories,
        ["Knowledge", "Specs", "Resources", "Session"]
    );

    let (_, metadata) = engine.project_details("home-lab").expect("project.toml");
    assert_eq!(metadata.display_name, "Home Lab");
    assert_eq!(metadata.description, "Router and network notes");
    assert_eq!(metadata.model.as_deref(), Some("qwen3:8b"));
}

#[test]
fn a_1_0_workspace_keeps_its_permissions_notes_and_context() {
    let engine = workspace_1_0("content");
    let (_, page) = engine
        .permissions(
            Some("home-lab"),
            PermissionViewOptions {
                filter: None,
                page: 1,
                page_size: 50,
            },
        )
        .expect("Session/permissions.toml");
    assert!(
        page.entries
            .iter()
            .any(|g| g.display_path == "/home/me/granted"),
        "the temporary grant written by 1.0 is read: {:?}",
        page.entries
    );

    let project = engine
        .projects()
        .expect("projects")
        .into_iter()
        .next()
        .expect("home-lab");
    let sources = context::readable_sources(engine.paths(), Some(&project)).expect("sources");
    let read: Vec<String> = sources
        .sources
        .iter()
        .map(|(_, p)| p.file_name().unwrap().to_string_lossy().into_owned())
        .collect();
    for expected in [
        "intro.md",
        "router.md",
        "README.md",
        "session-note-2026-11-19-120000.md",
    ] {
        assert!(read.iter().any(|n| n == expected), "{expected} in {read:?}");
    }

    let outcome = engine
        .search("firewall", Some("home-lab"), 10)
        .expect("search");
    assert!(
        outcome
            .results
            .iter()
            .any(|r| r.path.ends_with("router.md"))
    );
}

#[test]
fn unknown_keys_from_a_later_version_are_ignored() {
    let engine = workspace_1_0("unknown-keys");
    let root = engine.paths().root.clone();
    for file in [
        "Config/workspace.toml",
        "Projects/home-lab/project.toml",
        "Projects/home-lab/Session/permissions.toml",
    ] {
        let path = root.join(file);
        let text = fs::read_to_string(&path).expect("read");
        // Top-level keys must come before the first table.
        fs::write(&path, format!("added_in_a_later_version = true\n{text}")).expect("write");
    }
    assert!(engine.validate().expect("validate").valid);
    engine.config().expect("workspace.toml still loads");
    engine
        .project_details("home-lab")
        .expect("project.toml still loads");
    engine
        .permissions(
            Some("home-lab"),
            PermissionViewOptions {
                filter: None,
                page: 1,
                page_size: 50,
            },
        )
        .expect("permissions.toml still loads");
}

#[test]
fn an_archived_1_0_project_restores() {
    let engine = workspace_1_0("restore");
    engine.restore_project("old-lab").expect("restore");
    let (_, metadata) = engine
        .project_details("old-lab")
        .expect("loads once restored");
    assert_eq!(metadata.display_name, "Old Lab");
}

#[test]
fn the_1_0_user_settings_file_is_read() {
    let root = read_user_root(&fixture().join("user-config-1.0.toml")).expect("readable");
    assert_eq!(root, Some(PathBuf::from("/home/me/AI")));
}
