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

#[test]
fn clone_copies_everything_but_the_permission_state() {
    let engine = test_engine("clone");
    let created = engine
        .create_project("Security", Some("Router notes"))
        .expect("created");
    engine
        .set_project_model("security", Some("sec"))
        .expect("model set");
    let source = created.project.path.clone();
    fs::write(source.join("Knowledge").join("router.md"), "# Router\n").expect("note");
    fs::write(
        source.join("Session").join("session-note-1.md"),
        "# Earlier\n",
    )
    .expect("note");
    // A grant makes the permission session state exist.
    let outside = engine.paths().root.join("outside");
    fs::create_dir_all(&outside).expect("outside folder");
    engine
        .grant(Some("security"), outside.to_str().expect("utf-8 path"))
        .expect("grant");
    assert!(source.join("Session").join("permissions.toml").is_file());
    let before = snapshot(&source);

    let (clone, metadata) = engine
        .clone_project("security")
        .expect("clone should succeed");
    assert_eq!(clone.name, "security-clone");
    assert_eq!(clone.path, engine.paths().projects.join("security-clone"));
    assert_eq!(metadata.display_name, "Security (Clone)");
    assert_eq!(metadata.project_id, "security-clone");
    assert_eq!(metadata.description, "Router notes");
    assert_eq!(metadata.model.as_deref(), Some("sec"));

    let cloned = snapshot(&clone.path);
    let expected: Vec<_> = before
        .iter()
        .filter(|(rel, _)| rel != "Session/permissions.toml" && rel != "project.toml")
        .cloned()
        .collect();
    let got: Vec<_> = cloned
        .iter()
        .filter(|(rel, _)| rel != "project.toml")
        .cloned()
        .collect();
    assert_eq!(
        got, expected,
        "every file but the permission state is copied unchanged"
    );
    assert_eq!(snapshot(&source), before, "the original is unchanged");

    let (_, shown) = engine
        .project_details("security-clone")
        .expect("the clone loads");
    assert_eq!(shown.display_name, "Security (Clone)");
    assert!(engine.validate().expect("validate").valid);
}

#[test]
fn clone_refuses_an_existing_destination_and_writes_nothing() {
    let engine = test_engine("clone-exists");
    engine.create_project("Router", None).expect("created");
    engine.clone_project("router").expect("first clone");
    fs::write(
        engine
            .paths()
            .projects
            .join("router-clone/Knowledge/mine.md"),
        "keep me",
    )
    .expect("note in the clone");

    match engine.clone_project("router") {
        Err(AppError::AlreadyExists(path)) => {
            assert_eq!(path, engine.paths().projects.join("router-clone"))
        }
        other => panic!("expected AlreadyExists, got {other:?}"),
    }
    assert_eq!(
        fs::read_to_string(
            engine
                .paths()
                .projects
                .join("router-clone/Knowledge/mine.md")
        )
        .expect("still there"),
        "keep me"
    );
}

#[test]
fn clone_is_gated_by_validation() {
    let engine = test_engine("clone-gated");
    engine.create_project("Router", None).expect("created");
    let broken = engine.create_project("Broken", None).expect("created");
    fs::write(broken.project.path.join("project.toml"), "not = [valid").expect("break it");
    assert!(matches!(
        engine.clone_project("router"),
        Err(AppError::ValidationGateBlocked { .. })
    ));
    assert!(!engine.paths().projects.join("router-clone").exists());
}

#[cfg(unix)]
#[test]
fn clone_refuses_links_and_leaves_no_partial_copy() {
    let engine = test_engine("clone-link");
    let created = engine.create_project("Router", None).expect("created");
    std::os::unix::fs::symlink("/etc", created.project.path.join("Knowledge/etc")).expect("link");
    assert!(engine.clone_project("router").is_err());
    assert!(
        !engine.paths().projects.join("router-clone").exists(),
        "a failed clone removes what it wrote"
    );
}
