use std::process::Command;

fn repo_path(relative: &str) -> String {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(relative)
        .display()
        .to_string()
}

#[test]
fn library_json_lists_recording_metadata() {
    let output = Command::new(env!("CARGO_BIN_EXE_listen"))
        .args([
            "library",
            &repo_path("examples/projects/audio-capsule.wayproject/audio/metadata"),
            "--json",
        ])
        .output()
        .expect("listen command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"id\":\"field-note\""));
    assert!(stdout.contains("\"title\":\"Field Note\""));
}
