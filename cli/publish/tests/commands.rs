use std::process::Command;

fn repo_path(relative: &str) -> String {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(relative)
        .display()
        .to_string()
}

#[test]
fn dry_run_json_reports_resolved_metadata() {
    let output = Command::new(env!("CARGO_BIN_EXE_publish"))
        .args([
            "--dry-run",
            "--project",
            &repo_path("examples/projects/ssh-capsule.wayproject"),
            "--target",
            "production",
            "--hosts",
            &repo_path("examples/connections/hosts"),
            "--identities",
            &repo_path("examples/connections/identities"),
            "--json",
        ])
        .output()
        .expect("publish command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"blocked\":false"));
    assert!(stdout.contains("\"host_resolution\""));
    assert!(stdout.contains("\"identity_resolution\""));
}
