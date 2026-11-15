//! Saving the workspace folder writes the per-user settings file, found through `HOME` and
//! `XDG_CONFIG_HOME`. Those are process-wide, so this lives in its own test binary (its own
//! process) and is the only test in it.

use aiw_desktop::commands::{self, OpenSession, RootChoice};
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn saving_a_folder_writes_the_settings_and_reports_an_override() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    let base = std::env::temp_dir().join(format!("ai-workspace-desktop-root-{unique}"));
    let home = base.join("home");
    let xdg = base.join("xdg");
    let chosen = base.join("chosen");
    for dir in [&home, &xdg, &chosen] {
        std::fs::create_dir_all(dir).expect("test folders should be created");
    }
    // SAFETY: this test binary runs this one test only, so nothing else reads the
    // environment while it changes.
    unsafe {
        std::env::set_var("HOME", &home);
        std::env::set_var("XDG_CONFIG_HOME", &xdg);
        std::env::remove_var("AI_WORKSPACE_HOME");
    }

    let open = OpenSession::default();
    let choice = RootChoice::default();
    let saved =
        commands::set_root(&choice, &open, &chosen.display().to_string()).expect("save works");
    let canonical = std::fs::canonicalize(&chosen).expect("chosen folder exists");
    assert_eq!(saved.root, canonical.display().to_string());
    assert!(saved.env_override.is_none());
    assert!(saved.session_closed.is_none());
    let settings = std::fs::read_to_string(xdg.join("ai-workspace").join("config.toml"))
        .expect("the settings file is written");
    assert!(settings.contains(&canonical.display().to_string()));

    let missing = commands::set_root(&choice, &open, &base.join("nope").display().to_string())
        .expect_err("a missing folder is refused");
    assert_eq!(missing.kind, "InvalidWorkspaceRoot");

    // SAFETY: as above.
    unsafe {
        std::env::set_var("AI_WORKSPACE_HOME", base.join("elsewhere"));
    }
    let overridden =
        commands::set_root(&choice, &open, &chosen.display().to_string()).expect("saved");
    assert!(
        overridden.env_override.is_some(),
        "the variable still decides, and says so"
    );

    let _ = std::fs::remove_dir_all(&base);
}
