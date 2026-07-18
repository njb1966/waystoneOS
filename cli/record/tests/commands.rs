use std::process::Command;

fn repo_path(relative: &str) -> String {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(relative)
        .display()
        .to_string()
}

#[test]
fn validate_rejects_invalid_audio_path() {
    let output = Command::new(env!("CARGO_BIN_EXE_record"))
        .args([
            "validate",
            &repo_path("tests/fixtures/audio/invalid-path/field-note.toml"),
        ])
        .output()
        .expect("record command should run");

    assert_eq!(output.status.code(), Some(3));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("invalid_audio_path"));
}
