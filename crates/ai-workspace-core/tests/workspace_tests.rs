use ai_workspace_core::config::WorkspaceConfig;
use ai_workspace_core::error::AppError;
use ai_workspace_core::paths::WorkspacePaths;
use ai_workspace_core::workspace::{
    PermissionStatus, PermissionViewOptions, create_project, grant_temporary_permission,
    inspect_project, normalize_project_name, revoke_temporary_permission, validate_workspace,
    view_permissions,
};
use std::time::{SystemTime, UNIX_EPOCH};

fn test_paths(name: &str) -> WorkspacePaths {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("ai-workspace-{name}-{unique}"));
    WorkspacePaths::from_root(root)
}

#[test]
fn normalize_project_name_accepts_valid_name() {
    let normalized =
        normalize_project_name("Demo Project").expect("valid project name should normalize");
    assert_eq!(normalized, "demo-project");
}

#[test]
fn normalize_project_name_rejects_reserved_name() {
    let result = normalize_project_name("Config");
    assert!(matches!(result, Err(AppError::InvalidProjectName(_))));
}

#[test]
fn normalize_project_name_rejects_invalid_characters() {
    let result = normalize_project_name("my/project");
    assert!(matches!(result, Err(AppError::InvalidProjectName(_))));
}

#[test]
fn validate_workspace_reports_missing_config_as_blocking_issue() {
    let paths = test_paths("validate-missing-config");
    paths
        .ensure_layout()
        .expect("workspace layout should be created for test");

    let config_path = paths.workspace_config_file();
    if config_path.exists() {
        std::fs::remove_file(&config_path).expect("test should remove workspace config");
    }

    let result = validate_workspace(&paths).expect("validation should return result");
    assert!(!result.valid);
    assert!(result.blocking_issue_count() >= 1);
    assert!(
        result
            .workspace_issues
            .iter()
            .any(|issue| issue.code == "WS-CONFIG-MISSING")
    );

    let _ = std::fs::remove_dir_all(&paths.root);
}

#[test]
fn permission_grant_is_idempotent_for_same_path() {
    let paths = test_paths("perm-idempotent");
    paths
        .ensure_layout()
        .expect("workspace layout should be created for test");

    let config = WorkspaceConfig::default_for_paths(&paths);
    create_project(&paths, &config, "Perm Demo", None).expect("project should be created");
    let (project, _) =
        inspect_project(&paths, &config, "Perm Demo").expect("project should be inspectable");

    let first =
        grant_temporary_permission(&paths, &project, "/tmp").expect("first grant should succeed");
    let second = grant_temporary_permission(&paths, &project, "/tmp")
        .expect("duplicate grant should succeed idempotently");

    assert!(first.created);
    assert!(!second.created);

    let _ = std::fs::remove_dir_all(&paths.root);
}

#[test]
fn permission_revoke_missing_returns_not_found() {
    let paths = test_paths("perm-revoke-missing");
    paths
        .ensure_layout()
        .expect("workspace layout should be created for test");

    let config = WorkspaceConfig::default_for_paths(&paths);
    create_project(&paths, &config, "Perm Demo", None).expect("project should be created");
    let (project, _) =
        inspect_project(&paths, &config, "Perm Demo").expect("project should be inspectable");

    let result =
        revoke_temporary_permission(&paths, &project, "/tmp/does-not-exist-for-revoke-test");
    assert!(matches!(result, Err(AppError::PermissionGrantNotFound(_))));

    let _ = std::fs::remove_dir_all(&paths.root);
}

#[test]
fn permission_view_marks_deleted_grant_unavailable() {
    let paths = test_paths("perm-unavailable");
    paths
        .ensure_layout()
        .expect("workspace layout should be created for test");

    let config = WorkspaceConfig::default_for_paths(&paths);
    create_project(&paths, &config, "Perm Demo", None).expect("project should be created");
    let (project, _) =
        inspect_project(&paths, &config, "Perm Demo").expect("project should be inspectable");

    let temp_file = std::env::temp_dir().join("ai-workspace-permission-view-unavailable.tmp");
    std::fs::write(&temp_file, "x").expect("temp file should be writable");

    grant_temporary_permission(&paths, &project, temp_file.to_string_lossy().as_ref())
        .expect("grant should succeed");
    std::fs::remove_file(&temp_file).expect("temp file should be removed for unavailable check");

    let snapshot = view_permissions(
        &paths,
        &project,
        &PermissionViewOptions {
            filter: None,
            page: 1,
            page_size: 50,
        },
    )
    .expect("view should succeed");

    let temporary = snapshot
        .entries
        .iter()
        .find(|entry| entry.category == ai_workspace_core::workspace::PermissionCategory::Temporary)
        .expect("temporary grant entry should be present");
    assert_eq!(temporary.status, PermissionStatus::Unavailable);

    let _ = std::fs::remove_dir_all(&paths.root);
}
