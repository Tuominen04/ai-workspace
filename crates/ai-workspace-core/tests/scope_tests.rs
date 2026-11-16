//! What the model may read (ADR-0014): Base and the whole project by default, plus the
//! session's temporary grants while they are granted.

use ai_workspace_core::Engine;
use ai_workspace_core::context::{build_context_snapshot, readable_sources};
use ai_workspace_core::paths::{RootSource, WorkspacePaths};
use ai_workspace_core::search::SourceKind;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp(name: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("ai-workspace-scope-{name}-{unique}"))
}

#[test]
fn the_whole_project_and_granted_paths_are_read_and_revoking_removes_them() {
    let root = temp("ws");
    let outside = temp("outside");
    let engine = Engine::at(
        WorkspacePaths::from_root(root.clone()),
        RootSource::Environment,
    );
    engine.init().expect("init");
    let created = engine.create_project("Sec", None).expect("project");
    let project = created.project.clone();

    // Default scope: every project folder, not only Knowledge.
    fs::write(
        project.path.join("Specs").join("plan.md"),
        "# Plan\nfirewall plan",
    )
    .expect("w");
    fs::write(
        project.path.join("Resources").join("router.md"),
        "# Router\nfirewall rules",
    )
    .expect("w");

    // Outside the workspace: a granted file, a granted folder, and a secret inside it.
    fs::create_dir_all(outside.join("notes").join("deep")).expect("folders");
    let granted_file = outside.join("router-hardening-notes.md");
    fs::write(&granted_file, "# Hardening\nThe firewall drops WAN SSH.").expect("w");
    fs::write(
        outside.join("notes").join("vlan.md"),
        "# VLAN\nguest firewall",
    )
    .expect("w");
    fs::write(
        outside.join("notes").join("deep").join("isp.md"),
        "# ISP\nfirewall",
    )
    .expect("w");
    fs::write(outside.join("notes").join("admin-password.md"), "hunter2").expect("w");
    fs::write(outside.join("not-markdown.conf"), "x").expect("w");

    let before =
        build_context_snapshot(engine.paths(), Some(&project), Some("firewall")).expect("context");
    assert!(
        before
            .project_files
            .contains(&project.path.join("Specs").join("plan.md"))
    );
    assert!(
        before
            .project_files
            .contains(&project.path.join("Resources").join("router.md"))
    );
    assert!(before.granted_files.is_empty(), "nothing granted yet");

    engine
        .grant(Some("sec"), &granted_file.display().to_string())
        .expect("grant a file");
    engine
        .grant(Some("sec"), &outside.join("notes").display().to_string())
        .expect("grant a folder");

    let canonical = |p: PathBuf| fs::canonicalize(p).expect("exists");
    let after =
        build_context_snapshot(engine.paths(), Some(&project), Some("firewall")).expect("context");
    assert!(
        after
            .granted_files
            .contains(&canonical(granted_file.clone()))
    );
    assert!(
        after
            .granted_files
            .contains(&canonical(outside.join("notes")).join("vlan.md"))
    );
    assert!(
        after
            .granted_files
            .contains(&canonical(outside.join("notes")).join("deep").join("isp.md")),
        "a granted folder is read two levels deep"
    );
    assert!(after.assembled_content.contains("drops WAN SSH"));
    assert!(
        !after.assembled_content.contains("hunter2"),
        "the block list still applies"
    );
    assert!(
        after
            .skipped_files
            .contains(&canonical(outside.join("notes")).join("admin-password.md"))
    );

    let kinds = readable_sources(engine.paths(), Some(&project)).expect("sources");
    assert!(
        kinds
            .sources
            .iter()
            .all(|(kind, path)| *kind != SourceKind::Granted
                || path.starts_with(canonical(outside.clone()))),
        "only granted paths are tagged granted"
    );
    assert!(
        kinds
            .sources
            .iter()
            .all(|(_, path)| path.extension().is_some_and(|e| e == "md")),
        "Markdown only"
    );

    let found = engine.search("firewall", Some("sec"), 20).expect("search");
    assert!(
        found
            .results
            .iter()
            .any(|hit| hit.path.ends_with("router-hardening-notes.md")),
        "search sees granted files too"
    );

    engine
        .revoke(Some("sec"), &granted_file.display().to_string())
        .expect("revoke");
    let revoked =
        build_context_snapshot(engine.paths(), Some(&project), Some("firewall")).expect("context");
    assert!(
        !revoked
            .granted_files
            .contains(&canonical(granted_file.clone())),
        "revoking removes it from the context"
    );

    let _ = fs::remove_dir_all(&root);
    let _ = fs::remove_dir_all(&outside);
}
