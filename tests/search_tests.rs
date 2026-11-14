use ai_workspace::config::{WorkspaceConfig, ensure_default_workspace_config};
use ai_workspace::paths::WorkspacePaths;
use ai_workspace::workspace::create_project;
use assert_cmd::Command;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

fn test_paths(name: &str) -> WorkspacePaths {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("ai-workspace-search-{name}-{unique}"));
    WorkspacePaths::from_root(root)
}

/// Bootstraps a workspace that will pass `enforce_validation_gate` and
/// creates a project with the given Knowledge files.
fn bootstrap_project_with_knowledge(
    paths: &WorkspacePaths,
    project_name: &str,
    files: &[(&str, &str)],
) {
    paths
        .ensure_layout()
        .expect("workspace layout should be created for test");
    ensure_default_workspace_config(paths, false).expect("workspace config should bootstrap");

    let config = WorkspaceConfig::default_for_paths(paths);
    create_project(paths, &config, project_name, None).expect("project should be created");

    let knowledge_dir = paths.projects.join(project_name).join("Knowledge");
    fs::create_dir_all(&knowledge_dir).expect("knowledge directory should exist");
    for (file_name, content) in files {
        fs::write(knowledge_dir.join(file_name), content)
            .expect("knowledge markdown should be written");
    }
}

#[test]
fn search_ranks_on_topic_documents_above_unrelated_ones() {
    let paths = test_paths("ranked");
    bootstrap_project_with_knowledge(
        &paths,
        "demo",
        &[
            (
                "ollama-setup.md",
                "# Ollama Setup\nInstall Ollama locally and start the Ollama runtime before running ai.\n",
            ),
            (
                "notes.md",
                "# Notes\nGeneral runtime notes for the workspace, unrelated to any specific tool.\n",
            ),
            (
                "permissions.md",
                "# Permissions\nTemporary permissions are granted per session and revoked on exit.\n",
            ),
        ],
    );

    let mut cmd = Command::cargo_bin("ai-workspace").expect("binary should exist");
    cmd.env("AI_WORKSPACE_HOME", &paths.root);
    cmd.args(["search", "--query", "ollama runtime", "--project", "demo"]);
    let assert = cmd.assert().success();
    let output = assert.get_output();
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stdout.contains("ollama-setup.md"),
        "on-topic document should appear in results, got:\n{stdout}"
    );
    assert!(
        !stdout.contains("permissions.md"),
        "unrelated document should not appear in results, got:\n{stdout}"
    );

    let ollama_index = stdout
        .find("ollama-setup.md")
        .expect("ollama-setup.md should be present");
    if let Some(notes_index) = stdout.find("notes.md") {
        assert!(
            ollama_index < notes_index,
            "the stronger match should rank ahead of the weaker one, got:\n{stdout}"
        );
    }

    let _ = fs::remove_dir_all(&paths.root);
}

#[test]
fn search_reports_no_results_clearly_instead_of_erroring() {
    let paths = test_paths("no-results");
    bootstrap_project_with_knowledge(
        &paths,
        "demo",
        &[("notes.md", "# Notes\nGeneral workspace notes.\n")],
    );

    let mut cmd = Command::cargo_bin("ai-workspace").expect("binary should exist");
    cmd.env("AI_WORKSPACE_HOME", &paths.root);
    cmd.args([
        "search",
        "--query",
        "kubernetes cluster autoscaling",
        "--project",
        "demo",
    ]);
    let assert = cmd.assert().success();
    let output = assert.get_output();
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stdout.contains("No documents matched"),
        "a distinct no-results message should be shown, got:\n{stdout}"
    );

    let _ = fs::remove_dir_all(&paths.root);
}

#[test]
fn search_never_returns_content_outside_granted_permission_paths() {
    let paths = test_paths("permission-boundary");
    bootstrap_project_with_knowledge(
        &paths,
        "demo",
        &[(
            "inside.md",
            "# Inside\nThis file about ollama lives inside the granted project path.\n",
        )],
    );

    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    let outside_dir = std::env::temp_dir().join(format!("ai-search-outside-{unique}"));
    fs::create_dir_all(&outside_dir).expect("outside directory should be created");
    fs::write(
        outside_dir.join("secret.md"),
        "# Secret\nUngranted ollama content that must never leak into search results.\n",
    )
    .expect("outside markdown should be written");

    let mut cmd = Command::cargo_bin("ai-workspace").expect("binary should exist");
    cmd.env("AI_WORKSPACE_HOME", &paths.root);
    cmd.args(["search", "--query", "ollama", "--project", "demo"]);
    let assert = cmd.assert().success();
    let output = assert.get_output();
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stdout.contains("inside.md"),
        "the granted document should still be found, got:\n{stdout}"
    );
    assert!(
        !stdout.contains("secret.md") && !stdout.contains("Ungranted"),
        "content outside any granted permission path must never appear, got:\n{stdout}"
    );

    let _ = fs::remove_dir_all(&paths.root);
    let _ = fs::remove_dir_all(&outside_dir);
}

#[test]
fn search_works_as_the_first_command_without_any_prior_session() {
    let paths = test_paths("no-session-needed");
    bootstrap_project_with_knowledge(
        &paths,
        "demo",
        &[(
            "ollama-setup.md",
            "# Ollama Setup\nInstall Ollama locally before running ai.\n",
        )],
    );

    // No `ai` interactive session has ever run against this workspace —
    // `ai-workspace search` is the very first command invoked.
    let mut cmd = Command::cargo_bin("ai-workspace").expect("binary should exist");
    cmd.env("AI_WORKSPACE_HOME", &paths.root);
    cmd.args(["search", "--query", "ollama", "--project", "demo"]);
    let assert = cmd.assert().success();
    let output = assert.get_output();
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stdout.contains("ollama-setup.md"),
        "result should include the file path, got:\n{stdout}"
    );
    assert!(
        stdout.contains('"'),
        "result should include a quoted snippet, got:\n{stdout}"
    );

    let _ = fs::remove_dir_all(&paths.root);
}
