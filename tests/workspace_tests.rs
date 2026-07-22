use ai_workspace::error::AppError;
use ai_workspace::paths::WorkspacePaths;
use ai_workspace::workspace::{normalize_project_name, validate_workspace};
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
    let normalized = normalize_project_name("Demo Project").expect("valid project name should normalize");
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
    assert!(result
        .workspace_issues
        .iter()
        .any(|issue| issue.code == "WS-CONFIG-MISSING"));

    let _ = std::fs::remove_dir_all(&paths.root);
}
