//! Ollama in Podman (ADR-0016): the settings in workspace.toml and the exact `podman run`
//! arguments. Nothing here runs Podman.

use ai_workspace_core::config::{
    Gpu, OLLAMA_IMAGE, OLLAMA_ROCM_IMAGE, PodmanSettings, Runtime, WorkspaceConfig,
};
use ai_workspace_core::runtime::{PODMAN_NAME, podman_pull_command, podman_run_args};

fn parse(toml_text: &str) -> WorkspaceConfig {
    toml::from_str(toml_text).expect("the configuration should parse")
}

const BASE: &str = r#"
root_path = "/w"
base_path = "/w/Base"
projects_path = "/w/Projects"
cache_path = "/w/Cache"
logs_path = "/w/Logs"
config_path = "/w/Config"
default_project_directories = ["Knowledge", "Specs", "Resources", "Session"]
project_metadata_file = "project.toml"
default_model = "qwen3:8b"
"#;

#[test]
fn a_file_without_the_new_keys_keeps_using_the_installed_ollama() {
    let config = parse(BASE);
    assert_eq!(config.runtime, Runtime::Ollama);
    assert_eq!(config.podman, PodmanSettings::default());
    assert_eq!(config.podman.gpu, Gpu::None);
}

#[test]
fn the_podman_settings_are_read_and_unknown_keys_ignored() {
    let config = parse(&format!(
        "{BASE}runtime = \"podman\"\nsomething_from_a_later_version = 1\n[podman]\ngpu = \"nvidia\"\nimage = \"localhost/my-ollama\"\n"
    ));
    assert_eq!(config.runtime, Runtime::Podman);
    assert_eq!(config.podman.gpu, Gpu::Nvidia);
    assert_eq!(config.podman.image(), "localhost/my-ollama");
}

#[test]
fn the_default_image_follows_the_gpu_setting() {
    let cpu = PodmanSettings::default();
    assert_eq!(cpu.image(), OLLAMA_IMAGE);
    let amd = PodmanSettings {
        image: None,
        gpu: Gpu::Amd,
    };
    assert_eq!(amd.image(), OLLAMA_ROCM_IMAGE);
    let blank = PodmanSettings {
        image: Some("  ".into()),
        gpu: Gpu::None,
    };
    assert_eq!(
        blank.image(),
        OLLAMA_IMAGE,
        "a blank image means the default"
    );
}

#[test]
fn the_container_never_pulls_and_is_published_on_localhost_only() {
    let args = podman_run_args(&PodmanSettings::default());
    assert_eq!(
        args,
        [
            "run",
            "--detach",
            "--rm",
            "--pull=never",
            "--name",
            PODMAN_NAME,
            "--publish",
            "127.0.0.1:11434:11434",
            "--volume",
            "ai-workspace-ollama:/root/.ollama",
            OLLAMA_IMAGE,
        ]
    );
    assert!(
        !args
            .iter()
            .any(|a| a.starts_with("0.0.0.0") || a == "11434:11434"),
        "never published on every interface"
    );
}

#[test]
fn the_gpu_setting_adds_its_devices() {
    let nvidia = podman_run_args(&PodmanSettings {
        image: None,
        gpu: Gpu::Nvidia,
    });
    assert!(
        nvidia
            .windows(2)
            .any(|w| w == ["--device", "nvidia.com/gpu=all"])
    );
    assert_eq!(nvidia.last().map(String::as_str), Some(OLLAMA_IMAGE));

    let amd = podman_run_args(&PodmanSettings {
        image: None,
        gpu: Gpu::Amd,
    });
    assert!(amd.windows(2).any(|w| w == ["--device", "/dev/kfd"]));
    assert!(amd.windows(2).any(|w| w == ["--device", "/dev/dri"]));
    assert_eq!(amd.last().map(String::as_str), Some(OLLAMA_ROCM_IMAGE));
}

#[test]
fn selinux_labelling_is_turned_off_only_for_a_gpu() {
    // Fedora's SELinux otherwise keeps Ollama in the container from using the card, and it
    // quietly runs on the CPU (issue #53).
    let label = "--security-opt=label=disable";
    for gpu in [Gpu::Nvidia, Gpu::Amd] {
        let args = podman_run_args(&PodmanSettings { image: None, gpu });
        assert_eq!(
            args.iter().filter(|a| *a == label).count(),
            1,
            "{gpu:?}: {args:?}"
        );
    }
    assert!(
        !podman_run_args(&PodmanSettings::default())
            .iter()
            .any(|a| a.starts_with("--security-opt")),
        "without a GPU the container keeps its SELinux separation"
    );
}

#[test]
fn a_missing_image_is_explained_with_the_pull_command() {
    assert_eq!(
        podman_pull_command(&PodmanSettings::default()),
        format!("podman pull {OLLAMA_IMAGE}")
    );
}

#[test]
fn a_written_default_configuration_reads_back_the_same() {
    let paths = ai_workspace_core::paths::WorkspacePaths::from_root("/w".into());
    let config = WorkspaceConfig::default_for_paths(&paths);
    let text = toml::to_string_pretty(&config).expect("serializes");
    let back: WorkspaceConfig = toml::from_str(&text).expect("reads back");
    assert_eq!(back.runtime, Runtime::Ollama);
    assert_eq!(back.podman, PodmanSettings::default());
    assert!(!text.contains("image ="), "an unset image is not written");
}
