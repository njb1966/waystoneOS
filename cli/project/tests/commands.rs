use std::fs;
use std::process::Command;

fn repo_path(relative: &str) -> String {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(relative)
        .display()
        .to_string()
}

#[test]
fn validate_reports_invalid_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_project"))
        .args([
            "validate",
            &repo_path("tests/fixtures/projects/invalid-missing-index.wayproject"),
        ])
        .output()
        .expect("project command should run");

    assert_eq!(output.status.code(), Some(3));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("missing_content_index"));
}

#[test]
fn create_audio_series_scaffolds_audio_defaults() {
    let root = std::env::temp_dir().join(format!("waystone-project-cli-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("temp root should be created");

    let output = Command::new(env!("CARGO_BIN_EXE_project"))
        .args([
            "create",
            "--json",
            root.to_str().expect("temp root path"),
            "audio-cli",
            "Audio CLI",
            "audio-series",
        ])
        .output()
        .expect("project command should run");

    assert_eq!(output.status.code(), Some(0));
    let project = root.join("audio-cli.wayproject");
    assert!(project.join("audio/masters").is_dir());
    assert!(project.join("audio/published").is_dir());
    assert!(project.join("audio/metadata").is_dir());
    assert!(project.join("feeds/feed.xml").is_file());

    let manifest = fs::read_to_string(project.join("project.toml")).expect("manifest");
    assert!(manifest.contains("type = \"audio-series\""));
    assert!(manifest.contains("[audio]"));
    assert!(manifest.contains("[feed]"));

    let _ = fs::remove_dir_all(root);
}
