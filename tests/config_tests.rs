use ai_workspace::config::{ensure_default_workspace_config, load_workspace_config};
use ai_workspace::error::AppError;
use ai_workspace::paths::WorkspacePaths;
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
fn bootstrap_creates_default_workspace_config() {
    let paths = test_paths("bootstrap-create");
    paths
        .ensure_layout()
        .expect("workspace layout should be created for test");

    let bootstrap =
        ensure_default_workspace_config(&paths, false).expect("default config bootstrap should succeed");

    assert!(bootstrap.created);
    assert!(bootstrap.path.exists());

    let loaded = load_workspace_config(&paths).expect("config should be loadable");
    assert_eq!(loaded.project_metadata_file, "project.toml");

    let _ = std::fs::remove_dir_all(&paths.root);
}

#[test]
fn bootstrap_preserves_existing_config_when_overwrite_false() {
    let paths = test_paths("bootstrap-preserve");
    paths
        .ensure_layout()
        .expect("workspace layout should be created for test");

    let path = paths.workspace_config_file();
    std::fs::write(
        &path,
        "root_path = \"custom\"\nbase_path = \"custom\"\nprojects_path = \"custom\"\ncache_path = \"custom\"\nlogs_path = \"custom\"\nconfig_path = \"custom\"\ndefault_project_directories = [\"Knowledge\"]\nproject_metadata_file = \"project.toml\"\n",
    )
    .expect("test config should be writable");

    let bootstrap = ensure_default_workspace_config(&paths, false)
        .expect("bootstrap should not overwrite existing config");

    assert!(!bootstrap.created);

    let loaded = load_workspace_config(&paths).expect("config should still load");
    assert_eq!(loaded.root_path, "custom");

    let _ = std::fs::remove_dir_all(&paths.root);
}

#[test]
fn load_missing_config_returns_config_missing() {
    let paths = test_paths("missing-config");
    paths
        .ensure_layout()
        .expect("workspace layout should be created for test");

    let result = load_workspace_config(&paths);
    assert!(matches!(result, Err(AppError::ConfigMissing(_))));

    let _ = std::fs::remove_dir_all(&paths.root);
}
