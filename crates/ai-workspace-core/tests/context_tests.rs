use ai_workspace_core::config::WorkspaceConfig;
use ai_workspace_core::context::{ContextSnapshot, build_context_snapshot};
use ai_workspace_core::paths::WorkspacePaths;
use ai_workspace_core::workspace::{create_project, inspect_project};
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

fn test_paths(name: &str) -> WorkspacePaths {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("ai-workspace-context-{name}-{unique}"));
    WorkspacePaths::from_root(root)
}

#[test]
fn context_snapshot_includes_base_project_and_session_sources() {
    let paths = test_paths("snapshot-sources");
    paths
        .ensure_layout()
        .expect("workspace layout should be created for test");

    let base_dir = paths.base.join("Common");
    fs::create_dir_all(&base_dir).expect("base directory should be created");
    fs::write(base_dir.join("base.md"), "# Base knowledge")
        .expect("base markdown should be written");

    let config = WorkspaceConfig::default_for_paths(&paths);
    create_project(&paths, &config, "Context Demo", None).expect("project should be created");
    let (project, _) =
        inspect_project(&paths, &config, "Context Demo").expect("project should be inspectable");

    let project_knowledge_dir = project.path.join("Knowledge");
    fs::create_dir_all(&project_knowledge_dir)
        .expect("project knowledge directory should be created");
    fs::write(
        project_knowledge_dir.join("project.md"),
        "# Project knowledge",
    )
    .expect("project markdown should be written");

    let session_dir = project.path.join("Session");
    fs::create_dir_all(&session_dir).expect("session directory should be created");
    fs::write(session_dir.join("session.md"), "# Session resource")
        .expect("session markdown should be written");

    let snapshot = build_context_snapshot(&paths, Some(&project), None)
        .expect("context snapshot should build");

    assert_eq!(snapshot.base_files.len(), 1);
    assert_eq!(snapshot.project_files.len(), 1);
    assert_eq!(snapshot.session_files.len(), 1);
    assert_eq!(snapshot.source_count, 3);
    assert!(!snapshot.assembled_content.is_empty());

    let _ = fs::remove_dir_all(&paths.root);
}

#[test]
fn context_snapshot_includes_project_context_when_base_sources_are_abundant() {
    let paths = test_paths("project-preferred");
    paths
        .ensure_layout()
        .expect("workspace layout should be created for test");

    let base_dir = paths.base.join("Common");
    fs::create_dir_all(&base_dir).expect("base directory should be created");
    for index in 0..8 {
        let file_name = format!("base-{index}.md");
        fs::write(base_dir.join(file_name), "# Base content")
            .expect("base markdown should be written");
    }

    let config = WorkspaceConfig::default_for_paths(&paths);
    create_project(&paths, &config, "Project Preferred", None).expect("project should be created");
    let (project, _) = inspect_project(&paths, &config, "Project Preferred")
        .expect("project should be inspectable");

    let project_knowledge_dir = project.path.join("Knowledge");
    fs::create_dir_all(&project_knowledge_dir)
        .expect("project knowledge directory should be created");
    for index in 0..3 {
        let file_name = format!("project-{index}.md");
        fs::write(project_knowledge_dir.join(file_name), "# Project context")
            .expect("project markdown should be written");
    }

    let snapshot = build_context_snapshot(&paths, Some(&project), None)
        .expect("context snapshot should build");

    assert!(
        snapshot.project_files.len() >= 3,
        "all project context files should be selected"
    );
    assert!(snapshot.source_count <= ai_workspace_core::context::DEFAULT_MAX_CONTEXT_SOURCES);

    let _ = fs::remove_dir_all(&paths.root);
}

#[test]
fn context_snapshot_stays_bounded_and_deterministic() {
    let paths = test_paths("bounded-snapshot");
    paths
        .ensure_layout()
        .expect("workspace layout should be created for test");

    let base_dir = paths.base.join("Common");
    fs::create_dir_all(&base_dir).expect("base directory should be created");
    for index in 0..12 {
        let file_name = format!("base-{index}.md");
        fs::write(base_dir.join(file_name), "# Content").expect("base markdown should be written");
    }

    let config = WorkspaceConfig::default_for_paths(&paths);
    create_project(&paths, &config, "Bounded Demo", None).expect("project should be created");
    let (project, _) =
        inspect_project(&paths, &config, "Bounded Demo").expect("project should be inspectable");

    let snapshot = build_context_snapshot(&paths, Some(&project), None)
        .expect("context snapshot should build");

    assert!(snapshot.source_count <= ai_workspace_core::context::DEFAULT_MAX_CONTEXT_SOURCES);
    assert!(snapshot.total_bytes <= ai_workspace_core::context::DEFAULT_MAX_CONTEXT_BYTES);

    let _ = fs::remove_dir_all(&paths.root);
}

