//! The context budget follows the model's window, and a file that does not fit is cut with a
//! visible marker (ADR-0012).

use ai_workspace_core::Engine;
use ai_workspace_core::context::{
    ContextBudget, DEFAULT_MAX_CONTEXT_BYTES, build_context_snapshot_within,
};
use ai_workspace_core::paths::{RootSource, WorkspacePaths};
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn the_budget_follows_the_window_and_leaves_room_for_instructions() {
    let small = ContextBudget::for_window(8192, 0);
    assert_eq!(small.max_bytes, 12288);
    assert_eq!(small.max_sources, 8);
    assert_eq!(small.window_tokens, Some(8192));

    let large = ContextBudget::for_window(32768, 0);
    assert_eq!(large.max_bytes, 49152);
    assert_eq!(
        large.max_sources, 32,
        "sources grow with the budget, up to 32"
    );

    let with_rules = ContextBudget::for_window(8192, 2000);
    assert_eq!(
        with_rules.max_bytes, 10288,
        "the model's instructions take their share"
    );

    assert_eq!(
        ContextBudget::default().max_bytes,
        DEFAULT_MAX_CONTEXT_BYTES
    );
}

#[test]
fn a_file_that_does_not_fit_is_cut_and_marked() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("ai-workspace-budget-{unique}"));
    let engine = Engine::at(
        WorkspacePaths::from_root(root.clone()),
        RootSource::Environment,
    );
    engine.init().expect("init");
    let created = engine.create_project("Security", None).expect("project");
    let knowledge = created.project.path.join("Knowledge");
    // A long history, line by line, with non-ASCII text so a cut must respect characters.
    let history: String = (1..=400)
        .map(|n| format!("Muutos {n}: palomuurin sääntö päivitetty, firewall rule updated.\n"))
        .collect();
    fs::write(knowledge.join("history.md"), &history).expect("write");
    fs::write(knowledge.join("short.md"), "# Short\nfirewall note\n").expect("write");

    let budget = ContextBudget {
        max_sources: 8,
        max_bytes: 4000,
        window_tokens: None,
    };
    let context = build_context_snapshot_within(
        engine.paths(),
        Some(&created.project),
        Some("firewall"),
        budget,
    )
    .expect("context builds");

    assert_eq!(context.truncated_files.len(), 1);
    let cut = &context.truncated_files[0];
    assert!(cut.path.ends_with("history.md"));
    assert_eq!(cut.total, history.len());
    assert!(cut.shown < cut.total && cut.shown > 0);
    let marker = format!("[truncated: {} of {} bytes shown]", cut.shown, cut.total);
    assert!(
        context.assembled_content.contains(&marker),
        "the cut is visible in the text"
    );
    assert!(context.total_bytes <= 4000);
    assert!(
        context.assembled_content.contains("firewall note"),
        "a smaller file that fits is still included whole"
    );

    let _ = fs::remove_dir_all(&root);
}
