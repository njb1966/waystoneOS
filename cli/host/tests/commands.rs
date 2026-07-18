use std::process::Command;

fn repo_path(relative: &str) -> String {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(relative)
        .display()
        .to_string()
}

#[test]
fn validate_rejects_bad_trust_state() {
    let output = Command::new(env!("CARGO_BIN_EXE_host"))
        .args([
            "validate",
            &repo_path("tests/fixtures/hosts/invalid-trust/host.toml"),
        ])
        .output()
        .expect("host command should run");

    assert_eq!(output.status.code(), Some(3));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("unsupported_trust_state"));
}