#[test]
fn context_snapshot_as_system_prompt_wraps_assembled_content() {
    let paths = test_paths("system-prompt");
    paths
        .ensure_layout()
        .expect("workspace layout should be created for test");

    let base_dir = paths.base.join("Common");
    fs::create_dir_all(&base_dir).expect("base directory should be created");
    fs::write(base_dir.join("base.md"), "# Base knowledge")
        .expect("base markdown should be written");

    let snapshot =
        build_context_snapshot(&paths, None, None).expect("context snapshot should build");
    let system_prompt = snapshot.as_system_prompt();

    assert!(system_prompt.contains(&snapshot.assembled_content));
    assert!(system_prompt.len() > snapshot.assembled_content.len());

    let _ = fs::remove_dir_all(&paths.root);
}

#[test]
fn context_snapshot_uses_relevance_to_include_a_topic_file_that_fixed_order_would_exclude() {
    let paths = test_paths("relevance-override");
    paths
        .ensure_layout()
        .expect("workspace layout should be created for test");

    let base_dir = paths.base.join("Common");
    fs::create_dir_all(&base_dir).expect("base directory should be created");
    fs::write(
        base_dir.join("ollama-setup.md"),
        "# Ollama Setup\nOllama runtime installation notes.",
    )
    .expect("base markdown should be written");

    let config = WorkspaceConfig::default_for_paths(&paths);
    create_project(&paths, &config, "Relevance Demo", None).expect("project should be created");
    let (project, _) =
        inspect_project(&paths, &config, "Relevance Demo").expect("project should be inspectable");

    let project_knowledge_dir = project.path.join("Knowledge");
    fs::create_dir_all(&project_knowledge_dir)
        .expect("project knowledge directory should be created");
    for index in 0..8 {
        let file_name = format!("project-{index}.md");
        fs::write(
            project_knowledge_dir.join(file_name),
            "# Generic project note\nNothing topic-specific here.",
        )
        .expect("project markdown should be written");
    }

    // Under v0.4's fixed priority order, these 8 generic project files alone
    // already fill the cap, so the relevant base file would never be
    // selected. Relevance mode should surface it anyway.
    let snapshot = build_context_snapshot(&paths, Some(&project), Some("ollama"))
        .expect("context snapshot should build");

    assert!(
        snapshot
            .base_files
            .iter()
            .any(|path| path.ends_with("ollama-setup.md")),
        "the topic-relevant base file should be included when ranked by relevance, got: {:?}",
        snapshot.base_files
    );
    assert!(snapshot.source_count <= ai_workspace_core::context::DEFAULT_MAX_CONTEXT_SOURCES);

    let _ = fs::remove_dir_all(&paths.root);
}

#[test]
fn context_snapshot_no_project_no_query_matches_v04_fixed_order() {
    let paths = test_paths("no-project-no-query");
    paths
        .ensure_layout()
        .expect("workspace layout should be created for test");

    let base_dir = paths.base.join("Common");
    fs::create_dir_all(&base_dir).expect("base directory should be created");
    for index in 0..10 {
        let file_name = format!("base-{index}.md");
        fs::write(base_dir.join(file_name), "# Content").expect("base markdown should be written");
    }

    let first = build_context_snapshot(&paths, None, None).expect("context snapshot should build");
    let second = build_context_snapshot(&paths, None, None).expect("context snapshot should build");

    assert_eq!(
        first.base_files, second.base_files,
        "selection should be deterministic across calls"
    );
    assert_eq!(
        first.base_files.len(),
        ai_workspace_core::context::DEFAULT_MAX_CONTEXT_SOURCES
    );
    assert!(first.project_files.is_empty());
    assert!(first.session_files.is_empty());

    let _ = fs::remove_dir_all(&paths.root);
}

#[test]
fn agent_instructions_come_first_and_the_context_follows() {
    let snapshot = ContextSnapshot {
        base_files: Vec::new(),
        project_files: Vec::new(),
        session_files: Vec::new(),
        assembled_content: "--- router.md ---\nSSH only from the LAN.".to_string(),
        source_count: 1,
        total_bytes: 40,
        skipped_files: Vec::new(),
    };

    let with_agent = snapshot.as_system_prompt_for(Some("You are a security advisor."));
    assert!(with_agent.starts_with("You are a security advisor."));
    let rules_at = with_agent
        .find("security advisor")
        .expect("instructions present");
    let context_at = with_agent
        .find("SSH only from the LAN.")
        .expect("context present");
    assert!(
        rules_at < context_at,
        "the model's rules come before the context"
    );

    assert_eq!(
        snapshot.as_system_prompt_for(None),
        snapshot.as_system_prompt()
    );
}
