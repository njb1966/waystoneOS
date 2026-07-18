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
