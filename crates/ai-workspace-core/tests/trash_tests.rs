//! Deleting a project moves it to the desktop Trash (ADR-0015). One test in its own binary,
//! because it points `XDG_DATA_HOME` at a temporary folder so no real Trash is touched.

use ai_workspace_core::Engine;
use ai_workspace_core::paths::{RootSource, WorkspacePaths};
use ai_workspace_core::workspace::Deleted;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(target_os = "linux")]
#[test]
fn a_deleted_project_lands_in_the_trash_and_can_be_restored_from_there() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    let base = std::env::temp_dir().join(format!("ai-workspace-trash-{unique}"));
    let data_home = base.join("data");
    fs::create_dir_all(&data_home).expect("data home");
    // SAFETY: this binary holds this one test, so nothing else reads the environment.
    unsafe { std::env::set_var("XDG_DATA_HOME", &data_home) };

    let engine = Engine::at(
        WorkspacePaths::from_root(base.join("AI")),
        RootSource::Environment,
    );
    engine.init().expect("init");
    let created = engine
        .create_project("Router", Some("notes"))
        .expect("created");
    engine.archive_project("router").expect("archived");
    engine.create_project("Lab", None).expect("created");

    let (path, how) = engine.delete_project("lab", false, false).expect("trashed");
    assert_eq!(how, Deleted::Trashed);
    assert!(!path.exists(), "the project left the workspace");
    let trash = data_home.join("Trash");
    assert!(
        trash.join("files/lab/project.toml").is_file(),
        "its files are in the Trash"
    );
    let info = fs::read_to_string(trash.join("info/lab.trashinfo")).expect("trash info");
    assert!(
        info.contains("Path=") && info.contains("DeletionDate="),
        "{info}"
    );

    let (archived_path, how) = engine
        .delete_project("router", true, false)
        .expect("trashed");
    assert_eq!(how, Deleted::Trashed);
    assert_eq!(
        archived_path,
        engine.paths().projects.join("Archive/router")
    );
    assert!(trash.join("files/router").is_dir());
    assert!(!created.project.path.exists() && !archived_path.exists());

    let _ = fs::remove_dir_all(&base);
}
