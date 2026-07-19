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

#[test]
fn planned_history_json_reports_inspectable_record() {
    let output = Command::new(env!("CARGO_BIN_EXE_publish"))
        .args([
            "--planned-history",
            "--project",
            &repo_path("examples/projects/ssh-capsule.wayproject"),
            "--target",
            "production",
            "--date",
            "2026-07-18T00:00:00Z",
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
    assert!(stdout.contains("\"transfer_result\":\"planned\""));
    assert!(stdout.contains("\"verification_result\":\"not-run\""));
    assert!(stdout.contains("\"files\":["));
    assert!(stdout.contains("\"action\":\"planned-upload\""));
    assert!(stdout.contains("[publication]\\n"));
    assert!(stdout.contains("planned-upload"));
    assert!(stdout.contains("content/index.gmi"));
}
