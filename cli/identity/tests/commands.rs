use std::process::Command;

fn repo_path(relative: &str) -> String {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(relative)
        .display()
        .to_string()
}

#[test]
fn validate_rejects_private_key_material() {
    let output = Command::new(env!("CARGO_BIN_EXE_identity"))
        .args([
            "validate",
            &repo_path("tests/fixtures/identities/private-key-leak/identity.toml"),
        ])
        .output()
        .expect("identity command should run");

    assert_eq!(output.status.code(), Some(3));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("private_key_material_present"));
}
