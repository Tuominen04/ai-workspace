//! Session notes (ADR-0012): saved only on request, never overwritten, read back as context.

use ai_workspace_core::Engine;
use ai_workspace_core::config::load_workspace_config;
use ai_workspace_core::context::build_context_snapshot;
use ai_workspace_core::paths::{RootSource, WorkspacePaths};
use ai_workspace_core::workspace::save_session_note;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn a_note_is_saved_by_date_never_overwritten_and_read_back_as_context() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("ai-workspace-note-{unique}"));
    let engine = Engine::at(
        WorkspacePaths::from_root(root.clone()),
        RootSource::Environment,
    );
    engine.init().expect("init");
    let created = engine.create_project("Security", None).expect("project");
    let config = load_workspace_config(engine.paths()).expect("config");

    // 1794690257 is 2026-11-14 21:04:17 UTC.
    let note = "# Router hardening\n- Closed WAN SSH\n- Next: review the guest VLAN";
    let first = save_session_note(engine.paths(), &config, "security", note, 1_794_690_257)
        .expect("note saved");
    assert_eq!(
        first,
        created
            .project
            .path
            .join("Session")
            .join("session-note-2026-11-14-210417.md")
    );
    assert_eq!(
        fs::read_to_string(&first).expect("read"),
        format!("{note}\n")
    );

    let second = save_session_note(
        engine.paths(),
        &config,
        "security",
        "# Again",
        1_794_690_257,
    )
    .expect("second note saved");
    assert!(
        second.ends_with("session-note-2026-11-14-210417-2.md"),
        "{second:?}"
    );
    assert_eq!(
        fs::read_to_string(&first).expect("read"),
        format!("{note}\n"),
        "not overwritten"
    );

    let empty = save_session_note(engine.paths(), &config, "security", "  \n ", 1_794_690_300);
    assert!(empty.is_err(), "an empty note is refused");

    let context =
        build_context_snapshot(engine.paths(), Some(&created.project), Some("guest VLAN"))
            .expect("context");
    assert!(
        context.session_files.contains(&first),
        "later sessions read the note"
    );
    assert!(context.assembled_content.contains("review the guest VLAN"));

    let via_engine = engine
        .save_session_note("security", "# Through the engine")
        .expect("engine saves too");
    assert!(via_engine.starts_with(created.project.path.join("Session")));

    let _ = fs::remove_dir_all(&root);
}
