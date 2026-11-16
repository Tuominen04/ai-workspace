//! The core's front-end API returns data instead of printing (ADR-0008).

use ai_workspace_core::paths::{RootSource, WorkspacePaths};
use ai_workspace_core::workspace::{Action, SuggestedFix};
use ai_workspace_core::{AppError, Engine, GatedOperation};
use std::time::{SystemTime, UNIX_EPOCH};

fn test_engine(name: &str) -> Engine {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("ai-workspace-engine-{name}-{unique}"));
    Engine::at(WorkspacePaths::from_root(root), RootSource::Environment)
}

#[test]
fn create_and_search_return_their_results() {
    let engine = test_engine("create-search");
    engine.init().expect("init should succeed");

    let created = engine
        .create_project("Router", Some("Firewall notes"))
        .expect("project should be created");
    assert_eq!(created.project.name, "router");
    assert_eq!(created.metadata.description, "Firewall notes");

    std::fs::write(
        created.project.path.join("Knowledge").join("fw.md"),
        "# Firewall\nThe firewall blocks inbound traffic.\n",
    )
    .expect("test should write a knowledge file");

    let outcome = engine
        .search("firewall", Some("router"), 10)
        .expect("search should succeed");
    // The whole project is searched (ADR-0014), so the README holding the description
    // "Firewall notes" matches too; the knowledge note ranks first.
    assert_eq!(outcome.results.len(), 2);
    assert!(outcome.results[0].path.ends_with("fw.md"));
    assert!(outcome.results[1].path.ends_with("README.md"));

    let _ = std::fs::remove_dir_all(&engine.paths().root);
}

#[test]
fn a_refused_gated_operation_carries_the_validation_result() {
    let engine = test_engine("gate");
    engine.init().expect("init should succeed");
    let created = engine
        .create_project("Broken", None)
        .expect("project should be created");
    std::fs::write(created.project.path.join("project.toml"), "garbage = [")
        .expect("test should corrupt the project metadata");

    match engine.project_details("broken") {
        Err(AppError::ValidationGateBlocked { operation, result }) => {
            assert_eq!(operation, GatedOperation::ProjectShow);
            assert!(result.blocking_issue_count() >= 1);
        }
        other => panic!("expected the gate to refuse, got {other:?}"),
    }

    let _ = std::fs::remove_dir_all(&engine.paths().root);
}

#[test]
fn validation_fixes_name_actions_not_commands() {
    let engine = test_engine("fix");
    engine.init().expect("init should succeed");
    std::fs::remove_file(engine.paths().workspace_config_file())
        .expect("test should remove the workspace config");

    let result = engine.validate().expect("validate should return a result");
    let issue = result
        .workspace_issues
        .iter()
        .find(|issue| issue.code == "WS-CONFIG-MISSING")
        .expect("missing config should be reported");
    assert!(matches!(
        issue.suggested_fix,
        SuggestedFix::Run {
            action: Action::Init,
            ..
        }
    ));

    let _ = std::fs::remove_dir_all(&engine.paths().root);
}

#[test]
fn a_projects_model_is_set_cleared_and_checked() {
    let engine = test_engine("model");
    engine.init().expect("init should succeed");
    engine
        .create_project("Security", None)
        .expect("project should be created");

    let (_, metadata) = engine
        .set_project_model("security", Some("  sec  "))
        .expect("a model can be set");
    assert_eq!(metadata.model.as_deref(), Some("sec"));
    let (_, reread) = engine.project_details("security").expect("details load");
    assert_eq!(
        reread.model.as_deref(),
        Some("sec"),
        "it is saved in project.toml"
    );

    let (_, cleared) = engine
        .set_project_model("security", Some(""))
        .expect("blank clears it");
    assert_eq!(cleared.model, None);

    let refused = engine.set_project_model("security", Some("two words"));
    assert!(matches!(refused, Err(AppError::InvalidModelName(_))));

    let _ = std::fs::remove_dir_all(&engine.paths().root);
}
