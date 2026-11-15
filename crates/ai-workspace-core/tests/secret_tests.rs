//! The built-in block list (ADR-0012): secret-looking names never reach context or search.

use ai_workspace_core::Engine;
use ai_workspace_core::context::build_context_snapshot;
use ai_workspace_core::paths::{RootSource, WorkspacePaths};
use ai_workspace_core::workspace::{looks_like_secret, scan_markdown_files};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_root(name: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("ai-workspace-secret-{name}-{unique}"))
}

#[test]
fn secret_looking_names_are_recognised() {
    for name in [
        "server.key",
        "CA.PEM",
        "cert.p12",
        "vault.kdbx",
        "id_rsa",
        "id_rsa.pub",
        "id_ed25519",
        ".env",
        ".env.local",
        "router-passwords.md",
        "API_TOKEN.md",
        "secrets",
        "my-Secret-notes.md",
    ] {
        assert!(looks_like_secret(name), "{name} should be blocked");
    }
    for name in ["router.md", "firewall-rules.md", "Knowledge", "keyboard.md"] {
        assert!(!looks_like_secret(name), "{name} should be allowed");
    }
    // The list errs on the safe side: any name containing "token" is skipped.
    assert!(looks_like_secret("tokenizer-notes.md"));
}

#[test]
fn scanning_skips_secret_files_and_does_not_enter_secret_folders() {
    let root = temp_root("scan");
    fs::create_dir_all(root.join("secrets")).expect("folders");
    fs::create_dir_all(root.join("notes")).expect("folders");
    fs::write(root.join("router.md"), "ok").expect("write");
    fs::write(root.join("wifi-password.md"), "hunter2").expect("write");
    fs::write(root.join("secrets").join("plain.md"), "hidden").expect("write");
    fs::write(root.join("notes").join("ssh.md"), "ok").expect("write");

    let scan = scan_markdown_files(&root, 2).expect("scan works");
    assert_eq!(
        scan.files,
        [PathBuf::from("notes/ssh.md"), PathBuf::from("router.md")]
    );
    assert_eq!(
        scan.skipped,
        [PathBuf::from("secrets"), PathBuf::from("wifi-password.md")]
    );

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn secrets_never_reach_context_or_search() {
    let root = temp_root("context");
    let engine = Engine::at(
        WorkspacePaths::from_root(root.clone()),
        RootSource::Environment,
    );
    engine.init().expect("init");
    let created = engine.create_project("Security", None).expect("project");
    let knowledge = created.project.path.join("Knowledge");
    fs::write(
        knowledge.join("router.md"),
        "# Router\nThe firewall blocks inbound.\n",
    )
    .expect("write");
    fs::write(
        knowledge.join("firewall-password.md"),
        "# Firewall\nThe firewall admin password is hunter2.\n",
    )
    .expect("write");

    let context = build_context_snapshot(engine.paths(), Some(&created.project), Some("firewall"))
        .expect("context builds");
    assert!(
        !context.assembled_content.contains("hunter2"),
        "the secret is not read"
    );
    assert!(context.assembled_content.contains("blocks inbound"));
    assert_eq!(
        context.skipped_files,
        [knowledge.join("firewall-password.md")]
    );

    let results = engine
        .search("firewall", Some("security"), 10)
        .expect("search");
    assert!(
        results
            .results
            .iter()
            .all(|hit| !hit.path.ends_with("firewall-password.md")),
        "search never shows it either"
    );

    let _ = fs::remove_dir_all(&root);
}
