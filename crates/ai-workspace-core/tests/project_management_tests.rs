//! Archiving and restoring projects (ADR-0015, specs/008-project-management).

use ai_workspace_core::paths::{RootSource, WorkspacePaths};
use ai_workspace_core::{AppError, Engine};
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

fn test_engine(name: &str) -> Engine {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("ai-workspace-pm-{name}-{unique}"));
    let engine = Engine::at(WorkspacePaths::from_root(root), RootSource::Environment);
    engine.init().expect("init should succeed");
    engine
}

fn names(projects: &[ai_workspace_core::workspace::Project]) -> Vec<String> {
    projects.iter().map(|p| p.name.clone()).collect()
}

/// Every file under `dir`, relative, with its bytes.
fn snapshot(dir: &Path) -> Vec<(String, Vec<u8>)> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(next) = stack.pop() {
        for entry in fs::read_dir(&next).expect("readable test folder") {
            let path = entry.expect("entry").path();
            if path.is_dir() {
                stack.push(path);
            } else {
                let rel = path
                    .strip_prefix(dir)
                    .expect("inside")
                    .display()
                    .to_string();
                out.push((rel, fs::read(&path).expect("readable test file")));
            }
        }
    }
    out.sort();
    out
}

#[test]
fn archive_then_restore_moves_the_folder_unchanged() {
    let engine = test_engine("round-trip");
    let created = engine
        .create_project("Router", Some("Firewall notes"))
        .expect("project should be created");
    fs::write(
        created
            .project
            .path
            .join("Session")
            .join("session-note-1.md"),
        "# Earlier session\n",
    )
    .expect("test should write a note");
    let before = snapshot(&created.project.path);

    let archived = engine
        .archive_project("router")
        .expect("archive should succeed");
    let archive = engine.paths().projects.join("Archive");
    assert_eq!(archived, archive.join("router"));
    assert!(!created.project.path.exists(), "the project left Projects/");
    assert_eq!(snapshot(&archived), before, "archived files are unchanged");

    assert!(
        names(&engine.projects().expect("projects")).is_empty(),
        "the archive is not a project"
    );
    assert_eq!(
        names(&engine.archived_projects().expect("archived")),
        ["router"]
    );
    assert!(
        engine.validate().expect("validate").valid,
        "the archive folder raises no issue"
    );

    let restored = engine
        .restore_project("router")
        .expect("restore should succeed");
    assert_eq!(restored, created.project.path);
    assert_eq!(snapshot(&restored), before, "restored files are unchanged");
    assert_eq!(names(&engine.projects().expect("projects")), ["router"]);
    assert!(engine.archived_projects().expect("archived").is_empty());
}

#[test]
fn an_existing_destination_is_refused_and_nothing_moves() {
    let engine = test_engine("already-exists");
    engine.create_project("Router", None).expect("created");
    engine.archive_project("router").expect("archived");
    engine
        .create_project("Router", None)
        .expect("created again");

    match engine.archive_project("router") {
        Err(AppError::AlreadyExists(path)) => {
            assert!(path.ends_with("Archive/router"));
            assert!(
                AppError::AlreadyExists(path)
                    .to_string()
                    .starts_with("Already exists: ")
            );
        }
        other => panic!("expected AlreadyExists, got {other:?}"),
    }
    assert!(
        engine.paths().projects.join("router").is_dir(),
        "the project stayed"
    );

    match engine.restore_project("router") {
        Err(AppError::AlreadyExists(path)) => {
            assert_eq!(path, engine.paths().projects.join("router"))
        }
        other => panic!("expected AlreadyExists, got {other:?}"),
    }
    assert!(
        engine.paths().projects.join("Archive/router").is_dir(),
        "the archived one stayed"
    );
}

#[test]
fn only_plain_existing_folder_names_are_moved() {
    let engine = test_engine("names");
    engine.create_project("Router", None).expect("created");
    for bad in [
        "",
        ".",
        "..",
        "Archive",
        "../Base",
        "router/Knowledge",
        "missing",
    ] {
        assert!(
            matches!(
                engine.archive_project(bad),
                Err(AppError::ProjectNotFound(_))
            ),
            "{bad:?} must be refused"
        );
    }
    assert!(matches!(
        engine.restore_project("router"),
        Err(AppError::ProjectNotFound(_))
    ));
    assert!(engine.paths().base.is_dir() && engine.paths().projects.join("router").is_dir());
}

#[test]
fn archive_is_a_reserved_project_name() {
    let engine = test_engine("reserved");
    assert!(matches!(
        engine.create_project("Archive", None),
        Err(AppError::InvalidProjectName(_))
    ));
}

#[test]
fn archiving_works_while_validation_blocks() {
    let engine = test_engine("ungated");
    let created = engine.create_project("Broken", None).expect("created");
    fs::write(created.project.path.join("project.toml"), "not = [valid").expect("break it");
    assert!(
        !engine.validate().expect("validate").valid,
        "the workspace is blocked"
    );

    engine
        .archive_project("broken")
        .expect("a broken project can still be archived");
    assert!(
        engine.validate().expect("validate").valid,
        "and the workspace is clear again"
    );
}
